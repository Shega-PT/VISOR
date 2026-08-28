# Changelog

Todas as alterações notáveis neste projeto são documentadas neste ficheiro.

O formato é baseado em [Keep a Changelog](https://keepachangelog.com/pt-BR/1.0.0/),
e este projeto adere ao [Semantic Versioning](https://semver.org/lang/pt-BR/).

## [0.2.0] - 2026-08-28
   **Autor: ShegaPT**

### Fixed

- **view_video.py crash na abertura** — `reader.get_frame()` retornava None causando TypeError no unpack
- **view_video.py parse_msg() rejeitava todas as mensagens** — check `offset + OVER` incorreto, corrigido para `offset + SIG_SZ + CRC_SZ`
- **view_video.py decodificação JPEG** — `pygame.image.frombuffer` com dados JPEG como RGB, substituído por `pygame.image.load(io.BytesIO(...))`
- **Video.cpp stack overflow** — buffers de 153KB alocados na stack, migrados para heap com `malloc()`/`free()`
- **millis() não existe em ESP-IDF** — substituído por `esp_timer_get_time() / 1000` em Video.cpp e StoredVideo.cpp
- **CameraOV2640.h missing include** — `framesize_t`/`pixformat_t` não definidos, adicionado `#include "esp_camera.h"`
- **AviMjpegWriter.cpp offset errado** — dwTotalFrames escrito em offset 20 (era "hdrl"), corrigido para offset 48
- **AviMjpegWriter.cpp LIST sizes errados** — hdrl=228→208, strl=120→116
- **AviMjpegWriter.cpp strh dwLength** — nunca atualizado, corrigido no finalize()
- **AviMjpegWriter.cpp realloc() em SPIRAM** — substituído por malloc()+memcpy()+free() consistente
- **AviMjpegWriter.cpp _ensureCapacity loop infinito** — guard para _bufferCapacity==0
- **esp32cam build falha** — conflito esp32-camera em lib_deps vs managed_components, removido de lib_deps
- **platformio.ini paths inconsistentes** — -L redundante removido, CMakeLists.txt gerencia linkagem
- **test_transport_camera.cpp** — isActive()→isReady(), falta main(), args incorretos em begin()
- **test_video_ffi.cpp** — VISOR_FLD_*→ACP_FLD_*, args incorretos em visor_build_message() e visor_parser_new()
- **rust-toolchain.toml** — target `xtensa-esp32-espidf` em falta
- **Cargo.toml** — feature `no-std` inútil removida
- **parser/fsm.rs** — get_timestamp_us() sempre retornava 0 com código desnecessário, simplificado com TODO
- **main.cpp UART driver conflict** — `uart_driver_install` falhava quando console IDF já tinha UART0; agora trata `ESP_ERR_INVALID_STATE` graciosamente com `uart_set_baudrate()`
- **view_video.py import inútil** — `import subprocess` removido
- **view_video.py struct.pack args** — strh em save_avi() tinha 13 args para 12 formatos; reescrito com bytearray field-by-field
- **view_video.py f.seek(12)** — corrompia marcador "LIST" do AVI; corrigido para `f.seek(16)`
- **view_video.py reset_input_buffer** — boot logs causavam dessincronização; adicionado flush ao abrir serial
- **view_video.py _process scanning** — guard para `buf < 3` bytes; impedia loop infinito em buffers pequenos
- **view_video.py reassembly timeout** — frames incompletos acumulavam-se para sempre; timeout de 5s + cleanup automático
- **generate_test_video.py f.seek(12)** — mesmo bug de corrupção AVI; corrigido para `f.seek(16)`
- **test_transport_camera.cpp pinos** — valores AI-Thinker e ESP32-S3 não correspondiam a CameraConfig.h

### Changed

- **Estrutura de pastas** — scripts de teste movidos para `HARDWARE/tests/`
- **Documentação** — criada pasta `docs/` com GETTING-STARTED.md e HARDWARE-TEST.md
- **.gitignore** — atualizado: removidos view_video.py e generate_test_video.py da lista ignored; adicionados managed_components/, dependencies.lock, sdkconfig.*
- **.gitkeep** — criados em docs/, HARDWARE/tests/, lib/protocol_ffi/lib/
- **Bare except** — substituídos por `except Exception:` em view_video.py
- **self.buf slices** — otimizados com `del self.buf[:n]` em vez de reatribuição

### Removed

- **VideoFrame struct** — nunca utilizada, removida de Video.h
- **VideoCompression enum** — nunca utilizada, removida de Video.h
- **generate_minimal_jpeg()** — dead code removido de generate_test_video.py
- **visor_reader.py .pyc** — cache órfão removido

---

## [0.1.0] - 2026-08-27
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
