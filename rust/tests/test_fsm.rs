/**
 * @file test_fsm.rs
 * @brief Testes de integração do módulo FSM do Parser.
 */

use visor_protocol::parser::fsm::*;
use visor_protocol::protocol::types::*;
use visor_protocol::protocol::builder::*;
use visor_protocol::protocol::ffi::*;

/* ========================================================================
 * TESTES: Criação/Remoção do Parser
 * ======================================================================== */

#[test]
fn test_parser_new() {
    let parser = unsafe { visor_parser_new() };
    assert!(!parser.is_null());
    assert_eq!(unsafe { visor_parser_get_current_state(parser) }, ParserState::WaitStart as u8);
    assert_eq!(unsafe { visor_parser_has_message(parser) }, 0);
    unsafe { visor_parser_free(parser); }
}

#[test]
fn test_parser_ffi_free_null() {
    unsafe { visor_parser_free(std::ptr::null_mut()); }
}

/* ========================================================================
 * TESTES: Máquina de Estados via FFI
 * ======================================================================== */

#[test]
fn test_feed_start_byte() {
    let parser = unsafe { visor_parser_new() };
    let result = unsafe { visor_parser_feed(parser, START_BYTE) };
    assert_eq!(result, ParserError::Ok as u8);
    assert_eq!(unsafe { visor_parser_get_current_state(parser) }, ParserState::WaitMsgId as u8);
    unsafe { visor_parser_free(parser); }
}

#[test]
fn test_feed_invalid_start_byte() {
    let parser = unsafe { visor_parser_new() };
    let result = unsafe { visor_parser_feed(parser, 0x00) };
    assert_eq!(result, ParserError::ErrStart as u8);
    unsafe { visor_parser_free(parser); }
}

#[test]
fn test_feed_msg_id() {
    let parser = unsafe { visor_parser_new() };
    unsafe { visor_parser_feed(parser, START_BYTE); }
    let result = unsafe { visor_parser_feed(parser, VISOR_MSG_VIDEO) };
    assert_eq!(result, ParserError::Ok as u8);
    assert_eq!(unsafe { visor_parser_get_current_state(parser) }, ParserState::WaitTlvCount as u8);
    unsafe { visor_parser_free(parser); }
}

#[test]
fn test_feed_invalid_msg_id() {
    let parser = unsafe { visor_parser_new() };
    unsafe { visor_parser_feed(parser, START_BYTE); }
    let result = unsafe { visor_parser_feed(parser, 0xFF) };
    assert_eq!(result, ParserError::ErrMsgId as u8);
    unsafe { visor_parser_free(parser); }
}

#[test]
fn test_feed_tlv_count_zero() {
    let parser = unsafe { visor_parser_new() };
    unsafe { visor_parser_feed(parser, START_BYTE); }
    unsafe { visor_parser_feed(parser, VISOR_MSG_HEARTBEAT); }
    let result = unsafe { visor_parser_feed(parser, 0) };
    assert_eq!(result, ParserError::Ok as u8);
    assert_eq!(unsafe { visor_parser_get_current_state(parser) }, ParserState::WaitChecksum as u8);
    unsafe { visor_parser_free(parser); }
}

#[test]
fn test_feed_tlv_count() {
    let parser = unsafe { visor_parser_new() };
    unsafe { visor_parser_feed(parser, START_BYTE); }
    unsafe { visor_parser_feed(parser, VISOR_MSG_VIDEO); }
    let result = unsafe { visor_parser_feed(parser, 2) };
    assert_eq!(result, ParserError::Ok as u8);
    assert_eq!(unsafe { visor_parser_get_current_state(parser) }, ParserState::WaitTlvId as u8);
    unsafe { visor_parser_free(parser); }
}

/* ========================================================================
 * TESTES: Mensagens Completas via FFI
 * ======================================================================== */

fn build_and_feed(msg_id: u8, tlv_data: &[(u8, &[u8])]) -> *mut Parser {
    let mut builder = TLVBuilder::new();
    for (id, data) in tlv_data {
        builder.add_raw(*id, data).unwrap();
    }
    let mut buffer = [0u8; MAX_MESSAGE_SIZE];
    let size = builder.build(msg_id, &mut buffer).unwrap();

    let parser = unsafe { visor_parser_new() };
    for &byte in &buffer[..size] {
        unsafe { visor_parser_feed(parser, byte); }
    }
    parser
}

#[test]
fn test_complete_message_heartbeat() {
    let parser = build_and_feed(VISOR_MSG_HEARTBEAT, &[]);
    assert_eq!(unsafe { visor_parser_has_message(parser) }, 1);

    let mut msg = TLVMessage::new();
    let result = unsafe { visor_parser_copy_message(parser, &mut msg) };
    assert_eq!(result, 1);
    assert_eq!(msg.msg_id, VISOR_MSG_HEARTBEAT);

    unsafe { visor_parser_free(parser); }
}

#[test]
fn test_complete_message_video() {
    let parser = build_and_feed(VISOR_MSG_VIDEO, &[
        (0xB0, &42u16.to_le_bytes()),
        (0xB1, &[0x03]),
        (0xB2, &[0x10]),
    ]);
    assert_eq!(unsafe { visor_parser_has_message(parser) }, 1);

    let mut msg = TLVMessage::new();
    unsafe { visor_parser_copy_message(parser, &mut msg) };
    assert_eq!(msg.tlv_count, 3);
    assert_eq!(msg.tlvs[0].id, 0xB0);
    assert_eq!(msg.tlvs[1].id, 0xB1);
    assert_eq!(msg.tlvs[2].id, 0xB2);

    unsafe { visor_parser_free(parser); }
}

#[test]
fn test_complete_message_telemetry() {
    let parser = build_and_feed(VISOR_MSG_TELEMETRY, &[
        (0x30, &1.5f32.to_le_bytes()),
        (0x70, &[0x02]),
    ]);
    assert_eq!(unsafe { visor_parser_has_message(parser) }, 1);

    let mut msg = TLVMessage::new();
    unsafe { visor_parser_copy_message(parser, &mut msg) };
    assert_eq!(msg.tlv_count, 2);

    unsafe { visor_parser_free(parser); }
}

#[test]
fn test_multiple_messages() {
    let parser1 = build_and_feed(VISOR_MSG_HEARTBEAT, &[]);
    assert_eq!(unsafe { visor_parser_has_message(parser1) }, 1);
    unsafe { visor_parser_acknowledge(parser1); }
    assert_eq!(unsafe { visor_parser_has_message(parser1) }, 0);

    // Segunda mensagem no mesmo parser
    unsafe { visor_parser_free(parser1); }

    let parser = build_and_feed(VISOR_MSG_VIDEO, &[(0xB0, &1u16.to_le_bytes())]);
    assert_eq!(unsafe { visor_parser_has_message(parser) }, 1);
    assert_eq!(unsafe { visor_parser_get_success_count(parser) }, 1);

    unsafe { visor_parser_free(parser); }
}

#[test]
fn test_parser_ffi_acknowledge() {
    let parser = build_and_feed(VISOR_MSG_HEARTBEAT, &[]);
    assert_eq!(unsafe { visor_parser_has_message(parser) }, 1);

    unsafe { visor_parser_acknowledge(parser); }
    assert_eq!(unsafe { visor_parser_has_message(parser) }, 0);

    unsafe { visor_parser_free(parser); }
}

#[test]
fn test_parser_ffi_reset() {
    let parser = unsafe { visor_parser_new() };
    unsafe { visor_parser_feed(parser, START_BYTE); }
    unsafe { visor_parser_feed(parser, VISOR_MSG_VIDEO); }
    assert_ne!(unsafe { visor_parser_get_current_state(parser) }, ParserState::WaitStart as u8);

    unsafe { visor_parser_reset(parser); }
    assert_eq!(unsafe { visor_parser_get_current_state(parser) }, ParserState::WaitStart as u8);
    assert_eq!(unsafe { visor_parser_has_message(parser) }, 0);

    unsafe { visor_parser_free(parser); }
}

#[test]
fn test_parser_ffi_set_max_frame_gap() {
    let parser = unsafe { visor_parser_new() };
    unsafe { visor_parser_set_max_frame_gap(parser, 100); }
    unsafe { visor_parser_free(parser); }
}

#[test]
fn test_parser_ffi_is_timed_out() {
    let parser = unsafe { visor_parser_new() };
    assert_eq!(unsafe { visor_parser_is_timed_out(parser) }, 0);
    unsafe { visor_parser_free(parser); }
}

#[test]
fn test_parser_ffi_state_to_string() {
    let result = unsafe { visor_parser_state_to_string(ParserState::WaitStart as u8) };
    assert!(!result.is_null());
}

#[test]
fn test_parser_ffi_error_to_string() {
    let result = unsafe { visor_parser_error_to_string(ParserError::ErrStart as u8) };
    assert!(!result.is_null());
}

#[test]
fn test_parser_ffi_get_stats() {
    let parser = unsafe { visor_parser_new() };
    assert_eq!(unsafe { visor_parser_get_success_count(parser) }, 0);
    assert_eq!(unsafe { visor_parser_get_error_count(parser) }, 0);
    unsafe { visor_parser_free(parser); }
}

#[test]
fn test_parser_ffi_set_debug() {
    let parser = unsafe { visor_parser_new() };
    unsafe { visor_parser_set_debug(parser, 1); }
    unsafe { visor_parser_free(parser); }
}

#[test]
fn test_parser_invalid_crc() {
    let mut builder = TLVBuilder::new();
    builder.add_uint8(0x70, 0x02).unwrap();
    let mut buffer = [0u8; MAX_MESSAGE_SIZE];
    let size = builder.build(VISOR_MSG_HEARTBEAT, &mut buffer).unwrap();

    // Corromper CRC
    buffer[size - 1] = buffer[size - 1].wrapping_add(1);

    let parser = unsafe { visor_parser_new() };
    for &byte in &buffer[..size] {
        unsafe { visor_parser_feed(parser, byte); }
    }

    assert_eq!(unsafe { visor_parser_has_message(parser) }, 0);
    assert!(unsafe { visor_parser_get_error_count(parser) } > 0);

    unsafe { visor_parser_free(parser); }
}
