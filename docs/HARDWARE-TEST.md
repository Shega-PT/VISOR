# Hardware Testing — VISOR

Guia dos testes de hardware do módulo VISOR.

## Scripts de Teste

As scripts de teste estão em `HARDWARE/tests/`:

| Script | Descrição |
|---|---|
| `view_video.py` | Visualizador de vídeo ACP em tempo real via USB serial |
| `generate_test_video.py` | Gera ficheiro AVI de teste com frames JPEG sintéticas |

## view_video.py

Visualizador de vídeo ACP recebido do ESP32 via USB serial.

### Funcionalidades
- Auto-detecção de porta USB (CP2102, CH340, FTDI)
- Decodificação JPEG em tempo real com pygame
- Validação CRC16 + assinatura ACP v3.0.0
- Reassembly de chunks fragmentados
- Guarda como AVI

### Utilização

```bash
cd HARDWARE/tests
source ../../.venv/bin/activate

# Auto-detect porta
python3 view_video.py

# Porta específica
python3 view_video.py --port /dev/ttyUSB0

# Resolução personalizada
python3 view_video.py --width 320 --height 240

# Guardar video
python3 view_video.py --save output.avi

# Escala da janela
python3 view_video.py --scale 2
```

### Argumentos

| Arg | Default | Descrição |
|---|---|---|
| `--port` | auto-detect | Porta serial |
| `--baud` | 921600 | Baud rate |
| `--save` | None | Guardar como AVI |
| `--width` | 160 | Largura frame |
| `--height` | 120 | Altura frame |
| `--scale` | 4 | Escala janela |

## generate_test_video.py

Gera ficheiro AVI com frames JPEG sintéticas para teste offline.

```bash
cd HARDWARE/tests
python3 generate_test_video.py
```

O output é um ficheiro AVI e um header C (`test_frames.h`) com os frames codificados.
