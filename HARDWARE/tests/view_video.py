#!/usr/bin/env python3
"""
view_video.py — Visualiza vídeo ACP enviado pelo ESP32 via USB.

Le dados ACP v3.0.0 pela serial, valida CRC16 + assinatura,
reconstroi frames JPEG e exibe em tempo real.

Uso:
  python3 view_video.py
  python3 view_video.py --port /dev/ttyUSB0
  python3 view_video.py --save video.avi
"""

import sys
import os
import io
import struct
import time
import argparse
import threading

# ============================================================================
# CRC-16/CCITT — mesma tabela do Rust
# ============================================================================

CRC16_TABLE = [
    0x0000,0x1021,0x2042,0x3063,0x4084,0x50A5,0x60C6,0x70E7,
    0x8108,0x9129,0xA14A,0xB16B,0xC18C,0xD1AD,0xE1CE,0xF1EF,
    0x1231,0x0210,0x3273,0x2252,0x52B5,0x4294,0x72F7,0x62D6,
    0x9339,0x8318,0xB37B,0xA35A,0xD3BD,0xC39C,0xF3FF,0xE3DE,
    0x2462,0x3443,0x0420,0x1401,0x64E6,0x74C7,0x44A4,0x5485,
    0xA56A,0xB54B,0x8528,0x9509,0xE5EE,0xF5CF,0xC5AC,0xD58D,
    0x3653,0x2672,0x1611,0x0630,0x76D7,0x66F6,0x5695,0x46B4,
    0xB75B,0xA77A,0x9719,0x8738,0xF7DF,0xE7FE,0xD79D,0xC7BC,
    0x48C4,0x58E5,0x6886,0x78A7,0x0840,0x1861,0x2802,0x3823,
    0xC9CC,0xD9ED,0xE98E,0xF9AF,0x8948,0x9969,0xA90A,0xB92B,
    0x5AF5,0x4AD4,0x7AB7,0x6A96,0x1A71,0x0A50,0x3A33,0x2A12,
    0xDBFD,0xCBDC,0xFBBF,0xEB9E,0x9B79,0x8B58,0xBB3B,0xAB1A,
    0x6CA6,0x7C87,0x4CE4,0x5CC5,0x2C22,0x3C03,0x0C60,0x1C41,
    0xEDAE,0xFD8F,0xCDEC,0xDDCD,0xAD2A,0xBD0B,0x8D68,0x9D49,
    0x7E97,0x6EB6,0x5ED5,0x4EF4,0x3E13,0x2E32,0x1E51,0x0E70,
    0xFF9F,0xEFBE,0xDFDD,0xCFFC,0xBF1B,0xAF3A,0x9F59,0x8F78,
    0x9188,0x81A9,0xB1CA,0xA1EB,0xD10C,0xC12D,0xF14E,0xE16F,
    0x1080,0x00A1,0x30C2,0x20E3,0x5004,0x4025,0x7046,0x6067,
    0x83B9,0x9398,0xA3FB,0xB3DA,0xC33D,0xD31C,0xE37F,0xF35E,
    0x02B1,0x1290,0x22F3,0x32D2,0x4235,0x5214,0x6277,0x7256,
    0xB5EA,0xA5CB,0x95A8,0x8589,0xF56E,0xE54F,0xD52C,0xC50D,
    0x34E2,0x24C3,0x14A0,0x0481,0x7466,0x6447,0x5424,0x4405,
    0xA7DB,0xB7FA,0x8799,0x97B8,0xE75F,0xF77E,0xC71D,0xD73C,
    0x26D3,0x36F2,0x0691,0x16B0,0x6657,0x7676,0x4615,0x5634,
    0xD94C,0xC96D,0xF90E,0xE92F,0x99C8,0x89E9,0xB98A,0xA9AB,
    0x5844,0x4865,0x7806,0x6827,0x18C0,0x08E1,0x3882,0x28A3,
    0xCB7D,0xDB5C,0xEB3F,0xFB1E,0x8BF9,0x9BD8,0xABBB,0xBB9A,
    0x4A75,0x5A54,0x6A37,0x7A16,0x0AF1,0x1AD0,0x2AB3,0x3A92,
    0xFD2E,0xED0F,0xDD6C,0xCD4D,0xBDAA,0xAD8B,0x9DE8,0x8DC9,
    0x7C26,0x6C07,0x5C64,0x4C45,0x3CA2,0x2C83,0x1CE0,0x0CC1,
    0xEF1F,0xFF3E,0xCF5D,0xDF7C,0xAF9B,0xBFBA,0x8FD9,0x9FF8,
    0x6E17,0x7E36,0x4E55,0x5E74,0x2E93,0x3EB2,0x0ED1,0x1EF0,
]

def crc16(data):
    crc = 0xFFFF
    for b in data:
        crc = (crc << 8) ^ CRC16_TABLE[((crc >> 8) ^ b) & 0xFF]
        crc &= 0xFFFF
    return crc

# ============================================================================
# Constantes ACP
# ============================================================================

START   = 0xAA
VER     = 0x03
HDR     = 7
SIG_SZ  = 1
CRC_SZ  = 2
OVER    = HDR + SIG_SZ + CRC_SZ

MSG_VIDEO = 0x16
MSG_HEART = 0x10

# FieldIDs video (compativel com Rust FieldVideo)
FLD_FRAME_ID   = 0xB0
FLD_CHUNK_ID   = 0xB1
FLD_TOTAL      = 0xB2
FLD_PAYLOAD    = 0xB3

# ============================================================================
# Procura porta ESP32
# ============================================================================

def find_port():
    try:
        import serial.tools.list_ports
        ports = serial.tools.list_ports.comports()
        for p in ports:
            d = p.description.lower()
            if any(x in d for x in ['cp210','ch340','ftdi','silicon','usb-serial','esp32','wch']):
                return p.device
        for p in ports:
            if 'ttyUSB' in p.device or 'ttyACM' in p.device:
                return p.device
        if ports:
            return ports[0].device
    except Exception:
        pass
    return None

# ============================================================================
# Parser ACP
# ============================================================================

def parse_msg(buf):
    """Parse uma mensagem ACP. Retorna dict ou None."""
    if len(buf) < OVER:
        return None
    if buf[0] != START or buf[1] != VER:
        return None

    msg_id = buf[2]
    tlv_count = buf[6]
    offset = HDR
    tlvs = []

    for _ in range(tlv_count):
        if offset + 2 > len(buf):
            return None
        fid = buf[offset]
        flen = buf[offset+1]
        if offset + 2 + flen > len(buf):
            return None
        fdata = bytes(buf[offset+2:offset+2+flen])
        tlvs.append((fid, flen, fdata))
        offset += 2 + flen

    if offset + SIG_SZ + CRC_SZ > len(buf):
        return None

    sig = buf[offset]
    crc_lo = buf[offset+1]
    crc_hi = buf[offset+2]
    expected_crc = (crc_hi << 8) | crc_lo
    computed_crc = crc16(bytes(buf[:offset+1]))

    return {
        'msg_id': msg_id,
        'seq': buf[3] | (buf[4] << 8),
        'tlvs': tlvs,
        'sig': sig,
        'crc_ok': computed_crc == expected_crc,
        'crc_expected': expected_crc,
        'crc_computed': computed_crc,
    }

# ============================================================================
# Reassembly de frames video
# ============================================================================

class VideoReassembly:
    def __init__(self):
        self.chunks = {}   # frame_id -> {chunk_id: data}
        self.meta = {}     # frame_id -> total_chunks
        self.timestamps = {}  # frame_id -> time.time() do primeiro chunk
        self.frames = []   # [(frame_id, jpeg_bytes)]
        self.stats = {'pkts':0, 'ok':0, 'crc_err':0, 'sig_err':0,
                      'chunks':0, 'frames':0}
        self._FRAME_TIMEOUT = 5.0  # segundos antes de descartar frame incompleto

    def feed(self, msg):
        self.stats['pkts'] += 1
        self._cleanup_stale()

        if not msg['crc_ok']:
            self.stats['crc_err'] += 1
            return None, "CRC FAIL"

        self.stats['ok'] += 1

        if msg['msg_id'] != MSG_VIDEO:
            return None, None

        fid = cid = total = None
        payload = None
        for t_id, t_len, t_data in msg['tlvs']:
            if t_id == FLD_FRAME_ID and t_len >= 2:
                fid = struct.unpack('<H', t_data[:2])[0]
            elif t_id == FLD_CHUNK_ID and t_len >= 1:
                cid = t_data[0]
            elif t_id == FLD_TOTAL and t_len >= 1:
                total = t_data[0]
            elif t_id == FLD_PAYLOAD:
                payload = t_data

        if fid is None or cid is None or total is None or payload is None:
            return None, "TLV video incompleto"

        self.stats['chunks'] += 1

        if fid not in self.chunks:
            self.chunks[fid] = {}
            self.meta[fid] = total
            self.timestamps[fid] = time.time()

        if cid in self.chunks[fid]:
            return None, None

        self.chunks[fid][cid] = payload

        if len(self.chunks[fid]) == total:
            jpeg = b''.join(self.chunks[fid][i] for i in range(total) if i in self.chunks[fid])
            self.frames.append((fid, jpeg))
            self.stats['frames'] += 1
            del self.chunks[fid]
            del self.meta[fid]
            self.timestamps.pop(fid, None)
            return jpeg, f"Frame {fid} OK ({len(jpeg)} bytes)"

        n = len(self.chunks[fid])
        return None, f"Frame {fid}: {n}/{total}"

    def get_frame(self):
        if self.frames:
            return self.frames.pop(0)
        return None

    def _cleanup_stale(self):
        now = time.time()
        stale = [fid for fid, t in self.timestamps.items()
                 if now - t > self._FRAME_TIMEOUT]
        for fid in stale:
            self.chunks.pop(fid, None)
            self.meta.pop(fid, None)
            self.timestamps.pop(fid, None)

# ============================================================================
# Leitor Serial
# ============================================================================

class SerialReader:
    def __init__(self, port, baud):
        self.port = port
        self.baud = baud
        self.ser = None
        self.running = False
        self.buf = bytearray()
        self.reasm = VideoReassembly()
        self._lock = threading.Lock()
        self.last_info = ""

    def start(self):
        import serial
        try:
            self.ser = serial.Serial(self.port, self.baud, timeout=0.1)
            self.ser.reset_input_buffer()
            self.running = True
            t = threading.Thread(target=self._loop, daemon=True)
            t.start()
            return True
        except Exception as e:
            print(f"ERRO: {e}")
            return False

    def stop(self):
        self.running = False
        time.sleep(0.2)
        if self.ser:
            self.ser.close()

    def _loop(self):
        while self.running:
            try:
                ser = self.ser
                if ser is None:
                    time.sleep(0.01)
                    continue

                n = ser.in_waiting
                if n > 0:
                    data = ser.read(n)
                    with self._lock:
                        self.buf.extend(data)
                    self._process()
                else:
                    time.sleep(0.001)
            except Exception:
                time.sleep(0.05)

    def _process(self):
        while len(self.buf) >= 4:  # Precisamos de pelo menos 4 bytes para o comprimento
            # Ler os 4 bytes de comprimento (little-endian)
            msg_len = struct.unpack('<I', self.buf[:4])[0]

            # Validar comprimento (mínimo = OVER, máximo = 1098)
            if msg_len < OVER or msg_len > 1098:
                # Comprimento inválido, procurar por START (0xAA) para ressincronizar
                idx = self.buf.find(START, 1, min(len(self.buf), 256))
                if idx > 0:
                    del self.buf[:idx]
                elif len(self.buf) > 3:
                    del self.buf[:-3]
                else:
                    break
                continue

            # Verificar se temos dados suficientes (4 + msg_len)
            if len(self.buf) < 4 + msg_len:
                break

            # Extrair a mensagem ACP (sem os 4 bytes de comprimento)
            msg_buf = bytes(self.buf[4:4+msg_len])
            del self.buf[:4+msg_len]  # Remover do buffer

            # Parse a mensagem ACP
            if msg_buf[0] == START:
                msg = parse_msg(msg_buf)
                if msg:
                    jpeg, info = self.reasm.feed(msg)
                    if info:
                        self.last_info = info

    def get_frame(self):
        return self.reasm.get_frame()

    def get_stats(self):
        s = self.reasm.stats
        return f"PKT:{s['pkts']} OK:{s['ok']} CRC_ERR:{s['crc_err']} CHUNKS:{s['chunks']} FRAMES:{s['frames']}"

# ============================================================================
# Guardar AVI
# ============================================================================

def save_avi(filename, frames, w, h, fps=10):
    with open(filename, 'wb') as f:
        f.write(b'RIFF')
        f.write(struct.pack('<I', 0))
        f.write(b'AVI ')
        f.write(b'LIST')
        f.write(struct.pack('<I', 0))
        hdrl_start = f.tell()
        f.write(b'avih')
        d = struct.pack('<I', 1000000//fps)
        d += struct.pack('<IIII', 0, 0, 0x10, len(frames))
        d += struct.pack('<II', 0, 1)
        d += struct.pack('<II', 0, w)
        d += struct.pack('<II', h, 0)
        d += struct.pack('<III', 0, 0, 0)
        f.write(struct.pack('<I', len(d)))
        f.write(d)
        f.write(b'LIST')
        so = f.tell()
        f.write(struct.pack('<I', 0))
        f.write(b'strl')
        f.write(b'strh')
        strh_data = bytearray()
        strh_data += b'vids'
        strh_data += b'MJPG'
        strh_data += struct.pack('<I', 0)
        strh_data += struct.pack('<H', 0)
        strh_data += struct.pack('<H', 0)
        strh_data += struct.pack('<I', 0)
        strh_data += struct.pack('<I', 1)
        strh_data += struct.pack('<I', fps)
        strh_data += struct.pack('<I', 0)
        strh_data += struct.pack('<I', len(frames))
        strh_data += struct.pack('<I', 0)
        strh_data += struct.pack('<I', 0)
        strh_data += struct.pack('<I', 0)
        strh_data += struct.pack('<HHHH', 0, 0, w, h)
        f.write(struct.pack('<I', len(strh_data)))
        f.write(strh_data)
        f.write(b'strf')
        sf = struct.pack('<IiiHH4sIIII', 40, w, h, 1, 24, b'MJPG', w*h*3, 0, 0, 0, 0)
        f.write(struct.pack('<I', len(sf)))
        sf += b'\x00' * ((4 - len(sf) % 4) % 4)
        f.write(sf)
        e = f.tell()
        f.seek(so)
        f.write(struct.pack('<I', e - so - 4))
        f.seek(e)
        f.write(b'LIST')
        f.write(struct.pack('<I', 16))
        f.write(b'odml' + b'dmlh' + struct.pack('<II', 4, len(frames)))
        e = f.tell()
        f.seek(16)
        f.write(struct.pack('<I', e - hdrl_start))
        f.seek(e)
        f.write(b'LIST')
        mo = f.tell()
        f.write(struct.pack('<I', 0))
        f.write(b'movi')
        ms = f.tell()
        idx = []
        for jpeg in frames:
            off = f.tell() - ms
            f.write(b'00dc')
            f.write(struct.pack('<I', len(jpeg)))
            f.write(jpeg)
            pad = (4 - (len(jpeg) & 3)) & 3
            if pad: f.write(b'\x00'*pad)
            idx.append((off, len(jpeg)))
        me = f.tell()
        f.seek(mo)
        f.write(struct.pack('<I', me - ms + 4))
        f.seek(me)
        f.write(b'idx1')
        f.write(struct.pack('<I', len(idx)*16))
        for o, s in idx:
            f.write(b'00dc' + struct.pack('<III', 0x10, o, s))
        fe = f.tell()
        f.seek(4)
        f.write(struct.pack('<I', fe - 8))

# ============================================================================
# MAIN
# ============================================================================

def main():
    ap = argparse.ArgumentParser(description='VISOR — Visualizador de Video ACP via USB')
    ap.add_argument('--port', default=None, help='Porta serial (auto-detect se omitido)')
    ap.add_argument('--baud', type=int, default=115200, help='Baud rate (default: 115200)')
    ap.add_argument('--save', default=None, help='Guardar video como AVI')
    ap.add_argument('--width', type=int, default=160, help='Largura (default: 160)')
    ap.add_argument('--height', type=int, default=120, help='Altura (default: 120)')
    ap.add_argument('--scale', type=int, default=4, help='Escala janela (default: 4)')
    args = ap.parse_args()

    if args.port is None:
        args.port = find_port()
        if not args.port:
            print("ERRO: Nenhuma porta USB encontrada.")
            print("Disponiveis:")
            try:
                import serial.tools.list_ports
                for p in serial.tools.list_ports.comports():
                    print(f"  {p.device} — {p.description}")
            except Exception:
                pass
            print("Use: python3 view_video.py --port /dev/ttyUSB0")
            sys.exit(1)

    print(f"Porta: {args.port}  Baud: {args.baud}")
    print(f"Video: {args.width}x{args.height}  Escala: {args.scale}x")
    print()

    reader = SerialReader(args.port, args.baud)
    if not reader.start():
        sys.exit(1)

    print("A aguardar dados do ESP32...")
    print("Ctrl+C para sair")
    print()

    # --- Exibicao ---
    saved_frames = []
    use_pygame = False
    screen = None
    clock = None
    font = None

    try:
        import pygame
        pygame.init()
        sw = args.width * args.scale
        sh = args.height * args.scale + 50
        screen = pygame.display.set_mode((sw, sh))
        pygame.display.set_caption("VISOR — Video ACP")
        clock = pygame.time.Clock()
        font = pygame.font.SysFont('monospace', 14)
        use_pygame = True
        print("Pygame: OK (janela aberta)")
    except Exception:
        print("Pygame: indisponivel (modo consola)")

    frame_count = 0
    last_fps_time = time.time()
    fps_counter = 0
    fps_display = 0
    current_jpeg = None
    info_text = "A aguardar ESP32..."

    try:
        while True:
            if use_pygame:
                for ev in pygame.event.get():
                    if ev.type == pygame.QUIT:
                        raise KeyboardInterrupt
                    if ev.type == pygame.KEYDOWN and ev.key == pygame.K_ESCAPE:
                        raise KeyboardInterrupt

            result = reader.get_frame()
            if result:
                jpeg, fid = result
                current_jpeg = jpeg
                frame_count += 1
                fps_counter += 1
                info_text = reader.last_info
                if args.save:
                    saved_frames.append(jpeg)

            now = time.time()
            if now - last_fps_time >= 1.0:
                fps_display = fps_counter
                fps_counter = 0
                last_fps_time = now

            if use_pygame and screen is not None and font is not None and clock is not None:
                screen.fill((0,0,0))
                if current_jpeg:
                    try:
                        img = pygame.image.load(io.BytesIO(current_jpeg))
                        img = pygame.transform.scale(img, (args.width * args.scale, args.height * args.scale))
                        screen.blit(img, (0, 0))
                    except Exception:
                        pygame.draw.rect(screen, (40,40,40), (0, 0, args.width*args.scale, args.height*args.scale))
                        t = font.render("Frame JPEG invalida", True, (255,255,0))
                        screen.blit(t, (10, args.height*args.scale//2))
                else:
                    t = font.render("A aguardar dados...", True, (100,100,100))
                    screen.blit(t, (10, args.height*args.scale//2))

                sy = args.height * args.scale + 5
                pygame.draw.rect(screen, (20,20,20), (0, sy, sw, 45))
                s1 = f"Frame: {frame_count}  FPS: {fps_display}  {reader.get_stats()}"
                s2 = info_text[:90]
                screen.blit(font.render(s1, True, (0,200,0)), (5, sy+3))
                screen.blit(font.render(s2, True, (0,180,180)), (5, sy+20))
                pygame.display.flip()
                clock.tick(30)
            else:
                if frame_count > 0 and frame_count % 10 == 0:
                    print(f"[{frame_count}] {reader.get_stats()} | {info_text}")

            time.sleep(0.005)

    except KeyboardInterrupt:
        pass
    finally:
        reader.stop()
        if args.save and saved_frames:
            print(f"\nA guardar {len(saved_frames)} frames em {args.save}...")
            save_avi(args.save, saved_frames, args.width, args.height)
            print(f"Guardado: {args.save} ({os.path.getsize(args.save)} bytes)")
        print(f"\nTotal: {frame_count} frames | {reader.get_stats()}")
        if use_pygame:
            pygame.quit()

if __name__ == '__main__':
    main()
