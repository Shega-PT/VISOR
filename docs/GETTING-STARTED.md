# Getting Started — VISOR

Guia passo-a-passo para configurar, compilar e testar o módulo VISOR.

## Requisitos

- **PlatformIO CLI** (ou VS Code + extensão PlatformIO)
- **Rust** com toolchain `esp` (Xtensa) — instalar com `espup install`
- **Python 3.10+** com `pyserial` e `pygame` (para testes HW)
- **ESP32** (DevKitV1 para teste, ESP32-CAM para produção)

## 1. Configurar Ambiente

```bash
# Ativar venv Python (para testes)
python3 -m venv .venv
source .venv/bin/activate
pip install pyserial pygame

# Verificar toolchain Rust
RUSTUP_TOOLCHAIN=esp rustc --version
```

## 2. Compilar Firmware

```bash
# Compilar para ESP32 DevKitV1 (teste sem câmara)
pio run -e esp32dev

# Compilar para ESP32-CAM (produção)
pio run -e esp32cam
```

## 3. Flash no ESP32

```bash
# DevKitV1
pio run -e esp32dev -t upload

# ESP32-CAM
pio run -e esp32cam -t upload
```

## 4. Testar Video

```bash
# No PC, com ESP32 ligado via USB
cd HARDWARE/tests
python3 view_video.py

# Com porta específica
python3 view_video.py --port /dev/ttyUSB0

# Guardar video como AVI
python3 view_video.py --save video.avi
```

## 5. Testes Rust (host)

```bash
cd rust
RUSTUP_TOOLCHAIN=stable cargo test
```

## Troubleshooting

### Build falha: "esp_camera" not found
O componente `esp32-camera` é gerido via `idf_component.yml`. Limpar managed_components:
```bash
rm -rf managed_components dependencies.lock
```

### view_video.py crash na abertura
Verificar que `pyserial` e `pygame` estão instalados no venv.

### UART sem dados
O ESP32 trata graciosamente o caso do console IDF já ter instalado o driver UART0 — reconfigura o baud rate automaticamente. Verificar que `view_video.py` está a correr com a porta correta.
