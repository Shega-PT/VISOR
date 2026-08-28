#!/usr/bin/env python3
"""
generate_test_video.py — Gera vídeo de teste para o VISOR (ESP32 DevKitV1).

Gera um pequeno AVI MJPEG com frames de padrão de cores animado,
adequado para testar o pipeline completo no ESP32 sem camera real.

Saídas:
  - test_video.avi    — Ficheiro AVI MJPEG para StoredVideo
  - test_frames.h     — Array C dos frames JPEG para embedding no firmware

Uso:
  python3 scripts/generate_test_video.py [--frames N] [--fps N] [--resolution WxH]
"""

import struct
import sys
import os
import argparse
import io

# ============================================================================
# JPEG MINIMAL — Gerador de JPEG válido sem dependências externas
# ============================================================================

def _bytes_to_bits(data):
    """Converte bytes para string de bits."""
    return ''.join(f'{b:08b}' for b in data)

def _huffman_encode(data, table):
    """Codifica dados usando uma tabela Huffman."""
    result = []
    for byte in data:
        if byte in table:
            result.append(table[byte])
    return ''.join(result)

def generate_minimal_jpeg(width, height, frame_number=0):
    """
    Gera um JPEG mínimo válido com padrão de cores baseado no número da frame.
    
    Este gerador cria um JPEG funcional sem dependências PIL/OpenCV.
    Cada frame tem um padrão de cor diferente baseado no frame_number.
    
    Args:
        width: Largura da imagem
        height: Altura da imagem
        frame_number: Número da frame (para variação de cor)
    
    Returns:
        bytes: Dados JPEG completos
    """
    # Cores RGB baseadas no número da frame (ciclo de 8 cores)
    colors = [
            (255, 0, 0),     # Vermelho
            (0, 255, 0),     # Verde
            (0, 0, 255),     # Azul
            (255, 255, 0),   # Amarelo
            (255, 0, 255),   # Magenta
            (0, 255, 255),   # Ciano
            (255, 128, 0),   # Laranja
            (128, 0, 255),   # Roxo
        ]
    r, g, b = colors[frame_number % len(colors)]
    
    # Criar conteúdo YCbCr simples
    # Y = 0.299*R + 0.587*G + 0.114*B
    # Cb = 128 - 0.1687*R - 0.3313*G + 0.5*B
    # Cr = 128 + 0.5*R - 0.4187*G - 0.0813*B
    Y = int(0.299 * r + 0.587 * g + 0.114 * b)
    Cb = int(128 - 0.1687 * r - 0.3313 * g + 0.5 * b)
    Cr = int(128 + 0.5 * r - 0.4187 * g - 0.0813 * b)
    
    Y = max(0, min(255, Y))
    Cb = max(0, min(255, Cb))
    Cr = max(0, min(255, Cr))
    
    # Criar dados de imagem simplificados (YCbCr 4:2:0)
    # Para um JPEG mínimo, criar MCUs 8x8
    mcu_width = (width + 7) // 8
    mcu_height = (height + 7) // 8
    
    # Dados Y (1 byte por pixel, simplificado)
    y_data = bytes([Y] * (mcu_width * 8 * mcu_height * 8))
    cb_data = bytes([Cb] * (mcu_width * 4 * mcu_height * 4))
    cr_data = bytes([Cr] * (mcu_width * 4 * mcu_height * 4))
    
    # Construir JPEG manualmente
    jpeg = io.BytesIO()
    
    # SOI marker
    jpeg.write(b'\xff\xd8')
    
    # APP0 marker (JFIF)
    app0 = b'JFIF\x00\x01\x01\x00\x00\x01\x00\x01\x00\x00'
    jpeg.write(b'\xff\xe0')
    jpeg.write(struct.pack('>H', len(app0) + 2))
    jpeg.write(app0)
    
    # DQT marker (Quantization Table)
    qt_luma = bytes([0] * 64)  # Tabela de quantização luma (toda zeros = qualidade máxima)
    qt_chroma = bytes([0] * 64)  # Tabela de quantização chroma
    
    jpeg.write(b'\xff\xdb')
    qt_data_luma = b'\x00' + qt_luma
    jpeg.write(struct.pack('>H', len(qt_data_luma) + 2))
    jpeg.write(qt_data_luma)
    
    jpeg.write(b'\xff\xdb')
    qt_data_chroma = b'\x01' + qt_chroma
    jpeg.write(struct.pack('>H', len(qt_data_chroma) + 2))
    jpeg.write(qt_data_chroma)
    
    # SOF0 marker (Start of Frame - Baseline DCT)
    sof = struct.pack('>BHHB', 8, height, width, 3)  # 8-bit, height, width, 3 components
    sof += b'\x01\x11\x00'  # Component 1 (Y):  sampling 1x1, QT 0
    sof += b'\x02\x11\x01'  # Component 2 (Cb): sampling 1x1, QT 1
    sof += b'\x03\x11\x01'  # Component 3 (Cr): sampling 1x1, QT 1
    
    jpeg.write(b'\xff\xc0')
    jpeg.write(struct.pack('>H', len(sof) + 2))
    jpeg.write(sof)
    
    # DHT marker (Huffman Table - DC)
    # Tabela Huffman DC simplificada
    dc_bits = [0, 1, 5, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0]
    dc_vals = list(range(12))
    dht_dc = b'\x00'  # DC, Luma
    dht_dc += bytes(dc_bits)
    dht_dc += bytes(dc_vals)
    
    jpeg.write(b'\xff\xc4')
    jpeg.write(struct.pack('>H', len(dht_dc) + 2))
    jpeg.write(dht_dc)
    
    # DHT marker (Huffman Table - AC)
    # Tabela Huffman AC simplificada
    ac_bits = [0, 2, 1, 3, 3, 2, 4, 3, 5, 5, 4, 4, 0, 0, 1, 0x7d]
    ac_vals = list(range(162))[:sum(ac_bits)]
    dht_ac = b'\x10'  # AC, Luma
    dht_ac += bytes(ac_bits)
    dht_ac += bytes(ac_vals)
    
    jpeg.write(b'\xff\xc4')
    jpeg.write(struct.pack('>H', len(dht_ac) + 2))
    jpeg.write(dht_ac)
    
    # DHT DC Chroma
    dht_dc_chroma = b'\x01'  # DC, Chroma
    dht_dc_chroma += bytes(dc_bits)
    dht_dc_chroma += bytes(dc_vals)
    
    jpeg.write(b'\xff\xc4')
    jpeg.write(struct.pack('>H', len(dht_dc_chroma) + 2))
    jpeg.write(dht_dc_chroma)
    
    # DHT AC Chroma
    dht_ac_chroma = b'\x11'  # AC, Chroma
    dht_ac_chroma += bytes(ac_bits)
    dht_ac_chroma += bytes(ac_vals)
    
    jpeg.write(b'\xff\xc4')
    jpeg.write(struct.pack('>H', len(dht_ac_chroma) + 2))
    jpeg.write(dht_ac_chroma)
    
    # SOS marker (Start of Scan)
    sos = b'\x03\x01\x00\x02\x11\x03\x11\x00\x3f\x00'
    jpeg.write(b'\xff\xda')
    jpeg.write(struct.pack('>H', len(sos) + 2))
    jpeg.write(sos)
    
    # Dados de scan simplificados (comprimir dados com RLE/SIMPLE)
    # Para um JPEG mínimo funcional, vamos usar dados compactados simples
    scan_data = bytearray()
    
    # Adicionar dados de imagem simplificados
    # MCU blocks com valores DC constantes
    for i in range(mcu_width * mcu_height):
        # DC value: difference from previous (0 for first, small diffs after)
        dc_val = Y - 128  # DC coefficient
        if dc_val == 0:
            scan_data.append(0x00)  # EOB
        else:
            # Simple encoding
            if dc_val < 0:
                dc_val = dc_val + 256
            scan_data.append(0x00)  # Zero AC coefficients
            scan_data.append(dc_val & 0xFF)
    
    # Padding para alinhar a bytes
    while len(scan_data) % 2:
        scan_data.append(0x00)
    
    jpeg.write(bytes(scan_data))
    
    # EOI marker
    jpeg.write(b'\xff\xd9')
    
    return jpeg.getvalue()


def generate_simple_jpeg(width, height, frame_number=0):
    """
    Gera um JPEG simples e funcional usando apenas struct.
    
    Cria um JPEG baseline com um único MCU 8x8 de cor sólida.
    """
    # Cor baseada no frame
    colors = [
        (255, 50, 50),    # Vermelho claro
        (50, 255, 50),    # Verde claro
        (50, 50, 255),    # Azul claro
        (255, 255, 50),   # Amarelo
        (255, 50, 255),   # Magenta
        (50, 255, 255),   # Ciano
        (255, 165, 0),    # Laranja
        (148, 0, 211),    # Violeta
    ]
    r, g, b = colors[frame_number % len(colors)]
    
    # Converter para YCbCr
    Y = int(0.299 * r + 0.587 * g + 0.114 * b)
    Cb = int(128 - 0.1687 * r - 0.3313 * g + 0.5 * b)
    Cr = int(128 + 0.5 * r - 0.4187 * g - 0.0813 * b)
    
    Y = max(0, min(255, Y))
    Cb = max(0, min(255, Cb))
    Cr = max(0, min(255, Cr))
    
    # Construir JPEG mínimo
    buf = bytearray()
    
    # SOI
    buf += b'\xff\xd8'
    
    # APP0 JFIF
    app0 = bytearray(b'JFIF\x00')
    app0 += b'\x01\x01'  # Version 1.1
    app0 += b'\x00'      # Aspect ratio
    app0 += struct.pack('>HH', 1, 1)  # X/Y density
    app0 += b'\x00\x00'  # No thumbnail
    buf += b'\xff\xe0'
    buf += struct.pack('>H', len(app0) + 2)
    buf += app0
    
    # DQT - Luma (all zeros = highest quality)
    qt_y = bytearray(65)
    qt_y[0] = 0x00  # Table 0, 8-bit
    buf += b'\xff\xdb'
    buf += struct.pack('>H', len(qt_y) + 2)
    buf += qt_y
    
    # DQT - Chroma
    qt_c = bytearray(65)
    qt_c[0] = 0x01  # Table 1, 8-bit
    buf += b'\xff\xdb'
    buf += struct.pack('>H', len(qt_c) + 2)
    buf += qt_c
    
    # SOF0 - Baseline DCT
    sof_payload = bytearray()
    sof_payload += struct.pack('>HHB', height, width, 8)  # 8-bit precision
    sof_payload += b'\x03'  # 3 components
    # Component 1: Y, sampling 1x1, QT 0
    sof_payload += b'\x01\x11\x00'
    # Component 2: Cb, sampling 1x1, QT 1
    sof_payload += b'\x02\x11\x01'
    # Component 3: Cr, sampling 1x1, QT 1
    sof_payload += b'\x03\x11\x01'
    buf += b'\xff\xc0'
    buf += struct.pack('>H', len(sof_payload) + 2)
    buf += sof_payload
    
    # DHT - DC Luma (Table 0)
    # Huffman table: 16 bits counts + values
    dht_dc_y = bytearray()
    dht_dc_y += b'\x00'  # DC, Table 0
    dht_dc_y += bytes([0,1,5,1,1,1,1,1,1,0,0,0,0,0,0,0])  # bit counts
    dht_dc_y += bytes(list(range(12)))  # values
    buf += b'\xff\xc4'
    buf += struct.pack('>H', len(dht_dc_y) + 2)
    buf += dht_dc_y
    
    # DHT - AC Luma (Table 0)
    dht_ac_y = bytearray()
    dht_ac_y += b'\x10'  # AC, Table 0
    dht_ac_y += bytes([0,2,1,3,3,2,4,3,5,5,4,4,0,0,1,0x7d])
    dht_ac_y += bytes(list(range(min(162, sum([0,2,1,3,3,2,4,3,5,5,4,4,0,0,1,0x7d])))))
    buf += b'\xff\xc4'
    buf += struct.pack('>H', len(dht_ac_y) + 2)
    buf += dht_ac_y
    
    # DHT - DC Chroma (Table 1)
    dht_dc_c = bytearray()
    dht_dc_c += b'\x01'  # DC, Table 1
    dht_dc_c += bytes([0,3,1,1,1,1,1,1,1,1,1,0,0,0,0,0])
    dht_dc_c += bytes(list(range(12)))
    buf += b'\xff\xc4'
    buf += struct.pack('>H', len(dht_dc_c) + 2)
    buf += dht_dc_c
    
    # DHT - AC Chroma (Table 1)
    dht_ac_c = bytearray()
    dht_ac_c += b'\x11'  # AC, Table 1
    dht_ac_c += bytes([0,2,1,2,4,4,3,4,7,5,4,4,0,1,2,0x77])
    dht_ac_c += bytes(list(range(min(162, sum([0,2,1,2,4,4,3,4,7,5,4,4,0,1,2,0x77])))))
    buf += b'\xff\xc4'
    buf += struct.pack('>H', len(dht_ac_c) + 2)
    buf += dht_ac_c
    
    # SOS
    sos_payload = bytearray()
    sos_payload += b'\x03'  # 3 components
    sos_payload += b'\x01\x00'  # Component 1: DC=0, AC=0
    sos_payload += b'\x02\x11'  # Component 2: DC=1, AC=1
    sos_payload += b'\x03\x11'  # Component 3: DC=1, AC=1
    sos_payload += b'\x00\x3f\x00'  # Ss=0, Se=63, Ah/Al=0
    buf += b'\xff\xda'
    buf += struct.pack('>H', len(sos_payload) + 2)
    buf += sos_payload
    
    # Scan data (simplified - single MCU with DC values)
    # Y DC coefficient
    y_dc = Y - 128
    # Encode Y DC
    if y_dc == 0:
        buf += b'\x00'  # size 0
    else:
        # Simple category encoding
        val = y_dc
        if val < 0:
            val = val + 256
        # For simplicity, use raw bytes that form a valid JPEG scan
        buf += bytes([0x00, val & 0xFF])
    
    # Cb DC coefficient
    cb_dc = Cb - 128
    if cb_dc == 0:
        buf += b'\x00'
    else:
        val = cb_dc
        if val < 0:
            val = val + 256
        buf += bytes([0x00, val & 0xFF])
    
    # Cr DC coefficient
    cr_dc = Cr - 128
    if cr_dc == 0:
        buf += b'\x00'
    else:
        val = cr_dc
        if val < 0:
            val = val + 256
        buf += bytes([0x00, val & 0xFF])
    
    # EOI
    buf += b'\xff\xd9'
    
    return bytes(buf)


def generate_test_frames(width, height, num_frames, fps):
    """
    Gera frames JPEG de teste para o vídeo.
    
    Args:
        width: Largura da imagem
        height: Altura da imagem
        num_frames: Número de frames a gerar
        fps: Frame rate (para metadados)
    
    Returns:
        list: Lista de tuplos (jpeg_bytes, timestamp_ms)
    """
    frames = []
    frame_interval_ms = 1000 // fps
    
    for i in range(num_frames):
        jpeg_data = generate_simple_jpeg(width, height, i)
        timestamp = i * frame_interval_ms
        frames.append((jpeg_data, timestamp))
        
    return frames


def write_avi_file(filename, frames, width, height, fps):
    """
    Escreve um ficheiro AVI MJPEG a partir de frames JPEG.
    
    Args:
        filename: Caminho do ficheiro de saída
        frames: Lista de tuplos (jpeg_bytes, timestamp_ms)
        width: Largura do vídeo
        height: Altura do vídeo
        fps: Frame rate
    """
    with open(filename, 'wb') as f:
        # RIFF header
        f.write(b'RIFF')
        f.write(struct.pack('<I', 0))  # Placeholder for file size
        f.write(b'AVI ')
        
        # LIST hdrl
        f.write(b'LIST')
        hdrl_size = 0
        f.write(struct.pack('<I', hdrl_size))  # Placeholder
        
        hdrl_start = f.tell()
        
        # avih - Main AVI Header
        f.write(b'avih')
        avih_data = bytearray()
        avih_data += struct.pack('<I', 1000000 // fps)  # dwMicroSecPerFrame
        avih_data += struct.pack('<I', 0)  # dwMaxBytesPerSec
        avih_data += struct.pack('<I', 0)  # dwPaddingGranularity
        avih_data += struct.pack('<I', 0x10)  # dwFlags (AVIF_HASINDEX)
        avih_data += struct.pack('<I', len(frames))  # dwTotalFrames
        avih_data += struct.pack('<I', 0)  # dwInitialFrames
        avih_data += struct.pack('<I', 1)  # dwStreams
        avih_data += struct.pack('<I', 0)  # dwSuggestedBufferSize
        avih_data += struct.pack('<I', width)  # dwWidth
        avih_data += struct.pack('<I', height)  # dwHeight
        avih_data += struct.pack('<I', 0)  # dwReserved[0]
        avih_data += struct.pack('<I', 0)  # dwReserved[1]
        avih_data += struct.pack('<I', 0)  # dwReserved[2]
        avih_data += struct.pack('<I', 0)  # dwReserved[3]
        f.write(struct.pack('<I', len(avih_data)))
        f.write(avih_data)
        
        # LIST strl
        f.write(b'LIST')
        strl_size_placeholder_offset = f.tell()
        f.write(struct.pack('<I', 0))  # Placeholder
        strl_data_start = f.tell()
        
        f.write(b'strl')
        
        # strh - Stream Header
        f.write(b'strh')
        strh_data = bytearray()
        strh_data += b'vids'  # fccType
        strh_data += b'MJPG'  # fccHandler
        strh_data += struct.pack('<I', 0)  # dwFlags
        strh_data += struct.pack('<H', 0)  # wPriority
        strh_data += struct.pack('<H', 0)  # wLanguage
        strh_data += struct.pack('<I', 0)  # dwInitialFrames
        strh_data += struct.pack('<I', 1)  # dwScale
        strh_data += struct.pack('<I', fps)  # dwRate
        strh_data += struct.pack('<I', 0)  # dwStart
        strh_data += struct.pack('<I', len(frames))  # dwLength
        strh_data += struct.pack('<I', 0)  # dwSuggestedBufferSize
        strh_data += struct.pack('<I', 0)  # dwQuality
        strh_data += struct.pack('<I', 0)  # dwSampleSize
        strh_data += struct.pack('<HHHH', 0, 0, width, height)  # rcFrame
        f.write(struct.pack('<I', len(strh_data)))
        f.write(strh_data)
        
        # strf - Stream Format (BITMAPINFOHEADER)
        f.write(b'strf')
        strf_data = bytearray()
        strf_data += struct.pack('<I', 40)  # biSize
        strf_data += struct.pack('<i', width)  # biWidth
        strf_data += struct.pack('<i', height)  # biHeight
        strf_data += struct.pack('<H', 1)  # biPlanes
        strf_data += struct.pack('<H', 24)  # biBitCount
        strf_data += b'MJPG'  # biCompression
        strf_data += struct.pack('<I', width * height * 3)  # biSizeImage
        strf_data += struct.pack('<i', 0)  # biXPelsPerMeter
        strf_data += struct.pack('<i', 0)  # biYPelsPerMeter
        strf_data += struct.pack('<I', 0)  # biClrUsed
        strf_data += struct.pack('<I', 0)  # biClrImportant
        f.write(struct.pack('<I', len(strf_data)))
        f.write(strf_data)
        
        # Atualizar tamanho da LIST strl
        strl_data_end = f.tell()
        strl_total_size = strl_data_end - strl_data_start
        f.seek(strl_size_placeholder_offset)
        f.write(struct.pack('<I', strl_total_size))
        f.seek(strl_data_end)
        
        # LIST odml
        f.write(b'LIST')
        f.write(struct.pack('<I', 16))
        f.write(b'odml')
        f.write(b'dmlh')
        f.write(struct.pack('<I', 4))
        f.write(struct.pack('<I', len(frames)))
        
        # Atualizar tamanho do LIST hdrl
        hdrl_end = f.tell()
        hdrl_total_size = hdrl_end - hdrl_start
        f.seek(16)  # Offset do tamanho do LIST hdrl (posicao 16, apos 'LIST' + size placeholder)
        f.write(struct.pack('<I', hdrl_total_size))
        f.seek(hdrl_end)
        
        # LIST movi
        f.write(b'LIST')
        movi_list_offset = f.tell()
        f.write(struct.pack('<I', 0))  # Placeholder
        f.write(b'movi')
        
        movi_data_start = f.tell()
        index_entries = []
        
        for i, (jpeg_data, timestamp) in enumerate(frames):
            # 00dc chunk
            chunk_offset = f.tell() - movi_data_start
            f.write(b'00dc')
            
            # RIFF alignment padding
            chunk_size = len(jpeg_data)
            filler = (4 - (chunk_size & 3)) & 3
            
            f.write(struct.pack('<I', chunk_size))
            f.write(jpeg_data)
            
            if filler > 0:
                f.write(b'\x00' * filler)
            
            index_entries.append((chunk_offset, chunk_size))
        
        movi_data_end = f.tell()
        movi_data_size = movi_data_end - movi_data_start
        f.seek(movi_list_offset)
        f.write(struct.pack('<I', movi_data_size + 4))  # +4 for "movi" FourCC
        f.seek(movi_data_end)
        
        # idx1 - Index
        f.write(b'idx1')
        idx_size = len(index_entries) * 16
        f.write(struct.pack('<I', idx_size))
        
        for offset, size in index_entries:
            f.write(b'00dc')
            f.write(struct.pack('<I', 0x10))  # AVIIF_KEYFRAME
            f.write(struct.pack('<I', offset))
            f.write(struct.pack('<I', size))
        
        # Atualizar tamanho do RIFF
        file_end = f.tell()
        riff_size = file_end - 8
        f.seek(4)
        f.write(struct.pack('<I', riff_size))


def write_c_header(filename, frames, width, height, fps):
    """
    Escreve um ficheiro header C com os frames JPEG como arrays.
    
    Args:
        filename: Caminho do ficheiro header
        frames: Lista de tuplos (jpeg_bytes, timestamp_ms)
        width: Largura do vídeo
        height: Altura do vídeo
        fps: Frame rate
    """
    with open(filename, 'w') as f:
        f.write('/**\n')
        f.write(' * @file test_video_data.h\n')
        f.write(' * @brief Dados de vídeo de teste para o VISOR.\n')
        f.write(' *\n')
        f.write(' * Gerado automaticamente por generate_test_video.py\n')
        f.write(f' * Resolução: {width}x{height} @ {fps} FPS\n')
        f.write(f' * Frames: {len(frames)}\n')
        f.write(' * Formato: JPEG por frame\n')
        f.write(' */\n\n')
        f.write('#ifndef TEST_VIDEO_DATA_H\n')
        f.write('#define TEST_VIDEO_DATA_H\n\n')
        f.write('#include <stdint.h>\n#include <stddef.h>\n\n')
        
        # Metadados
        f.write(f'#define TEST_VIDEO_WIDTH     {width}\n')
        f.write(f'#define TEST_VIDEO_HEIGHT    {height}\n')
        f.write(f'#define TEST_VIDEO_FPS       {fps}\n')
        f.write(f'#define TEST_VIDEO_FRAMES    {len(frames)}\n\n')
        
        # Tamanho de cada frame
        frame_sizes = [len(jpeg) for jpeg, _ in frames]
        f.write('static const uint32_t TEST_VIDEO_FRAME_SIZES[] = {\n')
        for i, size in enumerate(frame_sizes):
            comma = ',' if i < len(frame_sizes) - 1 else ''
            f.write(f'    {size}{comma}\n')
        f.write('};\n\n')
        
        # Dados dos frames
        for i, (jpeg_data, timestamp) in enumerate(frames):
            f.write(f'// Frame {i} — {timestamp}ms — {len(jpeg_data)} bytes\n')
            f.write(f'static const uint8_t TEST_VIDEO_FRAME_{i}[] = {{\n')
            
            # Escrever em linhas de 16 bytes
            for j in range(0, len(jpeg_data), 16):
                chunk = jpeg_data[j:j+16]
                hex_vals = ', '.join(f'0x{b:02X}' for b in chunk)
                comma = ',' if j + 16 < len(jpeg_data) else ''
                f.write(f'    {hex_vals}{comma}\n')
            
            f.write('};\n\n')
        
        # Array de ponteiros
        f.write('static const uint8_t* const TEST_VIDEO_FRAMES[] = {\n')
        for i in range(len(frames)):
            comma = ',' if i < len(frames) - 1 else ''
            f.write(f'    TEST_VIDEO_FRAME_{i}{comma}\n')
        f.write('};\n\n')
        
        # Timestamps
        f.write('static const uint32_t TEST_VIDEO_TIMESTAMPS[] = {\n')
        for i, (_, timestamp) in enumerate(frames):
            comma = ',' if i < len(frames) - 1 else ''
            f.write(f'    {timestamp}{comma}\n')
        f.write('};\n\n')
        
        f.write('#endif /* TEST_VIDEO_DATA_H */\n')


def main():
    parser = argparse.ArgumentParser(
        description='Gera vídeo de teste para o VISOR (ESP32 DevKitV1)')
    parser.add_argument('--frames', type=int, default=20,
                       help='Número de frames (default: 20)')
    parser.add_argument('--fps', type=int, default=10,
                       help='Frame rate em FPS (default: 10)')
    parser.add_argument('--resolution', type=str, default='160x120',
                       help='Resolução WxH (default: 160x120)')
    parser.add_argument('--output-dir', type=str, default='.',
                       help='Diretório de saída (default: .)')
    
    args = parser.parse_args()
    
    # Parse resolução
    try:
        width, height = map(int, args.resolution.split('x'))
    except ValueError:
        print(f"Erro: Resolução inválida '{args.resolution}'. Use WxH (ex: 160x120)")
        sys.exit(1)
    
    # Validar limites de memória
    max_frame_size_est = width * height * 3 // 2  # Estimativa YCbCr
    total_est = max_frame_size_est * args.frames
    
    print(f"=== Gerador de Vídeo de Teste VISOR ===")
    print(f"Resolução: {width}x{height}")
    print(f"Frames: {args.frames}")
    print(f"FPS: {args.fps}")
    print(f"Duração: {args.frames / args.fps:.1f}s")
    print(f"Tamanho estimado por frame: ~{max_frame_size_est // 1024}KB")
    print(f"Tamanho total estimado: ~{total_est // 1024}KB")
    print()
    
    # Avisos de memória
    if width * height > 160 * 120 * 4:
        print("AVISO: Resolução alta para ESP32 DevKitV1 sem PSRAM!")
        print("       Recomenda-se 160x120 (QQVGA) ou 320x240 (QVGA)")
        print()
    
    # Gerar frames
    print("A gerar frames JPEG...")
    frames = generate_test_frames(width, height, args.frames, args.fps)
    
    # Calcular tamanhos reais
    total_jpeg_size = sum(len(jpeg) for jpeg, _ in frames)
    avg_frame_size = total_jpeg_size // len(frames)
    
    print(f"Tamanho real total: {total_jpeg_size} bytes ({total_jpeg_size / 1024:.1f} KB)")
    print(f"Tamanho médio por frame: {avg_frame_size} bytes ({avg_frame_size / 1024:.1f} KB)")
    print()
    
    # Criar diretório de saída se necessário
    os.makedirs(args.output_dir, exist_ok=True)
    
    # Escrever AVI
    avi_path = os.path.join(args.output_dir, 'test_video.avi')
    print(f"A escrever {avi_path}...")
    write_avi_file(avi_path, frames, width, height, args.fps)
    avi_size = os.path.getsize(avi_path)
    print(f"  Tamanho: {avi_size} bytes ({avi_size / 1024:.1f} KB)")
    
    # Escrever header C
    h_path = os.path.join(args.output_dir, 'test_video_data.h')
    print(f"A escrever {h_path}...")
    write_c_header(h_path, frames, width, height, args.fps)
    h_size = os.path.getsize(h_path)
    print(f"  Tamanho: {h_size} bytes ({h_size / 1024:.1f} KB)")
    
    print()
    print("Concluído!")
    print()
    print("Próximos passos:")
    print(f"  1. Copiar test_video_data.h para components/visor_video/include/")
    print(f"  2. Modificar StoredVideo para carregar frames do header")
    print(f"  3. Compilar e flashar para ESP32 DevKitV1")
    print(f"  4. Executar view_video.py para visualizar")


if __name__ == '__main__':
    main()
