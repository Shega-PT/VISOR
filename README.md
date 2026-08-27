# VISOR — Módulo de Visão Computacional

 **Autor: ShegaPT**

Sistema de visão computacional autónomo para o módulo AERUS, responsável pela captura, processamento e transmissão de vídeo ao Master via interface de transporte unidirecional.

## Visão Geral

O VISOR é um módulo quase totalmente autónomo do sistema AERUS. A sua função principal é:

1. Capturar vídeo de uma camera (OV2640 via DVP)
2. Processar imagens (redimensionamento, filtros, correção gamma)
3. Empacotar em container AVI com codec MJPEG
4. Fragmentar em chunks TLV
5. Enviar ao Master via callback de transporte

**Características:**
- Unidirecional — apenas envia dados, não recebe comunicação
- Protocolo TLV compartilhado com outros módulos AERUS
- Suporte a PSRAM para buffers de vídeo grandes
- Reutilizável em outros sistemas de voo (Corvus, etc.)

## Arquitetura

```
┌─────────────────────────────────────────────────────────┐
│                    VISOR Module                         │
├─────────────────────────────────────────────────────────┤
│  Video (Main)                                           │
│  ├── Camera (OV2640 / StoredVideo)                      │
│  ├── VideoProcessor (Resize, Filtros, Gamma)            │
│  ├── AviMjpegWriter (Container AVI + MJPEG)             │
│  └── Protocol FFI (Rust)                                │
│       ├── Protocol: Types, CRC8, Builder, Codec         │
│       └── Parser: FSM + FFI                             │
├─────────────────────────────────────────────────────────┤
│  TransportInterface (Callback)                          │
│  └── I2C / SPI / UART / Outro                           │
└─────────────────────────────────────────────────────────┘
```

## Diretórios

```
VISOR/
├── rust/                        # Protocolo TLV (Rust)
│   ├── Cargo.toml
│   ├── rust-toolchain.toml
│   ├── build.rs
│   ├── src/
│   │   ├── lib.rs
│   │   ├── protocol/
│   │   │   ├── types.rs
│   │   │   ├── crc8.rs
│   │   │   ├── builder.rs
│   │   │   ├── codec.rs
│   │   │   └── ffi.rs
│   │   └── parser/
│   │       ├── fsm.rs
│   │       └── ffi.rs
│   └── tests/
│       ├── test_types.rs
│       ├── test_crc8.rs
│       ├── test_builder.rs
│       ├── test_codec.rs
│       └── test_fsm.rs
├── lib/protocol_ffi/
│   ├── include/protocol_ffi.h
│   └── lib/
├── components/visor_video/
│   ├── CMakeLists.txt
│   ├── include/
│   │   ├── transport.h
│   │   ├── CameraConfig.h
│   │   ├── Camera.h
│   │   ├── CameraOV2640.h
│   │   ├── StoredVideo.h
│   │   ├── VideoProcessor.h
│   │   ├── AviMjpegWriter.h
│   │   └── Video.h
│   └── src/
│       ├── CameraOV2640.cpp
│       ├── StoredVideo.cpp
│       ├── VideoProcessor.cpp
│       ├── AviMjpegWriter.cpp
│       └── Video.cpp
├── src/main.cpp
├── scripts/build_rust.py
├── test/
│   ├── test_transport_camera.cpp
│   └── test_video_ffi.cpp
├── platformio.ini
├── README.md
└── CHANGELOG.md
```

## Componentes

### Protocolo (Rust)

| Módulo    | Descrição                                                                 |
|-----------|---------------------------------------------------------------------------|
| `types`   | Constantes, enums (MsgId, FieldId, FieldGps, etc.) e structs `#[repr(C)]` |
| `crc8`    | CRC-8/SMBUS com tabela de 256 entradas                                    |
| `builder` | `TLVBuilder` — construção e serialização de mensagens TLV                 |
| `codec`   | Funções de construção, validação e serialização de mensagens              |
| `ffi`     | Funções `extern "C"` para FFI com C/C++                                   |
| `fsm`     | Parser FSM — máquina de estados para reassembly de mensagens TLV          |

### Vídeo (C/C++)

| Módulo               | Descrição                                             |
|----------------------|-------------------------------------------------------|
| `Video`              | Classe principal — pipeline completo de processamento |
| `Camera`             | Interface abstrata para cameras                       |
| `CameraOV2640`       | Driver OV2640 via esp_camera (ESP32)                  |
| `StoredVideo`        | Vídeo armazenado para testes                          |
| `VideoProcessor`     | Resize, filtros de brilho/contraste, correção gamma   |
| `AviMjpegWriter`     | Escritor de container AVI com codec MJPEG             |
| `TransportInterface` | Interface de transporte via callback                  |

## Formato AVI

| Campo            | Valor                |
|------------------|----------------------|
| Codec            | MJPEG                |
| Container        | AVI (RIFF)           |
| Resolução padrão | VGA 640×480          |
| Quality          | 80-90 (configurável) |
| FPS alvo         | 30 (min 15, max 60)  |

## Formato TLV

| Campo      | Tamanho       | Descrição                 |
|------------|---------------|---------------------------|
| Start Byte | 1             | `0xAA`                    |
| Msg ID     | 1             | Identificador da mensagem |
| TLV Count  | 1             | Número de campos TLV      |
| TLV Fields | N × (2 + len) | Campo ID + Length + Value |
| Checksum   | 1             | CRC-8/SMBUS               |

## Requisitos

- **PlatformIO** com framework ESP-IDF
- **Rust** com toolchain `esp` (Xtensa)
- **ESP32** com PSRAM habilitada
- Camera OV2640 (ou StoredVideo para testes)

## Construção

```bash
# Compilar firmware ESP32
pio run

# Compilar e flash
pio run -t upload

# Testes Rust
cd rust && cargo test

# Testes C/C++ (host)
cd test && g++ -o test_main test_video_ffi.cpp -I../components/visor_video/include -I../lib/protocol_ffi/include && ./test_main
```

## Testes

- **Rust**: Testes unitários em `rust/tests/` — tipos, CRC8, builder, codec, FSM
- **C/C++**: Testes de integração em `test/` — transporte, camera, video, protocolo FFI
- **Objetivo**: Cobertura máxima (100% desejado)

## Protocolo TLV

O protocolo TLV é compartilhado entre todos os módulos do AERUS. As mensagens são serializadas em formato binário com CRC-8 para integridade.

### IDs de Mensagem

| ID   | Nome      | Prioridade    |
|------|-----------|---------------|
| 0x10 | HEARTBEAT | Normal        |
| 0x11 | TELEMETRY | Normal        |
| 0x12 | COMMAND   | High          |
| 0x13 | ACK       | High          |
| 0x14 | FAILSAFE  | SuperCritical |
| 0x15 | DEBUG     | SuperLow      |
| 0x16 | VIDEO     | Low           |
| 0x17 | SHELL     | Normal        |
| 0x18 | SI_DATA   | Normal        |

### Campos de Vídeo

| ID   | Nome           | Tipo   |
|------|----------------|--------|
| 0xB0 | VIDEO_FRAME_ID | uint16 |
| 0xB1 | VIDEO_CHUNK_ID | uint8  |
| 0xB2 | VIDEO_TOTAL    | uint8  |
| 0xB3 | VIDEO_PAYLOAD  | raw    |

## Licença

GPLv3 — ver ficheiro `LICENSE` para detalhes.

## Autor

**ShegaPT** — Desenvolvimento e manutenção.
