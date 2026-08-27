///!

//! # FFI Protocol — Funções extern "C" para Interoperação C/C++
//!
//! Este módulo expõe todas as funções do protocolo TLV como funções
//! `extern "C"` para permitir chamadas a partir de código C/C++.
//!
//! ## Nota de Segurança
//!
//! Todas as funções FFI são `unsafe` e requerem ponteiros válidos.
//! O caller é responsável por garantir que os ponteiros não são nulos
//! e que os buffers têm tamanho adequado.

use core::ffi::c_void;
use crate::protocol::types::*;
use crate::protocol::crc8::calc_crc8;
use crate::protocol::codec::*;

// ============================================================================
// FUNÇÕES FFI — CRC8
// ============================================================================

/// Calcula CRC-8/SMBUS de um array de bytes.
///
/// # Arguments
/// * `data` - Ponteiro para os dados.
/// * `len` - Número de bytes.
///
/// # Returns
/// Valor CRC-8 (0x00-0xFF).
#[no_mangle]
pub extern "C" fn visor_calc_crc8(data: *const u8, len: usize) -> u8 {
    if data.is_null() || len == 0 {
        return 0x00;
    }
    let slice = unsafe { core::slice::from_raw_parts(data, len) };
    calc_crc8(slice)
}

// ============================================================================
// FUNÇÕES FFI — SERIALIZAÇÃO
// ============================================================================

/// Serializa uma mensagem TLV completa num buffer de saída.
///
/// # Arguments
/// * `msg` - Ponteiro para a mensagem TLV a serializar.
/// * `msg_id` - Identificador do tipo de mensagem.
/// * `buffer` - Ponteiro para o buffer de saída.
/// * `buffer_size` - Tamanho do buffer de saída em bytes.
///
/// # Returns
/// Número de bytes escritos em sucesso, -1 em erro.
#[no_mangle]
pub extern "C" fn visor_build_message(
    msg: *const TLVMessage,
    msg_id: u8,
    buffer: *mut u8,
    buffer_size: usize,
) -> isize {
    if msg.is_null() || buffer.is_null() {
        return -1;
    }
    let msg = unsafe { &*msg };
    let buf = unsafe { core::slice::from_raw_parts_mut(buffer, buffer_size) };
    match build_message(msg, msg_id, buf) {
        Ok(size) => size as isize,
        Err(_) => -1,
    }
}

/// Valida a integridade e estrutura de uma mensagem TLV serializada.
///
/// # Arguments
/// * `buffer` - Ponteiro para a mensagem serializada.
/// * `length` - Tamanho da mensagem em bytes.
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

/// Deserializa campos TLV de um buffer de bytes brutos.
///
/// # Arguments
/// * `data` - Ponteiro para os bytes dos campos TLV.
/// * `length` - Tamanho dos dados em bytes.
/// * `output` - Ponteiro para o array de saída de TLVField.
/// * `count` - Ponteiro para o número de campos de saída (input: capacidade, output: real).
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
/// * `msg` - Ponteiro para a mensagem TLV.
/// * `id` - Identificador do campo.
/// * `data` - Ponteiro para os dados do campo.
/// * `len` - Número de bytes de dados.
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

/// Adiciona um campo TLV com valor float.
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

/// Retorna true (1) se o ID do campo TLV é válido.
#[no_mangle]
pub extern "C" fn visor_is_valid_field_id(id: u8) -> u8 {
    if FieldId::is_valid(id) { 1 } else { 0 }
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
/// Ponteiro para string estática "2.0.0" (não libertar).
#[no_mangle]
pub extern "C" fn visor_get_version() -> *const core::ffi::c_char {
    b"2.0.0\0".as_ptr() as *const core::ffi::c_char
}
