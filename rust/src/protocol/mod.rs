//! # Módulo Protocolo ACP (AERUS Communication Protocol)
//!
//! Implementação completa do protocolo de comunicação binário ACP v3.0.0
//! do sistema AERUS.
//!
//! Este módulo fornece:
//! - Definição de tipos e constantes (`types`) — CAN groups, FieldID com tipo, etc.
//! - Cálculo CRC-8/SMBUS (`crc8`) — legado, mantido por retrocompatibilidade
//! - Cálculo CRC-16/CCITT (`crc16`) — checksum padrão do ACP v3.0.0
//! - Construção de mensagens ACP (`builder`)
//! - Serialização/deserialização e validação (`codec`)
//! - Camada FFI para interoperação com C/C++ (`ffi`)

pub mod types;
pub mod crc8;
pub mod crc16;
pub mod builder;
pub mod codec;
pub mod ffi;
