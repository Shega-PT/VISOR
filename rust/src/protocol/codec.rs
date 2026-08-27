///!

//! # Codec TLV — Serialização, Validação e Parsing
//!
//! Este módulo implementa as operações fundamentais de serialização,
//! validação e parsing de mensagens TLV (Type-Length-Value).
//!
//! ## Operações Principais
//!
//! - `build_tlv` — Serializa um campo TLV individual
//! - `build_message` — Serializa uma mensagem TLV completa
//! - `validate_message` — Valida integridade e estrutura de uma mensagem
//! - `parse_tlv` — Deserializa campos TLV de bytes brutos
//!
//! ## Nota sobre Endianness
//!
//! Todos os campos multi-byte utilizam little-endian (LE), compatível
//! com a arquitetura ESP32 (Xtensa, little-endian).

use crate::protocol::types::*;
use crate::protocol::crc8::calc_crc8;

/// Erros possíveis nas operações de codec.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProtocolError {
    /// Tamanho do buffer insuficiente para a operação.
    BufferTooSmall,
    /// Número máximo de campos TLV excedido.
    TooManyFields,
    /// Byte de início inválido na mensagem.
    InvalidStartByte,
    /// ID da mensagem inválido.
    InvalidMsgId,
    /// Tamanho do campo TLV inválido.
    InvalidTlvLength,
    /// Checksum CRC8 inválido.
    InvalidChecksum,
    /// Dados de entrada corrompidos ou incompletos.
    CorruptedData,
    /// Campo TLV com tamanho maior que o permitido.
    TlvDataTooLong,
}

// ============================================================================
// FUNÇÕES DE SERIALIZAÇÃO
// ============================================================================

/// Serializa um campo TLV individual em bytes.
///
/// Formato resultante: `[ID][LEN][DATA...]`
///
/// # Arguments
///
/// * `id` - Identificador do campo TLV.
/// * `data` - Dados do campo.
/// * `output` - Buffer de saída.
///
/// # Returns
///
/// `Ok(bytes_written)` em sucesso, `Err(ProtocolError)` em caso de erro.
pub fn build_tlv(id: u8, data: &[u8], output: &mut [u8]) -> Result<usize, ProtocolError> {
    let data_len = data.len().min(MAX_TLV_DATA);
    let required_size = TLV_HEADER_SIZE + data_len;

    if output.len() < required_size {
        return Err(ProtocolError::BufferTooSmall);
    }

    output[0] = id;
    output[1] = data_len as u8;
    output[TLV_HEADER_SIZE..TLV_HEADER_SIZE + data_len]
        .copy_from_slice(&data[..data_len]);

    Ok(required_size)
}

/// Serializa um campo TLV de vídeo em bytes.
///
/// Utiliza `MAX_TLV_VIDEO_DATA` (128 bytes) em vez de `MAX_TLV_DATA` (32 bytes).
///
/// # Arguments
///
/// * `data` - Dados do campo de vídeo.
/// * `output` - Buffer de saída.
///
/// # Returns
///
/// `Ok(bytes_written)` em sucesso, `Err(ProtocolError)` em caso de erro.
pub fn build_tlv_video(data: &[u8], output: &mut [u8]) -> Result<usize, ProtocolError> {
    let data_len = data.len().min(MAX_TLV_VIDEO_DATA);
    let required_size = TLV_HEADER_SIZE + data_len;

    if output.len() < required_size {
        return Err(ProtocolError::BufferTooSmall);
    }

    output[0] = FieldVideo::Payload as u8;
    output[1] = data_len as u8;
    output[TLV_HEADER_SIZE..TLV_HEADER_SIZE + data_len]
        .copy_from_slice(&data[..data_len]);

    Ok(required_size)
}

/// Serializa uma mensagem TLV completa a partir de uma struct TLVMessage.
///
/// Formato resultante: `[START_BYTE][MSGID][TLV_COUNT][TLV_FIELDS...][CRC8]`
///
/// # Arguments
///
/// * `msg` - Mensagem TLV a serializar.
/// * `msg_id` - Identificador do tipo de mensagem (sobrepõe `msg.msg_id`).
/// * `buffer` - Buffer de saída.
///
/// # Returns
///
/// `Ok(bytes_written)` em sucesso, `Err(ProtocolError)` em caso de erro.
pub fn build_message(
    msg: &TLVMessage,
    msg_id: u8,
    buffer: &mut [u8],
) -> Result<usize, ProtocolError> {
    let required_size = MESSAGE_HEADER_SIZE
        + (msg.tlv_count as usize) * (TLV_HEADER_SIZE + MAX_TLV_DATA)
        + CHECKSUM_SIZE;

    if buffer.len() < required_size {
        return Err(ProtocolError::BufferTooSmall);
    }

    let mut offset = 0;

    // Cabeçalho da mensagem
    buffer[offset] = START_BYTE;
    offset += 1;
    buffer[offset] = msg_id;
    offset += 1;
    buffer[offset] = msg.tlv_count;
    offset += 1;

    // Campos TLV
    for i in 0..msg.tlv_count as usize {
        let tlv = &msg.tlvs[i];
        buffer[offset] = tlv.id;
        offset += 1;
        buffer[offset] = tlv.len;
        offset += 1;
        let len = tlv.len as usize;
        buffer[offset..offset + len].copy_from_slice(&tlv.data[..len]);
        offset += len;
    }

    // CRC8
    let crc = calc_crc8(&buffer[..offset]);
    buffer[offset] = crc;
    offset += 1;

    Ok(offset)
}

// ============================================================================
// FUNÇÕES DE VALIDAÇÃO
// ============================================================================

/// Valida a integridade e estrutura de uma mensagem TLV serializada.
///
/// Verifica:
/// 1. Byte de início (START_BYTE)
/// 2. ID da mensagem válido
/// 3. Número de campos TLV dentro do limite
/// 4. Checksum CRC8
/// 5. IDs dos campos TLV válidos
/// 6. Tamanhos dos campos TLV dentro dos limites
///
/// # Arguments
///
/// * `buffer` - Buffer contendo a mensagem serializada.
///
/// # Returns
///
/// `Ok(tlv_count)` em sucesso (número de campos TLV válidos),
/// `Err(ProtocolError)` em caso de erro.
pub fn validate_message(buffer: &[u8]) -> Result<u8, ProtocolError> {
    // Verificar tamanho mínimo
    if buffer.len() < MESSAGE_HEADER_SIZE + CHECKSUM_SIZE {
        return Err(ProtocolError::CorruptedData);
    }

    // Verificar byte de início
    if buffer[0] != START_BYTE {
        return Err(ProtocolError::InvalidStartByte);
    }

    // Verificar ID da mensagem
    let msg_id = buffer[1];
    if !MsgId::is_valid(msg_id) {
        return Err(ProtocolError::InvalidMsgId);
    }

    // Verificar número de campos TLV
    let tlv_count = buffer[2];
    if tlv_count as usize > MAX_TLV_FIELDS {
        return Err(ProtocolError::InvalidTlvLength);
    }

    // Verificar CRC8
    let total_size = MESSAGE_HEADER_SIZE
        + (tlv_count as usize) * (TLV_HEADER_SIZE + MAX_TLV_DATA)
        + CHECKSUM_SIZE;

    if buffer.len() < total_size {
        return Err(ProtocolError::CorruptedData);
    }

    let crc_offset = total_size - 1;
    let expected_crc = calc_crc8(&buffer[..crc_offset]);
    if buffer[crc_offset] != expected_crc {
        return Err(ProtocolError::InvalidChecksum);
    }

    // Validar campos TLV
    let mut offset = MESSAGE_HEADER_SIZE;
    for _ in 0..tlv_count {
        if offset + TLV_HEADER_SIZE > buffer.len() {
            return Err(ProtocolError::CorruptedData);
        }

        let field_id = buffer[offset];
        let field_len = buffer[offset + 1] as usize;

        if !FieldId::is_valid(field_id) {
            return Err(ProtocolError::InvalidTlvLength);
        }

        if field_len > MAX_TLV_DATA {
            return Err(ProtocolError::TlvDataTooLong);
        }

        offset += TLV_HEADER_SIZE + field_len;
    }

    Ok(tlv_count)
}

// ============================================================================
// FUNÇÕES DE PARSING
// ============================================================================

/// Deserializa campos TLV de um buffer de bytes brutos.
///
/// Assume que o buffer contém apenas campos TLV serializados (sem cabeçalho
/// de mensagem). Cada campo tem o formato: `[ID][LEN][DATA...]`.
///
/// # Arguments
///
/// * `data` - Buffer com campos TLV serializados.
/// * `output` - Array de saída para os campos TLV deserializados.
///
/// # Returns
///
/// `Ok(count)` com o número de campos TLV deserializados,
/// `Err(ProtocolError)` em caso de erro.
pub fn parse_tlv(data: &[u8], output: &mut [TLVField]) -> Result<usize, ProtocolError> {
    let mut offset = 0;
    let mut count = 0;

    while offset + TLV_HEADER_SIZE <= data.len() && count < output.len() {
        let id = data[offset];
        let len = data[offset + 1] as usize;

        if len > MAX_TLV_DATA {
            return Err(ProtocolError::TlvDataTooLong);
        }

        if offset + TLV_HEADER_SIZE + len > data.len() {
            return Err(ProtocolError::CorruptedData);
        }

        output[count] = TLVField {
            id,
            len: len as u8,
            data: {
                let mut arr = [0u8; MAX_TLV_DATA];
                arr[..len].copy_from_slice(&data[offset + TLV_HEADER_SIZE..offset + TLV_HEADER_SIZE + len]);
                arr
            },
        };

        offset += TLV_HEADER_SIZE + len;
        count += 1;
    }

    Ok(count)
}

/// Imprime informações de debug sobre um campo TLV.
///
/// Útil para depuração. Em produção, pode ser desativado.
pub fn print_tlv_field(field: &TLVField) {
    #[cfg(feature = "std")]
    {
        print!("[TLV] ID=0x{:02X} LEN={}", field.id, field.len);
        if field.len > 0 && (field.len as usize) <= field.data.len() {
            print!(" DATA=[");
            for i in 0..field.len as usize {
                if i > 0 {
                    print!(" ");
                }
                print!("{:02X}", field.data[i]);
            }
            print!("]");
        }
        println!();
    }
}

/// Imprime informações de debug sobre uma mensagem TLV.
///
/// Útil para depuração. Em produção, pode ser desativado.
pub fn print_message(msg: &TLVMessage) {
    #[cfg(feature = "std")]
    {
        println!("[MSG] START=0x{:02X} MSGID=0x{:02X} TLV_COUNT={} CRC=0x{:02X}",
            msg.start_byte, msg.msg_id, msg.tlv_count, msg.checksum);
        for i in 0..msg.tlv_count as usize {
            print_tlv_field(&msg.tlvs[i]);
        }
    }
}

// ============================================================================
// TESTES UNITÁRIOS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_tlv() {
        let data = [0x01, 0x02, 0x03];
        let mut output = [0u8; 100];
        let size = build_tlv(0x70, &data, &mut output).unwrap();

        assert_eq!(output[0], 0x70);  // id
        assert_eq!(output[1], 3);     // len
        assert_eq!(output[2], 0x01);  // data[0]
        assert_eq!(output[3], 0x02);  // data[1]
        assert_eq!(output[4], 0x03);  // data[2]
        assert_eq!(size, 5);          // TLV_HEADER_SIZE + 3
    }

    #[test]
    fn test_build_tlv_video() {
        let data = [0xAA, 0xBB, 0xCC];
        let mut output = [0u8; 200];
        let size = build_tlv_video(&data, &mut output).unwrap();

        assert_eq!(output[0], FieldVideo::Payload as u8);  // id
        assert_eq!(output[1], 3);                            // len
        assert_eq!(size, 5);
    }

    #[test]
    fn test_build_message() {
        let mut msg = TLVMessage::with_id(0x11);
        msg.tlv_count = 1;
        msg.tlvs[0] = TLVField::with_data(0x70, &[0x02]);

        let mut buffer = [0u8; 1093];
        let size = build_message(&msg, 0x11, &mut buffer).unwrap();

        assert_eq!(buffer[0], START_BYTE);
        assert_eq!(buffer[1], 0x11);
        assert_eq!(buffer[2], 1);
        assert!(size > 0);
    }

    #[test]
    fn test_validate_message_ok() {
        let mut msg = TLVMessage::with_id(0x11);
        msg.tlv_count = 1;
        msg.tlvs[0] = TLVField::with_data(0x70, &[0x02]);

        let mut buffer = [0u8; 1093];
        let size = build_message(&msg, 0x11, &mut buffer).unwrap();

        let result = validate_message(&buffer[..size]);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 1);
    }

    #[test]
    fn test_validate_message_invalid_start() {
        let mut buffer = [0u8; 1093];
        buffer[0] = 0x00;  // Byte de início inválido
        buffer[1] = 0x11;
        buffer[2] = 0;

        let result = validate_message(&buffer);
        assert_eq!(result, Err(ProtocolError::InvalidStartByte));
    }

    #[test]
    fn test_validate_message_invalid_msg_id() {
        let mut buffer = [0u8; 1093];
        buffer[0] = START_BYTE;
        buffer[1] = 0xFF;  // ID inválido
        buffer[2] = 0;

        let result = validate_message(&buffer);
        assert_eq!(result, Err(ProtocolError::InvalidMsgId));
    }

    #[test]
    fn test_validate_message_invalid_crc() {
        let mut msg = TLVMessage::with_id(0x11);
        msg.tlv_count = 0;

        let mut buffer = [0u8; 1093];
        let size = build_message(&msg, 0x11, &mut buffer).unwrap();

        // Corromper CRC
        buffer[size - 1] = buffer[size - 1].wrapping_add(1);

        let result = validate_message(&buffer[..size]);
        assert_eq!(result, Err(ProtocolError::InvalidChecksum));
    }

    #[test]
    fn test_parse_tlv() {
        let mut tlv_buf = [0u8; 100];
        let mut offset = 0;
        offset += build_tlv(0x70, &[0x02], &mut tlv_buf[offset..]).unwrap();
        offset += build_tlv(0x30, &float_to_bytes(1.5), &mut tlv_buf[offset..]).unwrap();

        let mut output = [TLVField::new(); 10];
        let count = parse_tlv(&tlv_buf[..offset], &mut output).unwrap();

        assert_eq!(count, 2);
        assert_eq!(output[0].id, 0x70);
        assert_eq!(output[0].len, 1);
        assert_eq!(output[1].id, 0x30);
        assert_eq!(output[1].len, 4);
    }

    #[test]
    fn test_parse_tlv_empty() {
        let mut output = [TLVField::new(); 10];
        let count = parse_tlv(&[], &mut output).unwrap();
        assert_eq!(count, 0);
    }

    #[test]
    fn test_roundtrip_build_validate() {
        // Construir mensagem, serializar, validar, parses
        let mut builder = crate::protocol::builder::TLVBuilder::new();
        builder.add_uint8(0x70, 2).unwrap();
        builder.add_float(0x30, 3.14).unwrap();
        builder.add_uint16(0xB0, 42).unwrap();

        let mut buffer = [0u8; 1093];
        let size = builder.build(0x11, &mut buffer).unwrap();

        // Validar
        let tlv_count = validate_message(&buffer[..size]).unwrap();
        assert_eq!(tlv_count, 3);
    }
}
