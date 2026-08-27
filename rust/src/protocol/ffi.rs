//! # FFI Protocol — Funções extern "C" para Interoperação C/C++ (ACP v3.0.0)

use crate::protocol::types::*;
use crate::protocol::crc16::calc_crc16;
use crate::protocol::codec::*;

// ============================================================================
// CONSTANTES FFI
// ============================================================================

/// Versão do protocolo ACP (string C).
pub const ACP_VERSION_STR: &[u8] = b"3.0.0\0";

// ============================================================================
// FUNÇÕES FFI — CRC16
// ============================================================================

/// Calcula CRC-16/CCITT de um array de bytes.
///
/// # Arguments
/// * `data` — Ponteiro para os dados.
/// * `len` — Número de bytes.
///
/// # Returns
/// Valor CRC-16 (0x0000-0xFFFF).
#[no_mangle]
pub extern "C" fn visor_calc_crc16(data: *const u8, len: usize) -> u16 {
    if data.is_null() || len == 0 {
        return 0xFFFF;
    }
    let slice = unsafe { core::slice::from_raw_parts(data, len) };
    calc_crc16(slice)
}

// ============================================================================
// FUNÇÕES FFI — CRC8 (legado, mantido por retrocompatibilidade)
// ============================================================================

/// Calcula CRC-8/SMBUS de um array de bytes (legado).
///
/// # Arguments
/// * `data` — Ponteiro para os dados.
/// * `len` — Número de bytes.
///
/// # Returns
/// Valor CRC-8 (0x00-0xFF).
#[no_mangle]
pub extern "C" fn visor_calc_crc8(data: *const u8, len: usize) -> u8 {
    if data.is_null() || len == 0 {
        return 0x00;
    }
    let slice = unsafe { core::slice::from_raw_parts(data, len) };
    crate::protocol::crc8::calc_crc8(slice)
}

// ============================================================================
// FUNÇÕES FFI — ASSINATURA
// ============================================================================

/// Calcula a assinatura de uma mensagem ACP.
///
/// signature = XOR(key, msg_id, seq_lo, seq_hi)
///
/// # Arguments
/// * `key` — Chave partilhada.
/// * `msg_id` — ID da mensagem.
/// * `seq_lo` — Byte baixo do SEQ_NUM.
/// * `seq_hi` — Byte alto do SEQ_NUM.
///
/// # Returns
/// Byte de assinatura (0x00-0xFF).
#[no_mangle]
pub extern "C" fn visor_compute_signature(key: u8, msg_id: u8, seq_lo: u8, seq_hi: u8) -> u8 {
    compute_signature(key, msg_id, seq_lo, seq_hi)
}

/// Valida a assinatura de uma mensagem ACP.
///
/// # Arguments
/// * `signature` — Assinatura recebida.
/// * `key` — Chave partilhada.
/// * `msg_id` — ID da mensagem.
/// * `seq_lo` — Byte baixo do SEQ_NUM.
/// * `seq_hi` — Byte alto do SEQ_NUM.
///
/// # Returns
/// 1 se válida, 0 caso contrário.
#[no_mangle]
pub extern "C" fn visor_validate_signature(
    signature: u8, key: u8, msg_id: u8, seq_lo: u8, seq_hi: u8
) -> u8 {
    if validate_signature(signature, key, msg_id, seq_lo, seq_hi) { 1 } else { 0 }
}

// ============================================================================
// FUNÇÕES FFI — FIELDID COM TIPO
// ============================================================================

/// Codifica um FieldID com tipo embutido.
///
/// # Arguments
/// * `field_type` — Tipo de dado (0-7).
/// * `field_id` — ID do campo (0-31).
///
/// # Returns
/// FieldID codificado: [TYPE:3][ID:5]
#[no_mangle]
pub extern "C" fn visor_field_id_encode(field_type: u8, field_id: u8) -> u8 {
    field_id_encode(field_type, field_id)
}

/// Decodifica um FieldID nos seus componentes.
///
/// # Arguments
/// * `field_id` — FieldID codificado.
/// * `type_out` — Ponteiro para o tipo de dado (output).
/// * `id_out` — Ponteiro para o ID do campo (output).
#[no_mangle]
pub extern "C" fn visor_field_id_decode(field_id: u8, type_out: *mut u8, id_out: *mut u8) {
    if type_out.is_null() || id_out.is_null() {
        return;
    }
    let (t, id) = field_id_decode(field_id);
    unsafe {
        *type_out = t;
        *id_out = id;
    }
}

/// Valida se um FieldID codificado tem um tipo válido.
///
/// # Arguments
/// * `field_id` — FieldID codificado.
///
/// # Returns
/// 1 se válido, 0 caso contrário.
#[no_mangle]
pub extern "C" fn visor_is_valid_field_id(field_id: u8) -> u8 {
    if is_valid_field_id(field_id) { 1 } else { 0 }
}

// ============================================================================
// FUNÇÕES FFI — CAN ID
// ============================================================================

/// Constrói um CAN ID extended (29-bit).
///
/// # Arguments
/// * `priority` — Nível de prioridade (0-4).
/// * `src_group` — Grupo de origem (0x0-0xF).
/// * `dst_group` — Grupo de destino (0x0=broadcast).
/// * `msg_type` — Tipo de mensagem (0x0-0x7).
///
/// # Returns
/// CAN ID de 29 bits.
#[no_mangle]
pub extern "C" fn visor_make_can_id(priority: u8, src_group: u8, dst_group: u8, msg_type: u8) -> u32 {
    make_can_id(priority, src_group, dst_group, msg_type)
}

/// Extrai a prioridade de um CAN ID extended.
#[no_mangle]
pub extern "C" fn visor_can_id_priority(can_id: u32) -> u8 {
    can_id_priority(can_id)
}

/// Extrai o grupo de origem de um CAN ID extended.
#[no_mangle]
pub extern "C" fn visor_can_id_src_group(can_id: u32) -> u8 {
    can_id_src_group(can_id)
}

/// Extrai o grupo de destino de um CAN ID extended.
#[no_mangle]
pub extern "C" fn visor_can_id_dst_group(can_id: u32) -> u8 {
    can_id_dst_group(can_id)
}

/// Extrai o tipo de mensagem de um CAN ID extended.
#[no_mangle]
pub extern "C" fn visor_can_id_msg_type(can_id: u32) -> u8 {
    can_id_msg_type(can_id)
}

/// Verifica se o CAN ID pertence ao bus de segurança.
#[no_mangle]
pub extern "C" fn visor_is_safety_bus_id(can_id: u32) -> u8 {
    if is_safety_bus_id(can_id) { 1 } else { 0 }
}

// ============================================================================
// FUNÇÕES FFI — SERIALIZAÇÃO
// ============================================================================

/// Serializa uma mensagem ACP completa num buffer de saída.
///
/// # Arguments
/// * `msg` — Ponteiro para a mensagem ACP a serializar.
/// * `msg_id` — Identificador do tipo de mensagem.
/// * `signature_key` — Chave de assinatura.
/// * `buffer` — Ponteiro para o buffer de saída.
/// * `buffer_size` — Tamanho do buffer de saída em bytes.
///
/// # Returns
/// Número de bytes escritos em sucesso, -1 em erro.
#[no_mangle]
pub extern "C" fn visor_build_message(
    msg: *const TLVMessage,
    msg_id: u8,
    signature_key: u8,
    buffer: *mut u8,
    buffer_size: usize,
) -> isize {
    if msg.is_null() || buffer.is_null() {
        return -1;
    }
    let msg = unsafe { &*msg };
    let buf = unsafe { core::slice::from_raw_parts_mut(buffer, buffer_size) };
    match build_message(msg, msg_id, signature_key, buf) {
        Ok(size) => size as isize,
        Err(_) => -1,
    }
}

/// Valida a integridade e estrutura de uma mensagem ACP serializada.
///
/// # Arguments
/// * `buffer` — Ponteiro para a mensagem serializada.
/// * `length` — Tamanho da mensagem em bytes.
///
/// # Returns
/// Número de campos TLV válidos em sucesso, 0xFF em erro.
#[no_mangle]
pub extern "C" fn visor_validate_message(buffer: *const u8, length: usize) -> u8 {
    if buffer.is_null() || length == 0 {
        return 0xFF;
    }
    let slice = unsafe { core::slice::from_raw_parts(buffer, length) };
    match validate_message(slice) {
        Ok(count) => count,
        Err(_) => 0xFF,
    }
}

/// Valida a assinatura de uma mensagem ACP serializada.
///
/// # Arguments
/// * `buffer` — Ponteiro para a mensagem serializada.
/// * `length` — Tamanho da mensagem em bytes.
/// * `signature_key` — Chave de assinatura esperada.
///
/// # Returns
/// 1 se válida, 0 caso contrário.
#[no_mangle]
pub extern "C" fn visor_validate_signature_in_message(
    buffer: *const u8, length: usize, signature_key: u8
) -> u8 {
    if buffer.is_null() || length == 0 {
        return 0;
    }
    let slice = unsafe { core::slice::from_raw_parts(buffer, length) };
    match validate_signature_in_message(slice, signature_key) {
        Ok(()) => 1,
        Err(_) => 0,
    }
}

/// Deserializa campos TLV de um buffer de bytes brutos.
///
/// # Arguments
/// * `data` — Ponteiro para os bytes dos campos TLV.
/// * `length` — Tamanho dos dados em bytes.
/// * `output` — Ponteiro para o array de saída de TLVField.
/// * `count` — Ponteiro para o número de campos de saída (input: capacidade, output: real).
#[no_mangle]
pub extern "C" fn visor_parse_tlv(
    data: *const u8,
    length: usize,
    output: *mut TLVField,
    count: *mut usize,
) {
    if data.is_null() || output.is_null() || count.is_null() {
        return;
    }
    let slice = unsafe { core::slice::from_raw_parts(data, length) };
    let capacity = unsafe { *count };
    let out_slice = unsafe { core::slice::from_raw_parts_mut(output, capacity) };
    match parse_tlv(slice, out_slice) {
        Ok(parsed) => unsafe { *count = parsed },
        Err(_) => unsafe { *count = 0 },
    }
}

// ============================================================================
// FUNÇÕES FFI — ADICIONAR CAMPOS TLV
// ============================================================================

/// Adiciona um campo TLV com dados brutos a uma mensagem.
///
/// # Arguments
/// * `msg` — Ponteiro para a mensagem ACP.
/// * `id` — FieldID codificado.
/// * `data` — Ponteiro para os dados do campo.
/// * `len` — Número de bytes de dados.
#[no_mangle]
pub extern "C" fn visor_add_tlv(msg: *mut TLVMessage, id: u8, data: *const u8, len: u8) {
    if msg.is_null() || data.is_null() {
        return;
    }
    let msg = unsafe { &mut *msg };
    let data_slice = unsafe { core::slice::from_raw_parts(data, len as usize) };

    if (msg.tlv_count as usize) < MAX_TLV_FIELDS {
        let field = TLVField::with_data(id, data_slice);
        msg.tlvs[msg.tlv_count as usize] = field;
        msg.tlv_count += 1;
    }
}

/// Adiciona um campo TLV com valor float (f32).
#[no_mangle]
pub extern "C" fn visor_add_tlv_float(msg: *mut TLVMessage, id: u8, value: f32) {
    let bytes = float_to_bytes(value);
    visor_add_tlv(msg, id, bytes.as_ptr(), 4);
}

/// Adiciona um campo TLV com valor i32.
#[no_mangle]
pub extern "C" fn visor_add_tlv_int32(msg: *mut TLVMessage, id: u8, value: i32) {
    let bytes = int32_to_bytes(value);
    visor_add_tlv(msg, id, bytes.as_ptr(), 4);
}

/// Adiciona um campo TLV com valor u32.
#[no_mangle]
pub extern "C" fn visor_add_tlv_uint32(msg: *mut TLVMessage, id: u8, value: u32) {
    let bytes = uint32_to_bytes(value);
    visor_add_tlv(msg, id, bytes.as_ptr(), 4);
}

/// Adiciona um campo TLV com valor u16.
#[no_mangle]
pub extern "C" fn visor_add_tlv_uint16(msg: *mut TLVMessage, id: u8, value: u16) {
    let bytes = uint16_to_bytes(value);
    visor_add_tlv(msg, id, bytes.as_ptr(), 2);
}

/// Adiciona um campo TLV com valor u8.
#[no_mangle]
pub extern "C" fn visor_add_tlv_uint8(msg: *mut TLVMessage, id: u8, value: u8) {
    visor_add_tlv(msg, id, &value as *const u8, 1);
}

// ============================================================================
// FUNÇÕES FFI — INICIALIZAÇÃO DE MENSAGEM
// ============================================================================

/// Inicializa uma mensagem ACP com node_id e campos padrão.
///
/// # Arguments
/// * `msg` — Ponteiro para a mensagem a inicializar.
/// * `node_id` — ID do grupo CAN deste nó.
/// * `msg_id` — ID da mensagem.
#[no_mangle]
pub extern "C" fn visor_acp_init(msg: *mut TLVMessage, node_id: u8, msg_id: u8) {
    if msg.is_null() {
        return;
    }
    let msg = unsafe { &mut *msg };
    msg.start_byte = START_BYTE;
    msg.version = ACP_VERSION;
    msg.node_id = node_id;
    msg.msg_id = msg_id;
    msg.seq_num = 0;
    msg.tlv_count = 0;
    msg.signature = 0;
    msg.checksum = 0;
}

/// Define o número de sequência de uma mensagem.
#[no_mangle]
pub extern "C" fn visor_acp_set_seq(msg: *mut TLVMessage, seq: u16) {
    if msg.is_null() {
        return;
    }
    let msg = unsafe { &mut *msg };
    msg.seq_num = seq;
}

/// Limpa uma mensagem ACP (reseta todos os campos).
#[no_mangle]
pub extern "C" fn visor_acp_clear(msg: *mut TLVMessage) {
    if msg.is_null() {
        return;
    }
    let msg = unsafe { &mut *msg };
    msg.clear();
}

// ============================================================================
// FUNÇÕES FFI — CONVERSÃO DE BYTES
// ============================================================================

/// Converte float para bytes (little-endian).
#[no_mangle]
pub extern "C" fn visor_float_to_bytes(value: f32, bytes: *mut u8) {
    if bytes.is_null() {
        return;
    }
    let result = float_to_bytes(value);
    unsafe {
        core::ptr::copy_nonoverlapping(result.as_ptr(), bytes, 4);
    }
}

/// Converte bytes (little-endian) para float.
#[no_mangle]
pub extern "C" fn visor_bytes_to_float(bytes: *const u8) -> f32 {
    if bytes.is_null() {
        return 0.0;
    }
    let mut arr = [0u8; 4];
    unsafe {
        core::ptr::copy_nonoverlapping(bytes, arr.as_mut_ptr(), 4);
    }
    bytes_to_float(&arr)
}

/// Converte i32 para bytes (little-endian).
#[no_mangle]
pub extern "C" fn visor_int32_to_bytes(value: i32, bytes: *mut u8) {
    if bytes.is_null() {
        return;
    }
    let result = int32_to_bytes(value);
    unsafe {
        core::ptr::copy_nonoverlapping(result.as_ptr(), bytes, 4);
    }
}

/// Converte bytes (little-endian) para i32.
#[no_mangle]
pub extern "C" fn visor_bytes_to_int32(bytes: *const u8) -> i32 {
    if bytes.is_null() {
        return 0;
    }
    let mut arr = [0u8; 4];
    unsafe {
        core::ptr::copy_nonoverlapping(bytes, arr.as_mut_ptr(), 4);
    }
    bytes_to_int32(&arr)
}

/// Converte u32 para bytes (little-endian).
#[no_mangle]
pub extern "C" fn visor_uint32_to_bytes(value: u32, bytes: *mut u8) {
    if bytes.is_null() {
        return;
    }
    let result = uint32_to_bytes(value);
    unsafe {
        core::ptr::copy_nonoverlapping(result.as_ptr(), bytes, 4);
    }
}

/// Converte bytes (little-endian) para u32.
#[no_mangle]
pub extern "C" fn visor_bytes_to_uint32(bytes: *const u8) -> u32 {
    if bytes.is_null() {
        return 0;
    }
    let mut arr = [0u8; 4];
    unsafe {
        core::ptr::copy_nonoverlapping(bytes, arr.as_mut_ptr(), 4);
    }
    bytes_to_uint32(&arr)
}

/// Converte u16 para bytes (little-endian).
#[no_mangle]
pub extern "C" fn visor_uint16_to_bytes(value: u16, bytes: *mut u8) {
    if bytes.is_null() {
        return;
    }
    let result = uint16_to_bytes(value);
    unsafe {
        core::ptr::copy_nonoverlapping(result.as_ptr(), bytes, 2);
    }
}

/// Converte bytes (little-endian) para u16.
#[no_mangle]
pub extern "C" fn visor_bytes_to_uint16(bytes: *const u8) -> u16 {
    if bytes.is_null() {
        return 0;
    }
    let mut arr = [0u8; 2];
    unsafe {
        core::ptr::copy_nonoverlapping(bytes, arr.as_mut_ptr(), 2);
    }
    bytes_to_uint16(&arr)
}

// ============================================================================
// FUNÇÕES FFI — VALIDAÇÃO
// ============================================================================

/// Retorna true (1) se o ID da mensagem é válido.
#[no_mangle]
pub extern "C" fn visor_is_valid_msg_id(id: u8) -> u8 {
    if MsgId::is_valid(id) { 1 } else { 0 }
}

/// Retorna a prioridade de uma mensagem.
#[no_mangle]
pub extern "C" fn visor_get_msg_priority(msg_id: u8, failsafe_active: u8) -> u8 {
    get_msg_priority(msg_id, failsafe_active != 0)
}

// ============================================================================
// FUNÇÕES FFI — UTILITÁRIOS
// ============================================================================

/// Retorna a versão do protocolo como string estática.
///
/// # Returns
/// Ponteiro para string estática "3.0.0" (não libertar).
#[no_mangle]
pub extern "C" fn visor_get_version() -> *const core::ffi::c_char {
    ACP_VERSION_STR.as_ptr() as *const core::ffi::c_char
}

/// Retorna o tamanho do overhead ACP (header + signature + crc16).
#[no_mangle]
pub extern "C" fn visor_get_overhead() -> usize {
    ACP_OVERHEAD
}

/// Retorna o tamanho máximo de uma mensagem ACP.
#[no_mangle]
pub extern "C" fn visor_get_max_message_size() -> usize {
    MAX_MESSAGE_SIZE
}
