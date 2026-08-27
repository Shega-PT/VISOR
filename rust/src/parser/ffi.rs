///!

//! # FFI Parser — Funções extern "C" para Interoperação C/C++
//!
//! Este módulo expõe as funções do parser FSM como funções `extern "C"`
//! para permitir chamadas a partir de código C/C++.
//!
//! ## Nota de Segurança
//!
//! Todas as funções FFI são `unsafe` e requerem ponteiros válidos.
//! O parser deve ser criado via `visor_parser_new()` e destruído via
//! `visor_parser_free()` para gerir a memória corretamente.

use crate::parser::fsm::*;
use crate::protocol::types::*;

/// Cria um novo parser FSM.
///
/// # Returns
/// Ponteiro para o parser criado, ou NULL em erro.
#[no_mangle]
pub extern "C" fn visor_parser_new() -> *mut Parser {
    let parser = Parser::new();
    Box::into_raw(Box::new(parser))
}

/// Destrói um parser FSM e liberta a memória.
///
/// # Arguments
/// * `parser` - Ponteiro para o parser a destruir.
#[no_mangle]
pub extern "C" fn visor_parser_free(parser: *mut Parser) {
    if !parser.is_null() {
        unsafe {
            let _ = Box::from_raw(parser);
        }
    }
}

/// Alimenta um byte ao parser.
///
/// # Arguments
/// * `parser` - Ponteiro para o parser.
/// * `byte` - Byte a processar.
///
/// # Returns
/// Código de erro (0 = OK).
#[no_mangle]
pub extern "C" fn visor_parser_feed(parser: *mut Parser, byte: u8) -> u8 {
    if parser.is_null() {
        return ParserError::ErrStart as u8;
    }
    let parser = unsafe { &mut *parser };
    parser.feed(byte) as u8
}

/// Retorna true (1) se uma mensagem completa está disponível.
///
/// # Arguments
/// * `parser` - Ponteiro para o parser.
#[no_mangle]
pub extern "C" fn visor_parser_has_message(parser: *const Parser) -> u8 {
    if parser.is_null() {
        return 0;
    }
    let parser = unsafe { &*parser };
    if parser.has_message() { 1 } else { 0 }
}

/// Retorna ponteiro para a mensagem reconstruída.
///
/// # Arguments
/// * `parser` - Ponteiro para o parser.
///
/// # Returns
/// Ponteiro para a mensagem TLV, ou NULL se não houver mensagem.
#[no_mangle]
pub extern "C" fn visor_parser_get_message(parser: *const Parser) -> *const TLVMessage {
    if parser.is_null() {
        return core::ptr::null();
    }
    let parser = unsafe { &*parser };
    if parser.has_message() {
        parser.get_message() as *const TLVMessage
    } else {
        core::ptr::null()
    }
}

/// Copia a mensagem reconstruída para um buffer de saída.
///
/// # Arguments
/// * `parser` - Ponteiro para o parser.
/// * `output` - Ponteiro para o buffer de saída.
///
/// # Returns
/// 1 em sucesso, 0 em erro.
#[no_mangle]
pub extern "C" fn visor_parser_copy_message(
    parser: *const Parser,
    output: *mut TLVMessage,
) -> u8 {
    if parser.is_null() || output.is_null() {
        return 0;
    }
    let parser = unsafe { &*parser };
    let output = unsafe { &mut *output };
    if parser.copy_message(output) { 1 } else { 0 }
}

/// Reconhece a mensagem processada.
///
/// # Arguments
/// * `parser` - Ponteiro para o parser.
#[no_mangle]
pub extern "C" fn visor_parser_acknowledge(parser: *mut Parser) {
    if parser.is_null() {
        return;
    }
    let parser = unsafe { &mut *parser };
    parser.acknowledge();
}

/// Reseta o parser para o estado inicial.
///
/// # Arguments
/// * `parser` - Ponteiro para o parser.
#[no_mangle]
pub extern "C" fn visor_parser_reset(parser: *mut Parser) {
    if parser.is_null() {
        return;
    }
    let parser = unsafe { &mut *parser };
    parser.reset();
}

/// Define o intervalo máximo entre bytes (timeout) em microssegundos.
///
/// # Arguments
/// * `parser` - Ponteiro para o parser.
/// * `micros` - Timeout em microssegundos.
#[no_mangle]
pub extern "C" fn visor_parser_set_max_frame_gap(parser: *mut Parser, micros: u32) {
    if parser.is_null() {
        return;
    }
    let parser = unsafe { &mut *parser };
    parser.set_max_frame_gap(micros);
}

/// Verifica se o parser excedeu o timeout.
///
/// # Arguments
/// * `parser` - Ponteiro para o parser.
///
/// # Returns
/// 1 se em timeout, 0 caso contrário.
#[no_mangle]
pub extern "C" fn visor_parser_is_timed_out(parser: *const Parser) -> u8 {
    if parser.is_null() {
        return 0;
    }
    let parser = unsafe { &*parser };
    if parser.is_timed_out() { 1 } else { 0 }
}

/// Retorna o último erro registado.
///
/// # Arguments
/// * `parser` - Ponteiro para o parser.
///
/// # Returns
/// Código do erro (0 = OK).
#[no_mangle]
pub extern "C" fn visor_parser_get_last_error(parser: *const Parser) -> u8 {
    if parser.is_null() {
        return ParserError::ErrStart as u8;
    }
    let parser = unsafe { &*parser };
    parser.get_last_error() as u8
}

/// Retorna o estado atual da FSM.
///
/// # Arguments
/// * `parser` - Ponteiro para o parser.
///
/// # Returns
/// Código do estado (0-6).
#[no_mangle]
pub extern "C" fn visor_parser_get_current_state(parser: *const Parser) -> u8 {
    if parser.is_null() {
        return ParserState::WaitStart as u8;
    }
    let parser = unsafe { &*parser };
    parser.get_current_state() as u8
}

/// Retorna o número de mensagens processadas com sucesso.
///
/// # Arguments
/// * `parser` - Ponteiro para o parser.
#[no_mangle]
pub extern "C" fn visor_parser_get_success_count(parser: *const Parser) -> u32 {
    if parser.is_null() {
        return 0;
    }
    let parser = unsafe { &*parser };
    parser.get_success_count()
}

/// Retorna o número de erros de parsing.
///
/// # Arguments
/// * `parser` - Ponteiro para o parser.
#[no_mangle]
pub extern "C" fn visor_parser_get_error_count(parser: *const Parser) -> u32 {
    if parser.is_null() {
        return 0;
    }
    let parser = unsafe { &*parser };
    parser.get_error_count()
}

/// Ativa ou desativa a saída de debug.
///
/// # Arguments
/// * `parser` - Ponteiro para o parser.
/// * `enable` - 1 para ativar, 0 para desativar.
#[no_mangle]
pub extern "C" fn visor_parser_set_debug(parser: *mut Parser, enable: u8) {
    if parser.is_null() {
        return;
    }
    let parser = unsafe { &mut *parser };
    parser.set_debug(enable != 0);
}

/// Converte um estado do parser para string legível.
///
/// # Arguments
/// * `state` - Código do estado.
///
/// # Returns
/// Ponteiro para string estática com o nome do estado.
#[no_mangle]
pub extern "C" fn visor_parser_state_to_string(state: u8) -> *const core::ffi::c_char {
    // Nota: retorna ponteiro para string estática — não libertar
    let s = match state {
        0 => "WAIT_START\0",
        1 => "WAIT_MSGID\0",
        2 => "WAIT_TLVCOUNT\0",
        3 => "WAIT_TLV_ID\0",
        4 => "WAIT_TLV_LEN\0",
        5 => "WAIT_TLV_DATA\0",
        6 => "WAIT_CHECKSUM\0",
        _ => "UNKNOWN\0",
    };
    s.as_ptr() as *const core::ffi::c_char
}

/// Converte um código de erro do parser para string legível.
///
/// # Arguments
/// * `error` - Código do erro.
///
/// # Returns
/// Ponteiro para string estática com o nome do erro.
#[no_mangle]
pub extern "C" fn visor_parser_error_to_string(error: u8) -> *const core::ffi::c_char {
    let s = match error {
        0 => "OK\0",
        1 => "ERR_START\0",
        2 => "ERR_MSGID\0",
        3 => "ERR_TLV_COUNT\0",
        4 => "ERR_TLV_ID\0",
        5 => "ERR_TLV_LEN\0",
        6 => "ERR_CHECKSUM\0",
        7 => "ERR_TIMEOUT\0",
        _ => "UNKNOWN\0",
    };
    s.as_ptr() as *const core::ffi::c_char
}
