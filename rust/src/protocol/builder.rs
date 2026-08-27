///!

//! # TLVBuilder — Construtor de Mensagens TLV
//!
//! Implementação do construtor fluent para mensagens TLV (Type-Length-Value).
//! O TLVBuilder permite construir mensagens TLV de forma segura e eficiente,
//! adicionando campos um a um e serializando a mensagem completa no final.
//!
//! ## Uso
//!
//! ```rust
//! use visor_protocol::protocol::builder::TLVBuilder;
//!
//! let mut builder = TLVBuilder::new();
//! builder.add_uint8(0x70, 2);           // Estado do sistema
//! builder.add_float(0x30, 1.5);         // Roll do IMU
//! builder.add_uint16(0xB0, 42);         // Frame ID de vídeo
//!
//! let mut buffer = [0u8; 1093];
//! let size = builder.build(0x11, &mut buffer).unwrap();  // Telemetria
//! ```

use crate::protocol::types::*;
use crate::protocol::crc8::calc_crc8;

/// Construtor fluente para mensagens TLV.
///
/// Permite construir mensagens TLV de forma iterativa, adicionando campos
/// com diferentes tipos de dados. A mensagem é serializada e validada
/// quando `build()` é chamado.
///
/// O builder mantém internamente um array de campos TLV e um contador.
/// Quando `build()` é chamado, serializa todos os campos no buffer de
/// saída e calcula o CRC8.
pub struct TLVBuilder {
    /// Array interno de campos TLV.
    fields: [TLVField; MAX_TLV_FIELDS],
    /// Número de campos adicionados.
    tlv_count: u8,
}

impl TLVBuilder {
    /// Cria um novo TLVBuilder vazio.
    ///
    /// # Returns
    ///
    /// Um novo TLVBuilder pronto para uso.
    pub fn new() -> Self {
        Self {
            fields: [TLVField::new(); MAX_TLV_FIELDS],
            tlv_count: 0,
        }
    }

    /// Reseta o builder para o estado inicial.
    ///
    /// Limpa todos os campos adicionados e redefine o contador para zero.
    /// Útil para reutilizar o mesmo builder em múltiplas mensagens.
    pub fn reset(&mut self) {
        self.tlv_count = 0;
        for field in self.fields.iter_mut() {
            *field = TLVField::new();
        }
    }

    /// Retorna o número de campos TLV adicionados.
    pub fn get_tlv_count(&self) -> u8 {
        self.tlv_count
    }

    /// Adiciona um campo TLV com dados brutos (bytes).
    ///
    /// # Arguments
    ///
    /// * `id` - Identificador do campo TLV.
    /// * `data` - Dados a incluir no campo.
    ///
    /// # Returns
    ///
    /// `Ok(())` em sucesso, `Err(ProtocolError::TooManyFields)` se o número
    /// máximo de campos for excedido.
    pub fn add_raw(&mut self, id: u8, data: &[u8]) -> Result<(), ProtocolError> {
        if self.tlv_count as usize >= MAX_TLV_FIELDS {
            return Err(ProtocolError::TooManyFields);
        }
        let len = data.len().min(MAX_TLV_DATA);
        let field = TLVField {
            id,
            len: len as u8,
            data: {
                let mut arr = [0u8; MAX_TLV_DATA];
                arr[..len].copy_from_slice(&data[..len]);
                arr
            },
        };
        self.fields[self.tlv_count as usize] = field;
        self.tlv_count += 1;
        Ok(())
    }

    /// Adiciona um campo TLV com valor float (32 bits).
    ///
    /// # Arguments
    ///
    /// * `id` - Identificador do campo TLV.
    /// * `value` - Valor float a incluir.
    pub fn add_float(&mut self, id: u8, value: f32) -> Result<(), ProtocolError> {
        self.add_raw(id, &float_to_bytes(value))
    }

    /// Adiciona um campo TLV com valor inteiro sinalizado (32 bits).
    ///
    /// # Arguments
    ///
    /// * `id` - Identificador do campo TLV.
    /// * `value` - Valor i32 a incluir.
    pub fn add_int32(&mut self, id: u8, value: i32) -> Result<(), ProtocolError> {
        self.add_raw(id, &int32_to_bytes(value))
    }

    /// Adiciona um campo TLV com valor inteiro sem sinal (32 bits).
    ///
    /// # Arguments
    ///
    /// * `id` - Identificador do campo TLV.
    /// * `value` - Valor u32 a incluir.
    pub fn add_uint32(&mut self, id: u8, value: u32) -> Result<(), ProtocolError> {
        self.add_raw(id, &uint32_to_bytes(value))
    }

    /// Adiciona um campo TLV com valor inteiro sem sinal (16 bits).
    ///
    /// # Arguments
    ///
    /// * `id` - Identificador do campo TLV.
    /// * `value` - Valor u16 a incluir.
    pub fn add_uint16(&mut self, id: u8, value: u16) -> Result<(), ProtocolError> {
        self.add_raw(id, &uint16_to_bytes(value))
    }

    /// Adiciona um campo TLV com valor inteiro sem sinal (8 bits).
    ///
    /// # Arguments
    ///
    /// * `id` - Identificador do campo TLV.
    /// * `value` - Valor u8 a incluir.
    pub fn add_uint8(&mut self, id: u8, value: u8) -> Result<(), ProtocolError> {
        self.add_raw(id, &[value])
    }

    /// Serializa a mensagem TLV completa no buffer de saída.
    ///
    /// Constrói a mensagem no formato:
    /// ```text
    /// [START_BYTE][MSGID][TLV_COUNT][TLV_FIELDS...][CRC8]
    /// ```
    ///
    /// # Arguments
    ///
    /// * `msg_id` - Identificador do tipo de mensagem.
    /// * `buffer` - Buffer de saída para serialização.
    ///
    /// # Returns
    ///
    /// `Ok(size)` com o número total de bytes serializados em sucesso,
    /// ou `Err(ProtocolError)` em caso de erro.
    pub fn build(&self, msg_id: u8, buffer: &mut [u8]) -> Result<usize, ProtocolError> {
        // Verificar tamanho do buffer
        let required_size = MESSAGE_HEADER_SIZE
            + (self.tlv_count as usize) * (TLV_HEADER_SIZE + MAX_TLV_DATA)
            + CHECKSUM_SIZE;
        if buffer.len() < required_size {
            return Err(ProtocolError::BufferTooSmall);
        }

        let mut offset = 0;

        // Escrever cabeçalho
        buffer[offset] = START_BYTE;
        offset += 1;
        buffer[offset] = msg_id;
        offset += 1;
        buffer[offset] = self.tlv_count;
        offset += 1;

        // Escrever campos TLV
        for i in 0..self.tlv_count as usize {
            let field = &self.fields[i];
            buffer[offset] = field.id;
            offset += 1;
            buffer[offset] = field.len;
            offset += 1;
            let len = field.len as usize;
            buffer[offset..offset + len].copy_from_slice(&field.data[..len]);
            offset += len;
        }

        // Calcular CRC8 sobre todos os bytes até agora (sem o checksum)
        let crc = calc_crc8(&buffer[..offset]);
        buffer[offset] = crc;
        offset += 1;

        Ok(offset)
    }

    /// Serializa a mensagem TLV completa no buffer de saída com指定 msg_id.
    ///
    /// Equivalente a `build(msg_id, buffer)`, mantido por compatibilidade.
    pub fn serialize(&self, msg_id: u8, buffer: &mut [u8]) -> Result<usize, ProtocolError> {
        self.build(msg_id, buffer)
    }
}

impl Default for TLVBuilder {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// TESTES UNITÁRIOS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builder_new() {
        let builder = TLVBuilder::new();
        assert_eq!(builder.get_tlv_count(), 0);
    }

    #[test]
    fn test_builder_add_uint8() {
        let mut builder = TLVBuilder::new();
        builder.add_uint8(0x70, 0x02).unwrap();
        assert_eq!(builder.get_tlv_count(), 1);
    }

    #[test]
    fn test_builder_add_uint16() {
        let mut builder = TLVBuilder::new();
        builder.add_uint16(0xB0, 0x1234).unwrap();
        assert_eq!(builder.get_tlv_count(), 1);
    }

    #[test]
    fn test_builder_add_uint32() {
        let mut builder = TLVBuilder::new();
        builder.add_uint32(0x72, 0xDEADBEEF).unwrap();
        assert_eq!(builder.get_tlv_count(), 1);
    }

    #[test]
    fn test_builder_add_int32() {
        let mut builder = TLVBuilder::new();
        builder.add_int32(0x42, -12345).unwrap();
        assert_eq!(builder.get_tlv_count(), 1);
    }

    #[test]
    fn test_builder_add_float() {
        let mut builder = TLVBuilder::new();
        builder.add_float(0x30, 3.14).unwrap();
        assert_eq!(builder.get_tlv_count(), 1);
    }

    #[test]
    fn test_builder_add_raw() {
        let mut builder = TLVBuilder::new();
        let data = [1u8, 2, 3, 4, 5];
        builder.add_raw(0xB3, &data).unwrap();
        assert_eq!(builder.get_tlv_count(), 1);
    }

    #[test]
    fn test_builder_multiple_fields() {
        let mut builder = TLVBuilder::new();
        builder.add_uint8(0x70, 2).unwrap();
        builder.add_float(0x30, 1.5).unwrap();
        builder.add_uint16(0xB0, 42).unwrap();
        builder.add_uint8(0xB1, 0).unwrap();
        assert_eq!(builder.get_tlv_count(), 4);
    }

    #[test]
    fn test_builder_build() {
        let mut builder = TLVBuilder::new();
        builder.add_uint8(0x70, 2).unwrap();
        builder.add_float(0x30, 1.5).unwrap();

        let mut buffer = [0u8; 1093];
        let size = builder.build(0x11, &mut buffer).unwrap();

        // Verificar cabeçalho
        assert_eq!(buffer[0], START_BYTE);
        assert_eq!(buffer[1], 0x11);  // msg_id
        assert_eq!(buffer[2], 2);     // tlv_count

        // Verificar que o tamanho é razoável
        assert!(size > MESSAGE_HEADER_SIZE + CHECKSUM_SIZE);
        assert!(size <= MAX_MESSAGE_SIZE);
    }

    #[test]
    fn test_builder_build_verifies_crc() {
        let mut builder = TLVBuilder::new();
        builder.add_uint8(0x70, 2).unwrap();

        let mut buffer = [0u8; 1093];
        let size = builder.build(0x11, &mut buffer).unwrap();

        // CRC deve ser válido
        let crc = calc_crc8(&buffer[..size - 1]);
        assert_eq!(buffer[size - 1], crc);
    }

    #[test]
    fn test_builder_reset() {
        let mut builder = TLVBuilder::new();
        builder.add_uint8(0x70, 2).unwrap();
        builder.add_float(0x30, 1.5).unwrap();
        assert_eq!(builder.get_tlv_count(), 2);

        builder.reset();
        assert_eq!(builder.get_tlv_count(), 0);
    }

    #[test]
    fn test_builder_overflow() {
        let mut builder = TLVBuilder::new();
        // Adicionar MAX_TLV_FIELDS campos
        for i in 0..MAX_TLV_FIELDS {
            builder.add_uint8(i as u8, 0).unwrap();
        }
        assert_eq!(builder.get_tlv_count(), MAX_TLV_FIELDS as u8);

        // O próximo deve falhar
        let result = builder.add_uint8(0xFF, 0);
        assert!(result.is_err());
    }

    #[test]
    fn test_builder_buffer_too_small() {
        let mut builder = TLVBuilder::new();
        builder.add_uint8(0x70, 2).unwrap();

        let mut buffer = [0u8; 10];  // Buffer muito pequeno
        let result = builder.build(0x11, &mut buffer);
        assert!(result.is_err());
    }
}
