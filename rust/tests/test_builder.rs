/**
 * @file test_builder.rs
 * @brief Testes de integração do módulo Builder do Protocolo TLV.
 */

use visor_protocol::protocol::builder::*;
use visor_protocol::protocol::types::*;

#[test]
fn test_builder_new() {
    let builder = TLVBuilder::new();
    assert_eq!(builder.get_tlv_count(), 0);
}

#[test]
fn test_builder_add_float() {
    let mut builder = TLVBuilder::new();
    let result = builder.add_float(0x30, 37.7749);
    assert!(result.is_ok());
    assert_eq!(builder.get_tlv_count(), 1);
}

#[test]
fn test_builder_add_int32() {
    let mut builder = TLVBuilder::new();
    let result = builder.add_int32(0x72, -12345);
    assert!(result.is_ok());
    assert_eq!(builder.get_tlv_count(), 1);
}

#[test]
fn test_builder_add_uint32() {
    let mut builder = TLVBuilder::new();
    let result = builder.add_uint32(0x73, 0xDEADBEEF);
    assert!(result.is_ok());
    assert_eq!(builder.get_tlv_count(), 1);
}

#[test]
fn test_builder_add_uint16() {
    let mut builder = TLVBuilder::new();
    let result = builder.add_uint16(0xB0, 42);
    assert!(result.is_ok());
    assert_eq!(builder.get_tlv_count(), 1);
}

#[test]
fn test_builder_add_uint8() {
    let mut builder = TLVBuilder::new();
    let result = builder.add_uint8(0xB1, 5);
    assert!(result.is_ok());
    assert_eq!(builder.get_tlv_count(), 1);
}

#[test]
fn test_builder_add_raw() {
    let mut builder = TLVBuilder::new();
    let data = [0x01, 0x02, 0x03, 0x04, 0x05];
    let result = builder.add_raw(0xB3, &data);
    assert!(result.is_ok());
    assert_eq!(builder.get_tlv_count(), 1);
}

#[test]
fn test_builder_multiple_fields() {
    let mut builder = TLVBuilder::new();
    builder.add_float(0x30, 37.7749).unwrap();
    builder.add_float(0x31, -122.4194).unwrap();
    builder.add_uint32(0x73, 0x12345678).unwrap();
    assert_eq!(builder.get_tlv_count(), 3);
}

#[test]
fn test_builder_build() {
    let mut builder = TLVBuilder::new();
    builder.add_uint8(0x70, 2).unwrap();
    builder.add_float(0x30, 1.5).unwrap();

    let mut buffer = [0u8; 1093];
    let result = builder.build(0x11, &mut buffer);
    assert!(result.is_ok());

    let size = result.unwrap();
    assert_eq!(buffer[0], START_BYTE);
    assert_eq!(buffer[1], 0x11);
    assert_eq!(buffer[2], 2);
    assert!(size > MESSAGE_HEADER_SIZE + CHECKSUM_SIZE);
}

#[test]
fn test_builder_build_buffer_too_small() {
    let mut builder = TLVBuilder::new();
    builder.add_uint8(0x70, 2).unwrap();

    let mut buffer = [0u8; 5];
    let result = builder.build(0x11, &mut buffer);
    assert!(result.is_err());
}

#[test]
fn test_builder_overflow() {
    let mut builder = TLVBuilder::new();
    for i in 0..MAX_TLV_FIELDS {
        builder.add_uint8(i as u8, 0).unwrap();
    }
    let result = builder.add_uint8(0xFF, 0);
    assert!(result.is_err());
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
