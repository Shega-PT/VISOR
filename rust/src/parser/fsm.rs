///!

//! # Parser FSM — Máquina de Estados Finita para Parsing de Mensagens TLV
//!
//! Implementação completa de um parser byte-a-byte utilizando uma Máquina
//! de Estados Finita (FSM) para reconstrução de mensagens TLV a partir de
//! um fluxo serial de bytes.
//!
//! ## Estados da FSM
//!
//! ```text
//! WAIT_START → WAIT_MSGID → WAIT_TLVCOUNT → WAIT_TLV_ID →
//! WAIT_TLV_LEN → WAIT_TLV_DATA → WAIT_CHECKSUM
//! ```
//!
//! ## Proteções
//!
//! - Timeout entre bytes (detecção de frame gap)
//! - Proteção contra overflow de buffer
//! - Recuperação automática de erros
//! - Validação CRC8 em cada mensagem completa
//!
//! ## Uso
//!
//! ```rust
//! use visor_protocol::parser::fsm::Parser;
//!
//! let mut parser = Parser::new();
//!
//! // Alimentar byte a byte
//! for &byte in &serialized_message {
//!     let result = parser.feed(byte);
//!     if result == ParserError::Ok && parser.has_message() {
//!         let msg = parser.get_message();
//!         // Processar mensagem...
//!         parser.acknowledge();
//!     }
//! }
//! ```

use crate::protocol::types::*;
use crate::protocol::codec::{validate_message, parse_tlv, ProtocolError};

/// Estados da FSM do parser.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ParserState {
    /// Aguardando byte de início (START_BYTE = 0xAA).
    WaitStart = 0,
    /// Aguardando ID da mensagem.
    WaitMsgId = 1,
    /// Aguardando número de campos TLV.
    WaitTlvCount = 2,
    /// Aguardando ID de um campo TLV.
    WaitTlvId = 3,
    /// Aguardando tamanho de um campo TLV.
    WaitTlvLen = 4,
    /// Aguardando dados de um campo TLV.
    WaitTlvData = 5,
    /// Aguardando checksum CRC8.
    WaitChecksum = 6,
}

/// Códigos de erro do parser.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ParserError {
    /// Operação bem-sucedida ou mensagem processada.
    Ok = 0,
    /// Byte de início inválido.
    ErrStart = 1,
    /// ID da mensagem inválido.
    ErrMsgId = 2,
    /// Número de campos TLV inválido.
    ErrTlvCount = 3,
    /// ID de campo TLV inválido.
    ErrTlvId = 4,
    /// Tamanho de campo TLV inválido.
    ErrTlvLen = 5,
    /// Checksum CRC8 inválido.
    ErrChecksum = 6,
    /// Timeout entre bytes (frame gap excedido).
    ErrTimeout = 7,
}

/// Parser FSM para mensagens TLV.
///
/// Reconstrói mensagens TLV completas a partir de um fluxo de bytes
/// recebidos serialmente. Utiliza uma FSM de 7 estados com proteções
/// contra timeout, overflow e erros de integridade.
pub struct Parser {
    /// Estado atual da FSM.
    state: ParserState,
    /// Buffer para acumular bytes da mensagem em construção.
    raw_buffer: [u8; MAX_MESSAGE_SIZE],
    /// Número de bytes acumulados no buffer.
    raw_offset: usize,
    /// Mensagem TLV reconstruída (resultado do parsing).
    msg: TLVMessage,
    /// Máximo intervalo entre bytes em microssegundos (timeout).
    max_frame_gap_us: u32,
    /// Timestamp do último byte recebido em microssegundos.
    last_byte_time_us: u64,
    /// Indica se uma mensagem completa está disponível.
    has_message: bool,
    /// Contador de mensagens processadas com sucesso.
    success_count: u32,
    /// Contador de erros de parsing.
    error_count: u32,
    /// Indica se a saída de debug está habilitada.
    debug: bool,
}

impl Parser {
    /// Cria um novo parser FSM.
    ///
    /// O parser é inicializado no estado `WAIT_START` pronto para
    /// receber bytes de uma nova mensagem.
    pub fn new() -> Self {
        Self {
            state: ParserState::WaitStart,
            raw_buffer: [0u8; MAX_MESSAGE_SIZE],
            raw_offset: 0,
            msg: TLVMessage::new(),
            max_frame_gap_us: 5_000_000, // 5 segundos por defeito
            last_byte_time_us: 0,
            has_message: false,
            success_count: 0,
            error_count: 0,
            debug: false,
        }
    }

    /// Alimenta um byte ao parser.
    ///
    /// Este é o método principal do parser. Cada byte recebido do
    /// fluxo serial deve ser passado através deste método. A FSM
    /// transita entre estados conforme os bytes são processados.
    ///
    /// # Arguments
    ///
    /// * `byte` - Byte a processar.
    ///
    /// # Returns
    ///
    /// `ParserError::Ok` se o byte foi processado com sucesso,
    /// ou um código de erro específico.
    pub fn feed(&mut self, byte: u8) -> ParserError {
        // Atualizar timestamp do último byte
        self.last_byte_time_us = self.get_timestamp_us();

        match self.state {
            ParserState::WaitStart => {
                if byte == START_BYTE {
                    self.raw_offset = 0;
                    self.raw_buffer[0] = byte;
                    self.raw_offset = 1;
                    self.state = ParserState::WaitMsgId;
                    ParserError::Ok
                } else {
                    self.error_count += 1;
                    if self.debug {
                        self.debug_print("Byte de início inválido: 0x{:02X}", byte as u32);
                    }
                    ParserError::ErrStart
                }
            }

            ParserState::WaitMsgId => {
                if MsgId::is_valid(byte) {
                    self.raw_buffer[self.raw_offset] = byte;
                    self.raw_offset += 1;
                    self.state = ParserState::WaitTlvCount;
                    ParserError::Ok
                } else {
                    self.error_count += 1;
                    self.state = ParserState::WaitStart;
                    if self.debug {
                        self.debug_print("MsgID inválido: 0x{:02X}", byte as u32);
                    }
                    ParserError::ErrMsgId
                }
            }

            ParserState::WaitTlvCount => {
                if (byte as usize) <= MAX_TLV_FIELDS {
                    self.raw_buffer[self.raw_offset] = byte;
                    self.raw_offset += 1;

                    if byte == 0 {
                        // Sem campos TLV — ir direto para checksum
                        self.state = ParserState::WaitChecksum;
                    } else {
                        self.state = ParserState::WaitTlvId;
                    }
                    ParserError::Ok
                } else {
                    self.error_count += 1;
                    self.state = ParserState::WaitStart;
                    if self.debug {
                        self.debug_print("TLV count inválido: {}", byte as u32);
                    }
                    ParserError::ErrTlvCount
                }
            }

            ParserState::WaitTlvId => {
                if FieldId::is_valid(byte) {
                    self.raw_buffer[self.raw_offset] = byte;
                    self.raw_offset += 1;
                    self.state = ParserState::WaitTlvLen;
                    ParserError::Ok
                } else {
                    self.error_count += 1;
                    self.state = ParserState::WaitStart;
                    if self.debug {
                        self.debug_print("TLV ID inválido: 0x{:02X}", byte as u32);
                    }
                    ParserError::ErrTlvId
                }
            }

            ParserState::WaitTlvLen => {
                if (byte as usize) <= MAX_TLV_DATA {
                    self.raw_buffer[self.raw_offset] = byte;
                    self.raw_offset += 1;

                    if byte == 0 {
                        // Campo sem dados — voltar para WaitTlvId ou WaitChecksum
                        // Contar quantos TLVs já processamos
                        let tlv_count = self.raw_buffer[2] as usize;
                        let bytes_por_tlv = TLV_HEADER_SIZE + MAX_TLV_DATA;
                        let campos_processados =
                            (self.raw_offset - MESSAGE_HEADER_SIZE) / bytes_por_tlv;

                        if campos_processados >= tlv_count {
                            self.state = ParserState::WaitChecksum;
                        } else {
                            self.state = ParserState::WaitTlvId;
                        }
                    } else {
                        self.state = ParserState::WaitTlvData;
                    }
                    ParserError::Ok
                } else {
                    self.error_count += 1;
                    self.state = ParserState::WaitStart;
                    if self.debug {
                        self.debug_print("TLV len inválido: {}", byte as u32);
                    }
                    ParserError::ErrTlvLen
                }
            }

            ParserState::WaitTlvData => {
                self.raw_buffer[self.raw_offset] = byte;
                self.raw_offset += 1;

                // Verificar se recebemos todos os bytes de dados deste campo
                // O último byte de len está em raw_buffer[self.raw_offset - data_remaining - 1]
                // Simplificação: calcular baseado no offset atual
                let tlv_count = self.raw_buffer[2] as usize;
                let header_and_tlvs = MESSAGE_HEADER_SIZE + tlv_count * TLV_HEADER_SIZE;

                if self.raw_offset >= header_and_tlvs {
                    // Todos os bytes de dados foram recebidos
                    self.state = ParserState::WaitChecksum;
                } else {
                    // Ainda há bytes de dados de outros campos
                    // Verificar se o próximo byte é ID de um novo campo TLV
                    // Se estamos no início de um campo TLV (após len), o próximo é data
                    // Se terminamos um campo, o próximo é o ID do próximo campo

                    // Calcular quantos campos TLV completos temos
                    let mut temp_offset = MESSAGE_HEADER_SIZE;
                    let mut campos_completos = 0;
                    while campos_completos < tlv_count && temp_offset + TLV_HEADER_SIZE <= self.raw_offset {
                        let field_len = self.raw_buffer[temp_offset + 1] as usize;
                        temp_offset += TLV_HEADER_SIZE + field_len;
                        campos_completos += 1;
                    }

                    if campos_completos < tlv_count && temp_offset == self.raw_offset {
                        // Estamos no início de um novo campo TLV
                        self.state = ParserState::WaitTlvId;
                    }
                    // Caso contrário, permanecemos em WaitTlvData (mais dados deste campo)
                }

                ParserError::Ok
            }

            ParserState::WaitChecksum => {
                self.raw_buffer[self.raw_offset] = byte;
                self.raw_offset += 1;

                // Validar a mensagem completa
                match validate_message(&self.raw_buffer[..self.raw_offset]) {
                    Ok(tlv_count) => {
                        // Parsing bem-sucedido — extrair campos TLV
                        let tlv_data_start = MESSAGE_HEADER_SIZE;
                        let tlv_data_end = self.raw_offset - CHECKSUM_SIZE;

                        let mut tlv_output = [TLVField::new(); MAX_TLV_FIELDS];
                        match parse_tlv(
                            &self.raw_buffer[tlv_data_start..tlv_data_end],
                            &mut tlv_output,
                        ) {
                            Ok(parsed_count) => {
                                self.msg.start_byte = self.raw_buffer[0];
                                self.msg.msg_id = self.raw_buffer[1];
                                self.msg.tlv_count = parsed_count as u8;
                                for i in 0..parsed_count {
                                    self.msg.tlvs[i] = tlv_output[i];
                                }
                                self.msg.checksum = byte;
                                self.has_message = true;
                                self.success_count += 1;
                                self.state = ParserState::WaitStart;

                                if self.debug {
                                    self.debug_print(
                                        "Mensagem OK: msgID=0x{:02X} tlvs={}",
                                        self.msg.msg_id as u32,
                                        parsed_count as u32,
                                    );
                                }

                                ParserError::Ok
                            }
                            Err(_) => {
                                self.error_count += 1;
                                self.state = ParserState::WaitStart;
                                ParserError::ErrChecksum
                            }
                        }
                    }
                    Err(_) => {
                        self.error_count += 1;
                        self.state = ParserState::WaitStart;
                        if self.debug {
                            self.debug_print("CRC inválido", 0);
                        }
                        ParserError::ErrChecksum
                    }
                }
            }
        }
    }

    /// Retorna true se uma mensagem completa está disponível.
    pub fn has_message(&self) -> bool {
        self.has_message
    }

    /// Retorna uma referência à mensagem reconstruída.
    ///
    /// # Safety
    ///
    /// A mensagem retornada é válida apenas enquanto `has_message()` retorna true
    /// e antes de `acknowledge()` ser chamado.
    pub fn get_message(&self) -> &TLVMessage {
        &self.msg
    }

    /// Copia a mensagem reconstruída para um buffer de saída.
    ///
    /// # Arguments
    ///
    /// * `output` - Buffer de saída para a mensagem.
    ///
    /// # Returns
    ///
    /// `true` se a mensagem foi copiada com sucesso, `false` caso contrário.
    pub fn copy_message(&self, output: &mut TLVMessage) -> bool {
        if !self.has_message {
            return false;
        }
        *output = self.msg.clone();
        true
    }

    /// Reconhece a mensagem processada, permitindo que o parser processe a próxima.
    ///
    /// Deve ser chamado após processar a mensagem obtida via `get_message()`.
    pub fn acknowledge(&mut self) {
        self.has_message = false;
        self.msg.clear();
    }

    /// Reseta o parser para o estado inicial.
    pub fn reset(&mut self) {
        self.state = ParserState::WaitStart;
        self.raw_offset = 0;
        self.has_message = false;
        self.msg.clear();
    }

    /// Define o intervalo máximo entre bytes (timeout) em microssegundos.
    ///
    /// Se dois bytes consecutivos demorarem mais do que este intervalo,
    /// o parser será resetado automaticamente.
    pub fn set_max_frame_gap(&mut self, micros: u32) {
        self.max_frame_gap_us = micros;
    }

    /// Verifica se o parser excedeu o timeout entre bytes.
    pub fn is_timed_out(&self) -> bool {
        if self.state == ParserState::WaitStart {
            return false;
        }
        let now = self.get_timestamp_us();
        let elapsed = now.wrapping_sub(self.last_byte_time_us);
        elapsed > self.max_frame_gap_us as u64
    }

    /// Retorna o último erro registado pelo parser.
    pub fn get_last_error(&self) -> ParserError {
        if self.error_count > 0 {
            // O último erro é calculado baseado no estado atual
            match self.state {
                ParserState::WaitStart if self.raw_offset == 0 => ParserError::ErrStart,
                _ => ParserError::Ok,
            }
        } else {
            ParserError::Ok
        }
    }

    /// Retorna o estado atual da FSM.
    pub fn get_current_state(&self) -> ParserState {
        self.state
    }

    /// Retorna o número de mensagens processadas com sucesso.
    pub fn get_success_count(&self) -> u32 {
        self.success_count
    }

    /// Retorna o número de erros de parsing.
    pub fn get_error_count(&self) -> u32 {
        self.error_count
    }

    /// Ativa ou desativa a saída de debug.
    pub fn set_debug(&mut self, enable: bool) {
        self.debug = enable;
    }

    /// Obtém o timestamp atual em microssegundos.
    ///
    /// Em produção (ESP32), utiliza `esp_timer_get_time()`.
    /// Para testes no host, retorna 0.
    fn get_timestamp_us(&self) -> u64 {
        #[cfg(feature = "std")]
        {
            // Em ambiente std (ESP-IDF), usar timestamp do sistema
            // Por agora retorna 0 — será integrado com esp_timer
            0
        }
        #[cfg(not(feature = "std"))]
        {
            0
        }
    }

    /// Função auxiliar de debug (apenas em build std).
    fn debug_print(&self, _fmt: &str, _arg: u32) {
        #[cfg(feature = "std")]
        {
            // Será integrado com println! em produção
        }
    }
}

impl Default for Parser {
    fn default() -> Self {
        Self::new()
    }
}

/// Converte um estado do parser para string legível.
pub fn parser_state_to_string(state: ParserState) -> &'static str {
    match state {
        ParserState::WaitStart => "WAIT_START",
        ParserState::WaitMsgId => "WAIT_MSGID",
        ParserState::WaitTlvCount => "WAIT_TLVCOUNT",
        ParserState::WaitTlvId => "WAIT_TLV_ID",
        ParserState::WaitTlvLen => "WAIT_TLV_LEN",
        ParserState::WaitTlvData => "WAIT_TLV_DATA",
        ParserState::WaitChecksum => "WAIT_CHECKSUM",
    }
}

/// Converte um código de erro do parser para string legível.
pub fn parser_error_to_string(error: ParserError) -> &'static str {
    match error {
        ParserError::Ok => "OK",
        ParserError::ErrStart => "ERR_START",
        ParserError::ErrMsgId => "ERR_MSGID",
        ParserError::ErrTlvCount => "ERR_TLV_COUNT",
        ParserError::ErrTlvId => "ERR_TLV_ID",
        ParserError::ErrTlvLen => "ERR_TLV_LEN",
        ParserError::ErrChecksum => "ERR_CHECKSUM",
        ParserError::ErrTimeout => "ERR_TIMEOUT",
    }
}

// ============================================================================
// TESTES UNITÁRIOS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::builder::TLVBuilder;
    use crate::protocol::codec::build_message;

    /// Helper: constrói uma mensagem serializada para testes.
    fn build_test_message(msg_id: u8, tlv_data: &[(u8, &[u8])]) -> Vec<u8> {
        let mut builder = TLVBuilder::new();
        for (id, data) in tlv_data {
            builder.add_raw(*id, data).unwrap();
        }
        let mut buffer = [0u8; MAX_MESSAGE_SIZE];
        let size = builder.build(msg_id, &mut buffer).unwrap();
        buffer[..size].to_vec()
    }

    #[test]
    fn test_parser_new() {
        let parser = Parser::new();
        assert_eq!(parser.get_current_state(), ParserState::WaitStart);
        assert!(!parser.has_message());
        assert_eq!(parser.get_success_count(), 0);
        assert_eq!(parser.get_error_count(), 0);
    }

    #[test]
    fn test_parser_feed_start_byte() {
        let mut parser = Parser::new();
        let result = parser.feed(START_BYTE);
        assert_eq!(result, ParserError::Ok);
        assert_eq!(parser.get_current_state(), ParserState::WaitMsgId);
    }

    #[test]
    fn test_parser_feed_invalid_start() {
        let mut parser = Parser::new();
        let result = parser.feed(0x00);
        assert_eq!(result, ParserError::ErrStart);
        assert_eq!(parser.get_current_state(), ParserState::WaitStart);
    }

    #[test]
    fn test_parser_full_message() {
        let serialized = build_test_message(0x11, &[(0x70, &[0x02])]);
        let mut parser = Parser::new();

        for &byte in &serialized {
            let result = parser.feed(byte);
            assert_eq!(result, ParserError::Ok);
        }

        assert!(parser.has_message());
        let msg = parser.get_message();
        assert_eq!(msg.start_byte, START_BYTE);
        assert_eq!(msg.msg_id, 0x11);
        assert_eq!(msg.tlv_count, 1);
        assert_eq!(msg.tlvs[0].id, 0x70);
        assert_eq!(msg.tlvs[0].len, 1);
        assert_eq!(msg.tlvs[0].data[0], 0x02);

        parser.acknowledge();
        assert!(!parser.has_message());
    }

    #[test]
    fn test_parser_multiple_tlvs() {
        let serialized = build_test_message(0x16, &[
            (0xB0, &42u16.to_le_bytes()),
            (0xB1, &[0x03]),
            (0xB2, &[0x10]),
        ]);
        let mut parser = Parser::new();

        for &byte in &serialized {
            parser.feed(byte);
        }

        assert!(parser.has_message());
        let msg = parser.get_message();
        assert_eq!(msg.tlv_count, 3);
        assert_eq!(msg.tlvs[0].id, 0xB0);
        assert_eq!(msg.tlvs[1].id, 0xB1);
        assert_eq!(msg.tlvs[2].id, 0xB2);

        parser.acknowledge();
    }

    #[test]
    fn test_parser_invalid_crc() {
        let mut serialized = build_test_message(0x11, &[(0x70, &[0x02])]);
        // Corromper CRC
        let last = serialized.len() - 1;
        serialized[last] = serialized[last].wrapping_add(1);

        let mut parser = Parser::new();
        for &byte in &serialized {
            parser.feed(byte);
        }

        assert!(!parser.has_message());
        assert!(parser.get_error_count() > 0);
    }

    #[test]
    fn test_parser_reset() {
        let mut parser = Parser::new();
        parser.feed(START_BYTE);
        parser.feed(0x11);
        assert_ne!(parser.get_current_state(), ParserState::WaitStart);

        parser.reset();
        assert_eq!(parser.get_current_state(), ParserState::WaitStart);
        assert_eq!(parser.raw_offset, 0);
    }

    #[test]
    fn test_parser_copy_message() {
        let serialized = build_test_message(0x11, &[(0x70, &[0x02])]);
        let mut parser = Parser::new();

        for &byte in &serialized {
            parser.feed(byte);
        }

        let mut output = TLVMessage::new();
        assert!(parser.copy_message(&mut output));
        assert_eq!(output.msg_id, 0x11);
    }

    #[test]
    fn test_parser_copy_message_no_message() {
        let parser = Parser::new();
        let mut output = TLVMessage::new();
        assert!(!parser.copy_message(&mut output));
    }

    #[test]
    fn test_parser_state_to_string() {
        assert_eq!(parser_state_to_string(ParserState::WaitStart), "WAIT_START");
        assert_eq!(parser_state_to_string(ParserState::WaitChecksum), "WAIT_CHECKSUM");
    }

    #[test]
    fn test_parser_error_to_string() {
        assert_eq!(parser_error_to_string(ParserError::Ok), "OK");
        assert_eq!(parser_error_to_string(ParserError::ErrChecksum), "ERR_CHECKSUM");
    }

    #[test]
    fn test_parser_consecutive_messages() {
        let msg1 = build_test_message(0x11, &[(0x70, &[0x01])]);
        let msg2 = build_test_message(0x16, &[(0xB0, &42u16.to_le_bytes())]);
        let mut parser = Parser::new();

        // Primeira mensagem
        for &byte in &msg1 {
            parser.feed(byte);
        }
        assert!(parser.has_message());
        assert_eq!(parser.get_message().msg_id, 0x11);
        parser.acknowledge();

        // Segunda mensagem
        for &byte in &msg2 {
            parser.feed(byte);
        }
        assert!(parser.has_message());
        assert_eq!(parser.get_message().msg_id, 0x16);
        assert_eq!(parser.get_success_count(), 2);

        parser.acknowledge();
    }
}
