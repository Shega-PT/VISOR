//! # VISOR Protocol — Biblioteca Principal
//!
//! Esta crate implementa o protocolo de comunicação ACP (AERUS Communication
//! Protocol) v3.0.0 binário do sistema AERUS.
//!
//! ## Módulos
//!
//! - `protocol` — Protocolo ACP: tipos, CRC8/CRC16, builder, codec e FFI
//! - `parser` — Parser FSM para reconstrução de mensagens ACP
//!
//! ## Formato da Mensagem ACP v3.0.0
//!
//! ```text
//! [START_BYTE][VERSION][NODE_ID][MSG_ID][SEQ_NUM(2)][TLV_COUNT]
//! [TLV_FIELDS...][SIGNATURE][CRC16(2)]
//! ```
//!
//! ## Uso em Rust
//!
//! ```rust
//! use visor_protocol::protocol::builder::TLVBuilder;
//! use visor_protocol::protocol::codec::validate_message;
//! use visor_protocol::parser::fsm::Parser;
//!
//! // Construir mensagem
//! let mut builder = TLVBuilder::new(0x06, 0x42);
//! builder.add_u8_field(0, 2).unwrap();          // State = Ready
//! builder.add_f32_field(0x10, 1.5).unwrap();    // Roll = 1.5
//! builder.set_seq(42);
//! let mut buffer = [0u8; 1098];
//! let size = builder.build(0x11, &mut buffer).unwrap();
//!
//! // Validar
//! assert!(validate_message(&buffer[..size]).is_ok());
//!
//! // Parse
//! let mut parser = Parser::new(0x42);
//! for &byte in &buffer[..size] {
//!     parser.feed(byte);
//! }
//! assert!(parser.has_message());
//! ```
//!
//! ## Uso em C/C++ (via FFI)
//!
//! Incluir `protocol_ffi.h` e linking com a library estática:
//!
//! ```c
//! #include "protocol_ffi.h"
//!
//! TLVMessage msg;
//! visor_acp_init(&msg, 0x06, 0x11);
//! visor_add_tlv_uint8(&msg, 0xC0, 2);
//! visor_add_tlv_float(&msg, 0x30, 1.5);
//!
//! uint8_t buffer[1098];
//! ssize_t size = visor_build_message(&msg, 0x11, buffer, sizeof(buffer));
//! ```

#![cfg_attr(target_os = "none", no_std)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(dead_code)]

#[cfg(target_os = "none")]
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}

pub mod protocol;
pub mod parser;
