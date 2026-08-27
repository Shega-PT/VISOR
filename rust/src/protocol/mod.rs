///!

//! # Módulo Protocolo TLV
//!
//! Implementação completa do protocolo de comunicação TLV binário v2.0.0
//! do sistema AERUS VISOR.
//!
//! Este módulo fornece:
//! - Definição de tipos e constantes (`types`)
//! - Cálculo CRC-8/SMBUS (`crc8`)
//! - Construção de mensagens TLV (`builder`)
//! - Serialização/deserialização e validação (`codec`)
//! - Camada FFI para interoperação com C/C++ (`ffi`)

pub mod types;
pub mod crc8;
pub mod builder;
pub mod codec;
pub mod ffi;
