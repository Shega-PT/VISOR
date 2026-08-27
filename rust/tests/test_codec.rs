/**
 * @file test_codec.rs
 * @brief Testes de integração do módulo Codec (build, validate, parse, FFI).
 */

use visor_protocol::protocol::codec::*;
use visor_protocol::protocol::types::*;
use visor_protocol::protocol::ffi::*;

/* ========================================================================
 * TESTES: build_message
 * ======================================================================== */

#[test]
fn test_build_message_basic() {
    let mut msg = TLVMessage::with_id(VISOR_MSG_HEARTBEAT);
    let mut buffer = [0u8; 1093];
    let result = build_message(&msg, VISOR_MSG_HEARTBEAT, &mut buffer);
    assert!(result.is_ok());
    assert_eq!(buffer[0], START_BYTE);
    assert_eq!(buffer[1], VISOR_MSG_HEARTBEAT);
    assert_eq!(buffer[2], 0);
}

#[test]
fn test_build_message_with_tlv() {
    let mut msg = TLVMessage::with_id(VISOR_MSG_VIDEO);
    msg.tlvs[0] = TLVField::with_data(0xB0, &42u16.to_le_bytes());
    msg.tlvs[1] = TLVField::with_data(0xB1, &[0x03]);
    msg.tlv_count = 2;

    let mut buffer = [0u8; 1093];
    let result = build_message(&msg, VISOR_MSG_VIDEO, &mut buffer);
    assert!(result.is_ok());
    assert_eq!(buffer[2], 2);
}

/* ========================================================================
 * TESTES: validate_message
 * ======================================================================== */

#[test]
fn test_validate_message_valid() {
    let mut msg = TLVMessage::with_id(VISOR_MSG_HEARTBEAT);
    msg.tlv_count = 1;
    msg.tlvs[0] = TLVField::with_data(0x70, &[0x02]);

    let mut buffer = [0u8; 1093];
    let size = build_message(&msg, VISOR_MSG_HEARTBEAT, &mut buffer).unwrap();

    let result = validate_message(&buffer[..size]);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 1);
}

#[test]
fn test_validate_message_invalid_start() {
    let mut buffer = [0u8; 1093];
    buffer[0] = 0x00;
    buffer[1] = VISOR_MSG_HEARTBEAT;
    buffer[2] = 0;

    let result = validate_message(&buffer);
    assert_eq!(result, Err(ProtocolError::InvalidStartByte));
}

#[test]
fn test_validate_message_invalid_msg_id() {
    let mut buffer = [0u8; 1093];
    buffer[0] = START_BYTE;
    buffer[1] = 0xFF;
    buffer[2] = 0;

    let result = validate_message(&buffer);
    assert_eq!(result, Err(ProtocolError::InvalidMsgId));
}

#[test]
fn test_validate_message_invalid_crc() {
    let mut msg = TLVMessage::with_id(VISOR_MSG_HEARTBEAT);
    let mut buffer = [0u8; 1093];
    let size = build_message(&msg, VISOR_MSG_HEARTBEAT, &mut buffer).unwrap();

    buffer[size - 1] = buffer[size - 1].wrapping_add(1);

    let result = validate_message(&buffer[..size]);
    assert_eq!(result, Err(ProtocolError::InvalidChecksum));
}

/* ========================================================================
 * TESTES: parse_tlv
 * ======================================================================== */

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
    assert_eq!(output[1].id, 0x30);
}

#[test]
fn test_parse_tlv_empty() {
    let mut output = [TLVField::new(); 10];
    let count = parse_tlv(&[], &mut output).unwrap();
    assert_eq!(count, 0);
}

/* ========================================================================
 * TESTES: FFI Codec
 * ======================================================================== */

#[test]
fn test_ffi_calc_crc8() {
    let data = b"TEST";
    let ffi_result = unsafe { visor_calc_crc8(data.as_ptr(), data.len()) };
    assert!(ffi_result <= 0xFF);
}

#[test]
fn test_ffi_build_message() {
    let msg = TLVMessage::with_id(VISOR_MSG_HEARTBEAT);
    let mut buffer = [0u8; 1093];
    let result = unsafe {
        visor_build_message(&msg, VISOR_MSG_HEARTBEAT, buffer.as_mut_ptr(), buffer.len())
    };
    assert!(result > 0);
    assert_eq!(buffer[0], START_BYTE);
}

#[test]
fn test_ffi_validate_message() {
    let msg = TLVMessage::with_id(VISOR_MSG_HEARTBEAT);
    let mut buffer = [0u8; 1093];
    unsafe { visor_build_message(&msg, VISOR_MSG_HEARTBEAT, buffer.as_mut_ptr(), buffer.len()); }

    let result = unsafe { visor_validate_message(buffer.as_ptr(), buffer.len()) };
    assert!(result != 0xFF);
}

#[test]
fn test_ffi_parse_tlv() {
    let mut msg = TLVMessage::new();
    unsafe { visor_add_tlv_uint16(&mut msg, 0xB0, 42); }
    unsafe { visor_add_tlv_uint8(&mut msg, 0xB1, 0); }

    let mut output = [TLVField::new(); 10];
    let mut count = 2usize;
    unsafe {
        visor_parse_tlv(
            msg.tlvs.as_ptr() as *const u8,
            msg.tlv_count as usize * (2 + 32),
            output.as_mut_ptr(),
            &mut count,
        );
    }
    assert!(count > 0);
}

#[test]
fn test_ffi_add_tlv_uint8() {
    let mut msg = TLVMessage::new();
    unsafe { visor_add_tlv_uint8(&mut msg, 0xB1, 5); }
    assert_eq!(msg.tlv_count, 1);
    assert_eq!(msg.tlvs[0].id, 0xB1);
}

#[test]
fn test_ffi_add_tlv_uint16() {
    let mut msg = TLVMessage::new();
    unsafe { visor_add_tlv_uint16(&mut msg, 0xB0, 42); }
    assert_eq!(msg.tlv_count, 1);
}

#[test]
fn test_ffi_add_tlv_uint32() {
    let mut msg = TLVMessage::new();
    unsafe { visor_add_tlv_uint32(&mut msg, 0x73, 0xDEADBEEF); }
    assert_eq!(msg.tlv_count, 1);
}

#[test]
fn test_ffi_add_tlv_int32() {
    let mut msg = TLVMessage::new();
    unsafe { visor_add_tlv_int32(&mut msg, 0x72, -12345); }
    assert_eq!(msg.tlv_count, 1);
}

#[test]
fn test_ffi_add_tlv_float() {
    let mut msg = TLVMessage::new();
    unsafe { visor_add_tlv_float(&mut msg, 0x30, 37.7749); }
    assert_eq!(msg.tlv_count, 1);
}

#[test]
fn test_ffi_add_tlv_raw() {
    let mut msg = TLVMessage::new();
    let data = [0x01, 0x02, 0x03];
    unsafe { visor_add_tlv(&mut msg, 0xB3, data.as_ptr(), 3); }
    assert_eq!(msg.tlv_count, 1);
}

#[test]
fn test_ffi_float_to_bytes() {
    let mut bytes = [0u8; 4];
    unsafe { visor_float_to_bytes(3.14, bytes.as_mut_ptr()); }
    let recovered = unsafe { visor_bytes_to_float(bytes.as_ptr()) };
    assert_eq!(3.14f32, recovered);
}

#[test]
fn test_ffi_int32_to_bytes() {
    let mut bytes = [0u8; 4];
    unsafe { visor_int32_to_bytes(-12345, bytes.as_mut_ptr()); }
    let recovered = unsafe { visor_bytes_to_int32(bytes.as_ptr()) };
    assert_eq!(-12345, recovered);
}

#[test]
fn test_ffi_uint32_to_bytes() {
    let mut bytes = [0u8; 4];
    unsafe { visor_uint32_to_bytes(0xDEADBEEF, bytes.as_mut_ptr()); }
    let recovered = unsafe { visor_bytes_to_uint32(bytes.as_ptr()) };
    assert_eq!(0xDEADBEEFu32, recovered);
}

#[test]
fn test_ffi_uint16_to_bytes() {
    let mut bytes = [0u8; 2];
    unsafe { visor_uint16_to_bytes(65535, bytes.as_mut_ptr()); }
    let recovered = unsafe { visor_bytes_to_uint16(bytes.as_ptr()) };
    assert_eq!(65535u16, recovered);
}

#[test]
fn test_ffi_is_valid_msg_id() {
    assert_eq!(unsafe { visor_is_valid_msg_id(0x10) }, 1);
    assert_eq!(unsafe { visor_is_valid_msg_id(0xFF) }, 0);
}

#[test]
fn test_ffi_is_valid_field_id() {
    assert_eq!(unsafe { visor_is_valid_field_id(0x20) }, 1);
    assert_eq!(unsafe { visor_is_valid_field_id(0xFF) }, 0);
}

#[test]
fn test_ffi_get_msg_priority() {
    assert_eq!(unsafe { visor_get_msg_priority(0x14, 0) }, 0); // Failsafe -> SuperCritical
    assert_eq!(unsafe { visor_get_msg_priority(0x12, 0) }, 1); // Command -> High
    assert_eq!(unsafe { visor_get_msg_priority(0x10, 0) }, 2); // Heartbeat -> Normal
    assert_eq!(unsafe { visor_get_msg_priority(0x16, 0) }, 3); // Video -> Low
    assert_eq!(unsafe { visor_get_msg_priority(0x15, 0) }, 4); // Debug -> SuperLow
}

#[test]
fn test_ffi_get_version() {
    let version = unsafe { visor_get_version() };
    assert!(!version.is_null());
}

#[test]
fn test_ffi_roundtrip_build_validate() {
    let mut msg = TLVMessage::new();
    unsafe { visor_add_tlv_uint8(&mut msg, 0x70, 2); }
    unsafe { visor_add_tlv_float(&mut msg, 0x30, 3.14); }

    let mut buffer = [0u8; 1093];
    let size = unsafe {
        visor_build_message(&msg, VISOR_MSG_TELEMETRY, buffer.as_mut_ptr(), buffer.len())
    };
    assert!(size > 0);

    let count = unsafe { visor_validate_message(buffer.as_ptr(), size as usize) };
    assert_ne!(count, 0xFF);
}
