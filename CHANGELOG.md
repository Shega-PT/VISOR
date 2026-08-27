# Changelog

Todas as alterações notáveis neste projeto são documentadas neste ficheiro.

O formato é baseado em [Keep a Changelog](https://keepachangelog.com/pt-BR/1.0.0/),
e este projeto adere ao [Semantic Versioning](https://semver.org/lang/pt-BR/).

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

### Changed

- Nenhum (primeira versão da nova arquitetura)

### Removed

- Nenhum (primeira versão da nova arquitetura)

### Deprecated

- Nenhum (primeira versão da nova arquitetura)

### Fixed

- Nenhum (primeira versão da nova arquitetura)
