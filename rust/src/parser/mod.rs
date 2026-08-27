///!

//! # Módulo Parser
//!
//! Implementação do parser FSM (Finite State Machine) para reconstrução
//! de mensagens TLV a partir de um fluxo de bytes.
//!
//! Este módulo fornece:
//! - FSM de 7 estados para parsing byte-a-byte (`fsm`)
//! - Camada FFI para interoperação com C/C++ (`ffi`)

pub mod fsm;
pub mod ffi;
