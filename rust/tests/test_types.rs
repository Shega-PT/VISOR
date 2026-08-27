/**
 * @file test_types.rs
 * @brief Testes de integração do módulo types do Protocolo TLV.
 */

use visor_protocol::protocol::types::*;
use visor_protocol::protocol::codec::*;

#[test]
fn test_start_byte() {
    assert_eq!(START_BYTE, 0xAA);
}

#[test]
fn test_max_values() {
    assert_eq!(MAX_TLV_DATA, 32);
    assert_eq!(MAX_TLV_VIDEO_DATA, 128);
    assert_eq!(MAX_TLV_FIELDS, 32);
    assert_eq!(MAX_MESSAGE_SIZE, 1093);
    assert_eq!(MESSAGE_HEADER_SIZE, 3);
    assert_eq!(CHECKSUM_SIZE, 1);
    assert_eq!(TLV_HEADER_SIZE, 2);
}

#[test]
fn test_msg_id_validity() {
    assert!(MsgId::is_valid(0x10));
    assert!(MsgId::is_valid(0x18));
    assert!(!MsgId::is_valid(0x00));
    assert!(!MsgId::is_valid(0x19));
}

#[test]
fn test_msg_id_from_u8() {
    assert_eq!(MsgId::from_u8(0x10), Some(MsgId::Heartbeat));
    assert_eq!(MsgId::from_u8(0x16), Some(MsgId::Video));
    assert_eq!(MsgId::from_u8(0xFF), None);
}

#[test]
fn test_field_id_validity() {
    assert!(FieldId::is_valid(0x20));
    assert!(FieldId::is_valid(0x30));
    assert!(FieldId::is_valid(0xB3));
    assert!(!FieldId::is_valid(0x00));
    assert!(!FieldId::is_valid(0x2F + 1));
}

#[test]
fn test_field_video_ids() {
    assert_eq!(FieldVideo::FrameId as u8, 0xB0);
    assert_eq!(FieldVideo::ChunkId as u8, 0xB1);
    assert_eq!(FieldVideo::TotalChunks as u8, 0xB2);
    assert_eq!(FieldVideo::Payload as u8, 0xB3);
}

#[test]
fn test_priority_levels() {
    assert_eq!(PriorityLevel::SuperCritical as u8, 0);
    assert_eq!(PriorityLevel::High as u8, 1);
    assert_eq!(PriorityLevel::Normal as u8, 2);
    assert_eq!(PriorityLevel::Low as u8, 3);
    assert_eq!(PriorityLevel::SuperLow as u8, 4);
}

#[test]
fn test_float_roundtrip() {
    let original: f32 = 3.14159;
    let bytes = float_to_bytes(original);
    let recovered = bytes_to_float(&bytes);
    assert_eq!(original, recovered);
}

#[test]
fn test_int32_roundtrip() {
    let original: i32 = -1234567;
    let bytes = int32_to_bytes(original);
    let recovered = bytes_to_int32(&bytes);
    assert_eq!(original, recovered);
}

#[test]
fn test_uint32_roundtrip() {
    let original: u32 = 4294967295;
    let bytes = uint32_to_bytes(original);
    let recovered = bytes_to_uint32(&bytes);
    assert_eq!(original, recovered);
}

#[test]
fn test_uint16_roundtrip() {
    let original: u16 = 65535;
    let bytes = uint16_to_bytes(original);
    let recovered = bytes_to_uint16(&bytes);
    assert_eq!(original, recovered);
}

#[test]
fn test_tlv_field_new() {
    let field = TLVField::new();
    assert_eq!(field.id, 0);
    assert_eq!(field.len, 0);
}

#[test]
fn test_tlv_field_with_data() {
    let data = [1u8, 2, 3, 4];
    let field = TLVField::with_data(0xB0, &data);
    assert_eq!(field.id, 0xB0);
    assert_eq!(field.len, 4);
    assert_eq!(field.data[0], 1);
    assert_eq!(field.data[3], 4);
}

#[test]
fn test_tlv_message_new() {
    let msg = TLVMessage::new();
    assert_eq!(msg.start_byte, START_BYTE);
    assert_eq!(msg.tlv_count, 0);
}

#[test]
fn test_tlv_message_clear() {
    let mut msg = TLVMessage::with_id(0x16);
    msg.tlv_count = 3;
    msg.clear();
    assert_eq!(msg.tlv_count, 0);
    assert_eq!(msg.msg_id, 0);
}

#[test]
fn test_get_msg_priority_normal() {
    assert_eq!(get_msg_priority(0x14, false), PriorityLevel::SuperCritical as u8);
    assert_eq!(get_msg_priority(0x12, false), PriorityLevel::High as u8);
    assert_eq!(get_msg_priority(0x10, false), PriorityLevel::Normal as u8);
    assert_eq!(get_msg_priority(0x16, false), PriorityLevel::Low as u8);
    assert_eq!(get_msg_priority(0x15, false), PriorityLevel::SuperLow as u8);
}

#[test]
fn test_get_msg_priority_failsafe() {
    assert_eq!(get_msg_priority(0x10, true), PriorityLevel::SuperCritical as u8);
    assert_eq!(get_msg_priority(0x15, true), PriorityLevel::SuperLow as u8);
}
