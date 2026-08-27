///!

//! # VISOR Protocol — Biblioteca Principal
//!
//! Esta crate implementa o protocolo de comunicação TLV (Type-Length-Value)
//! binário v2.0.0 do sistema AERUS VISOR.
//!
//! ## Módulos
//!
//! - `protocol` — Protocolo TLV: tipos, CRC8, builder, codec e FFI
//! - `parser` — Parser FSM para reconstrução de mensagens TLV
//!
//! ## Uso em Rust
//!
//! ```rust
//! use visor_protocol::protocol::builder::TLVBuilder;
//! use visor_protocol::protocol::codec::validate_message;
//! use visor_protocol::parser::fsm::Parser;
//!
//! // Construir mensagem
//! let mut builder = TLVBuilder::new();
//! builder.add_uint8(0x70, 2).unwrap();
//! builder.add_float(0x30, 1.5).unwrap();
//! let mut buffer = [0u8; 1093];
//! let size = builder.build(0x11, &mut buffer).unwrap();
//!
//! // Validar
//! assert!(validate_message(&buffer[..size]).is_ok());
//!
//! // Parse
//! let mut parser = Parser::new();
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
//! visor_add_tlv_uint8(&msg, 0x70, 2);
//! visor_add_tlv_float(&msg, 0x30, 1.5);
//!
//! uint8_t buffer[1093];
//! ssize_t size = visor_build_message(&msg, 0x11, buffer, sizeof(buffer));
//! ```

#![no_std]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(dead_code)]

pub mod protocol;
pub mod parser;
