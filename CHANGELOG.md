# Changelog

Todas as alterações notáveis neste projeto são documentadas neste ficheiro.

O formato é baseado em [Keep a Changelog](https://keepachangelog.com/pt-BR/1.0.0/),
e este projeto adere ao [Semantic Versioning](https://semver.org/lang/pt-BR/).

## [3.0.0] - 2026-08-27
   **Autor: ShegaPT**

### Added

- **Protocolo ACP v3.0.0** — Reescrita completa para AERUS Communication Protocol
  - Formato wire: `[START][VER][NODE][MSG][SEQ:2][TLV_COUNT][TLV...][SIG][CRC16:2]`
  - Overhead: 10 bytes (header 7 + signature 1 + CRC16 2)
  - Máximo: 32 campos TLV, 1098 bytes por mensagem
- **FieldID com tipo embutido** — `[TYPE:3][ID:5]` = 8 tipos × 32 IDs = 256 campos
  - Tipos: Raw, Float32, Float16, Int32, Uint32, Uint16, Uint8, Bool
- **CRC-16/CCITT** — Polynomial 0x1021, Init 0xFFFF, tabela de 256 entradas
- **Assinatura XOR** — `key ^ msg_id ^ seq_lo ^ seq_hi` por nó
- **SEQ_NUM u16** — Anti-replay para taxas até 100Hz
- **CAN ID Extended (29-bit)** — `[PRIO:3][SRC:4][DST:4][TYPE:4][RES:14]`
  - 7 grupos computacionais (RaspberryPi, ESP32-S, ESP32-A, ESP32-FS, ESP32-FS_A, Visor)
  - 8 tipos de mensagem CAN (Data, Cmd, Ack, Event, Sync, State, Heart, Safety)
- **Parser FSM 9 estados** — WaitStart → WaitHeader → WaitTlvCount → WaitTlvId → WaitTlvLen → WaitTlvData → WaitSignature → WaitCrc16Lo → WaitCrc16Hi
- **Tipos de sistema** — SystemState, FlightMode, FailsafeReason, FailsafeAction, PriorityLevel
- **Campo ID unificado** — FieldId enum com valores pré-codificados
- **Documentação completa** em `rust/docs/`:
  - ACP-SPECIFICATION.md, FIELD-ID-REFERENCE.md, MESSAGE-ID-REFERENCE.md
  - CAN-ID-MAPPING.md, PARSER-GUIDE.md, BUILDER-GUIDE.md
  - FFI-GUIDE.md, DEVELOPER-TIPS.md, EXAMPLES.md

### Changed

- **Versão do protocolo:** 2.0.0 → 3.0.0
- **Formato wire:** TLV simples → TLV com header completo (version, node, msg, seq, tlv_count)
- **CRC:** CRC-8/SMBUS (1 byte) → CRC-16/CCITT (2 bytes)
- **Tamanho da mensagem:** Variável → Máximo 1098 bytes fixo
- **FieldID:** Simple ID → Tipo embutido `[TYPE:3][ID:5]`
- **Assinatura:** Não existia → XOR com chave por nó
- **Anti-replay:** Não existia → SEQ_NUM u16
- **Parser:** 6 estados → 9 estados (WaitHeader, WaitSignature, WaitCrc16Lo/Hi)
- **Crate types:** `["staticlib"]` → `["staticlib", "rlib"]` (para testes)
- **Features:** `std = ["esp-idf-sys"]` → `std = []` + `esp = ["std", "esp-idf-sys"]`
- **Conditional no_std:** Baseado em `target_os = "none"` em vez de feature flag
- **Testes:** De testes unitários separados para suite unificada (89 unit + 42 integration + 4 doc = 135 total)

### Removed

- **Testes antigos** — test_fsm.rs, test_builder.rs, test_codec.rs, test_crc8.rs, test_types.rs (substituídos por test_acp.rs)
- **CRC-8/SMBUS** como checksum principal (mantido como utilitário)
- **TLV_HEADER_SIZE** antigo (simplificado)

### Fixed

- **WaitHeader off-by-one** — Parser lia TLV_COUNT como byte de header (AcpHeaderSize-1 bug)
- **CRC-16 table** — 112/256 entradas incorretas, tabela regenerada corretamente
- **CRC-8 test vector** — Teste esperava 0x46, valor correto é 0xF4
- **is_safety_bus_id** — Verificava bits errados (priority+src_group em vez de msg_type)
- **Doc example** — validate_message chamado com 2 argumentos em vez de 1

---

## [0.1.0] - 2026-08-27
   **Autor: ShegaPT**

### Added

- **Protocolo TLV v2.0.0** — Reescrito em Rust com FFI C
  - `types.rs` — Constantes, enums e structs `#[repr(C)]`
  - `crc8.rs` — CRC-8/SMBUS puro Rust
  - `builder.rs` — TLVBuilder para construção e serialização
  - `codec.rs` — Funções de construção, validação e serialização
  - `ffi.rs` — Funções `extern "C"` para FFI
  - `lib.rs` — Módulo principal com re-exports
- **Parser FSM** — Reescrito em Rust com FFI C
  - `fsm.rs` — Máquina de estados para reassembly TLV
  - `ffi.rs` — Funções `extern "C"` do parser
- **Interface de Transporte** — Callback function pointer
  - `transport.h` — `TransportInterface` com inline helpers
  - Substitui LoRa por callback unidirecional
- **Módulo de Vídeo** — Adaptado para nova arquitetura
  - `Video.h/.cpp` — Classe principal com pipeline completo
  - `Camera.h` — Interface abstrata para cameras
  - `CameraOV2640.h/.cpp` — Driver OV2640 via esp_camera
  - `StoredVideo.h/.cpp` — Vídeo armazenado para testes
  - `VideoProcessor.h/.cpp` — Processamento de imagem
  - `AviMjpegWriter.h/.cpp` — Escritor AVI MJPEG
  - `CameraConfig.h` — Configuração de pinos da camera
- **Build System** — PlatformIO + Cargo
  - `platformio.ini` — Configuração ESP32
  - `scripts/build_rust.py` — Script de build Rust
  - `rust/Cargo.toml` — Crate Rust (staticlib)
  - `rust/rust-toolchain.toml` — Toolchain ESP
- **Testes** — Unitários (Rust) e Integração (C/C++)
  - `rust/tests/test_types.rs` — Tipos e constantes
  - `rust/tests/test_crc8.rs` — CRC-8/SMBUS
  - `rust/tests/test_builder.rs` — TLVBuilder e FFI
  - `rust/tests/test_codec.rs` — Codec, validação e FFI
  - `rust/tests/test_fsm.rs` — Parser FSM e FFI
  - `test/test_transport_camera.cpp` — Transporte e Camera
  - `test/test_video_ffi.cpp` — Video e Protocolo FFI
- **Documentação**
  - `README.md` — Visão geral, arquitetura e construções
  - `CHANGELOG.md` — Este ficheiro
