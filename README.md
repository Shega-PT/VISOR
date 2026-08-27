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
- Protocolo ACP v3.0.0 compartilhado com outros módulos AERUS
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
│       ├── Protocol: Types, CRC8, CRC16, Builder, Codec  │
│       └── Parser: FSM + FFI                             │
├─────────────────────────────────────────────────────────┤
│  TransportInterface (Callback)                          │
│  └── I2C / SPI / UART / Outro                           │
└─────────────────────────────────────────────────────────┘
```

## Diretórios

```
VISOR/
├── rust/                        # Protocolo ACP v3.0.0 (Rust)
│   ├── Cargo.toml
│   ├── rust-toolchain.toml
│   ├── src/
│   │   ├── lib.rs
│   │   ├── protocol/
│   │   │   ├── mod.rs
│   │   │   ├── types.rs         # Tipos, constantes, enums
│   │   │   ├── crc8.rs          # CRC-8/SMBUS
│   │   │   ├── crc16.rs         # CRC-16/CCITT
│   │   │   ├── builder.rs       # TLVBuilder
│   │   │   ├── codec.rs         # Serialização e validação
│   │   │   └── ffi.rs           # FFI C
│   │   └── parser/
│   │       ├── mod.rs
│   │       ├── fsm.rs           # Parser FSM 9 estados
│   │       └── ffi.rs           # Parser FFI
│   ├── tests/
│   │   └── test_acp.rs          # Testes de integração
│   └── docs/                    # Documentação do protocolo
│       ├── ACP-SPECIFICATION.md
│       ├── FIELD-ID-REFERENCE.md
│       ├── MESSAGE-ID-REFERENCE.md
│       ├── CAN-ID-MAPPING.md
│       ├── PARSER-GUIDE.md
│       ├── BUILDER-GUIDE.md
│       ├── FFI-GUIDE.md
│       ├── DEVELOPER-TIPS.md
│       └── EXAMPLES.md
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

### Protocolo ACP v3.0.0 (Rust)

| Módulo    | Descrição                                                                 |
|-----------|---------------------------------------------------------------------------|
| `types`   | Constantes, enums (MsgId, FieldId, CanGroup, etc.) e structs `#[repr(C)]` |
| `crc8`    | CRC-8/SMBUS com tabela de 256 entradas                                    |
| `crc16`   | CRC-16/CCITT com tabela de 256 entradas                                   |
| `builder` | `TLVBuilder` — construção e serialização de mensagens ACP                 |
| `codec`   | Serialização, validação e parsing de mensagens ACP                        |
| `ffi`     | Funções `extern "C"` para FFI com C/C++                                   |
| `fsm`     | Parser FSM — máquina de 9 estados para reassembly de mensagens ACP        |

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

## Protocolo ACP v3.0.0

### Formato da Mensagem

```text
[START_BYTE:1][VERSION:1][NODE_ID:1][MSG_ID:1][SEQ_NUM:2 LE][TLV_COUNT:1]
[TLV_FIELDS...][SIGNATURE:1][CRC16:2 LE]
```

**Overhead:** 10 bytes | **Máx. tamanho:** 1098 bytes | **Máx. TLV fields:** 32

### Campo TLV

```text
[FIELD_ID:1][LEN:1][DATA:LEN]
```

FieldID = `[TYPE:3][ID:5]` — 8 tipos × 32 IDs = 256 campos possíveis

### Segurança

- **CRC-16/CCITT** — integridade dos dados
- **Signature XOR** — autenticidade (key ^ msg_id ^ seq_lo ^ seq_hi)
- **SEQ_NUM u16** — anti-replay

### IDs de Mensagem

| ID   | Nome      | Prioridade    |
|------|-----------|---------------|
| 0x10 | Heartbeat | Medium        |
| 0x11 | Telemetry | Medium        |
| 0x12 | Command   | High          |
| 0x13 | Ack       | High          |
| 0x14 | Failsafe  | SuperCritical |
| 0x15 | Debug     | Low           |
| 0x16 | Video     | Low           |
| 0x17 | Shell     | Medium        |
| 0x18 | SiData    | Medium        |
| 0x19 | Watchdog  | Medium        |
| 0x1A | Ping      | Medium        |
| 0x1B | Clock     | High          |

### CAN ID Extended (29-bit)

```text
[PRIORIDADE:3][SRC_GROUP:4][DST_GROUP:4][MSG_TYPE:4][RESERVADO:14]
```

| Grupo       | Valor | Descrição                    |
|-------------|:-----:|------------------------------|
| None        | 0x0   | Broadcast                    |
| RaspberryPi | 0x1   | Orquestração                 |
| Esp32S      | 0x2   | Sensores                     |
| Esp32A      | 0x3   | Atuadores                    |
| Esp32Fs     | 0x4   | Segurança                    |
| Esp32FsA    | 0x5   | Emergência                   |
| Visor       | 0x6   | Visão computacional          |

## Documentação

Ver `rust/docs/` para documentação completa:
- **ACP-SPECIFICATION.md** — Especificação completa do protocolo
- **FIELD-ID-REFERENCE.md** — Referência de todos os FieldIDs
- **MESSAGE-ID-REFERENCE.md** — Referência de todos os MsgIDs
- **CAN-ID-MAPPING.md** — Mapeamento CAN ID extended
- **PARSER-GUIDE.md** — Guia do parser FSM
- **BUILDER-GUIDE.md** — Guia do builder
- **FFI-GUIDE.md** — Guia FFI (C/C++)
- **DEVELOPER-TIPS.md** — Dicas para programadores
- **EXAMPLES.md** — Exemplos de uso

## Requisitos

- **PlatformIO** com framework ESP-IDF
- **Rust** com toolchain `esp` (Xtensa) para ESP32; `stable` para testes no host
- **ESP32** com PSRAM habilitada
- Camera OV2640 (ou StoredVideo para testes)

## Construção

```bash
# Compilar firmware ESP32
pio run

# Compilar e flash
pio run -t upload

# Testes Rust (host)
cd rust && RUSTUP_TOOLCHAIN=stable cargo test

# Testes C/C++ (host)
cd test && g++ -o test_main test_video_ffi.cpp -I../components/visor_video/include -I../lib/protocol_ffi/include && ./test_main
```

## Testes

- **Rust (unitários):** 89 testes em `src/` — types, CRC8, CRC16, builder, codec, parser FSM
- **Rust (integração):** 42 testes em `tests/test_acp.rs` — roundtrip, CAN ID, FieldID, validação
- **Rust (doc):** 4 testes doc em src/
- **C/C++:** Testes de integração em `test/` — transporte, camera, video, protocolo FFI
- **Total:** 135 testes Rust, todos a passar

## Licença

GPLv3 — ver ficheiro `LICENSE` para detalhes.

## Autor

**ShegaPT** — Desenvolvimento e manutenção.
