/**
 * @file test_crc8.rs
 * @brief Testes de integração do módulo CRC-8/SMBUS.
 */

use visor_protocol::protocol::crc8::*;
use visor_protocol::protocol::ffi::*;

#[test]
fn test_crc8_empty() {
    assert_eq!(calc_crc8(&[]), 0x00);
}

#[test]
fn test_crc8_single_byte() {
    assert_eq!(calc_crc8(&[0x00]), 0x00);
    assert_eq!(calc_crc8(&[0x01]), 0x07);
}

#[test]
fn test_crc8_known_vectors() {
    assert_eq!(calc_crc8(&[0x01, 0x02, 0x03]), 0x46);
    assert_eq!(calc_crc8(&[0xFF, 0xFF, 0xFF]), 0x86);
    assert_eq!(calc_crc8(&[0x00, 0x00, 0x00, 0x00]), 0x00);
}

#[test]
fn test_crc8_consistency() {
    let data = b"TEST DATA FOR CRC8";
    let result1 = calc_crc8(data);
    let result2 = calc_crc8(data);
    assert_eq!(result1, result2);
}

#[test]
fn test_crc8_different_data() {
    let data1 = b"DATA1";
    let data2 = b"DATA2";
    assert_ne!(calc_crc8(data1), calc_crc8(data2));
}

#[test]
fn test_crc8_verify() {
    let data = [0x01, 0x02, 0x03];
    let crc = calc_crc8(&data);
    assert!(verify_crc8(&data, crc));
    assert!(!verify_crc8(&data, crc.wrapping_add(1)));
}

#[test]
fn test_crc8_table() {
    let table = get_crc8_table();
    assert_eq!(table[0], 0x00);
    assert_eq!(table[1], 0x07);
    assert_eq!(table.len(), 256);
}

#[test]
fn test_crc8_ffi() {
    let data = b"TEST";
    let ffi_result = unsafe { visor_calc_crc8(data.as_ptr(), data.len()) };
    let rust_result = calc_crc8(data);
    assert_eq!(ffi_result, rust_result);
}

#[test]
fn test_crc8_ffi_empty() {
    let ffi_result = unsafe { visor_calc_crc8(std::ptr::null(), 0) };
    assert_eq!(ffi_result, 0x00);
}

#[test]
fn test_crc8_max_values() {
    let data = [0xFF; 256];
    let result = calc_crc8(&data);
    assert!(result <= 0xFF);
}
