/**
 * @file AviMjpegWriter.cpp
 * @brief Implementação do escritor de container AVI MJPEG.
 *
 * Implementa a criação de ficheiros AVI contendo vídeo MJPEG.
 * O AVI é construído em memória (buffer PSRAM) e pode ser
 * finalizado com o cabeçalho correto e índice.
 *
 * Estrutura AVI gerada:
 * ```
 * RIFF 'AVI '
 *   LIST 'hdrl'
 *     'avih' (Main AVI Header - 56 bytes)
 *     LIST 'strl'
 *       'strh' (Stream Header - 64 bytes, FourCC: MJPG)
 *       'strf' (Stream Format - BITMAPINFOHEADER)
 *     LIST 'odml'
 *       'dmlh' (Extended Header)
 *   LIST 'movi'
 *     '00dc' + data (para cada frame)
 *   'idx1' (Index)
 * ```
 *
 * @version 2.0.0
 * @author ShegaPT
 * @license GPLv3
 */

#include "AviMjpegWriter.h"

#ifdef ESP32
#include "esp_log.h"
#include <cstring>
#include <cstdlib>

static const char* TAG = "AviMjpegWriter";

// Tamanhos dos componentes AVI
#define AVI_RIFF_HEADER_SIZE    12
#define AVI_HDRL_LIST_SIZE      (12 + 76 + 132 + 28)  // ~248
#define AVI_MOVI_LIST_HEADER    12
#define AVI_CHUNK_HEADER_SIZE   8   // "00dc" + size
#define AVI_INDEX_ENTRY_SIZE    16  // Por frame no idx1

// Buffer padrão: 2MB (suficiente para ~60 segundos VGA)
#define DEFAULT_BUFFER_SIZE     (2 * 1024 * 1024)
#define DEFAULT_INDEX_SIZE      1024

AviMjpegWriter::AviMjpegWriter()
    : _initialized(false)
    , _width(640)
    , _height(480)
    , _fps(30)
    , _frameCount(0)
    , _buffer(nullptr)
    , _bufferCapacity(0)
    , _offset(0)
    , _moviListOffset(0)
    , _index(nullptr)
    , _indexCount(0)
    , _indexCapacity(0)
{
}

AviMjpegWriter::~AviMjpegWriter() {
    if (_buffer) {
        free(_buffer);
        _buffer = nullptr;
    }
    if (_index) {
        free(_index);
        _index = nullptr;
    }
}

bool AviMjpegWriter::begin(uint16_t width, uint16_t height, uint32_t fps) {
    _width = width;
    _height = height;
    _fps = fps;
    _frameCount = 0;
    _offset = 0;
    _indexCount = 0;

    // Alocar buffer
    if (!_buffer) {
        _buffer = (uint8_t*)heap_caps_malloc(DEFAULT_BUFFER_SIZE, MALLOC_CAP_SPIRAM);
        if (!_buffer) {
            _buffer = (uint8_t*)malloc(DEFAULT_BUFFER_SIZE);
        }
        if (!_buffer) {
            ESP_LOGE(TAG, "Falha ao alocar buffer AVI");
            return false;
        }
        _bufferCapacity = DEFAULT_BUFFER_SIZE;
    }

    // Alocar índice
    if (!_index) {
        _indexCapacity = DEFAULT_INDEX_SIZE;
        _index = (AviIndexEntry*)heap_caps_malloc(
            _indexCapacity * sizeof(AviIndexEntry), MALLOC_CAP_SPIRAM);
        if (!_index) {
            _index = (AviIndexEntry*)malloc(_indexCapacity * sizeof(AviIndexEntry));
        }
        if (!_index) {
            ESP_LOGE(TAG, "Falha ao alocar índice AVI");
            free(_buffer);
            _buffer = nullptr;
            return false;
        }
    }

    _initialized = true;
    _writeHeader();
    ESP_LOGI(TAG, "AviMjpegWriter inicializado (%ux%u @ %u fps)", width, height, fps);
    return true;
}

bool AviMjpegWriter::writeFrame(const uint8_t* jpegData, size_t jpegLen) {
    if (!_initialized || !jpegData || jpegLen == 0) {
        return false;
    }

    // Calcular tamanho com padding (RIFF exige alinhamento de 2 bytes)
    uint16_t filler = (4 - (jpegLen & 3)) & 3;
    size_t chunkSize = jpegLen + filler;

    // Verificar espaço no buffer
    if (!_ensureCapacity(AVI_CHUNK_HEADER_SIZE + chunkSize)) {
        ESP_LOGE(TAG, "Buffer AVI cheio");
        return false;
    }

    // Guardar offset para índice
    size_t frameOffset = _offset - _moviListOffset - 8;  // Offset relativo ao início dos dados 'movi'

    // Escrever chunk header: "00dc" + size
    _writeBytes((const uint8_t*)"00dc", 4);
    _writeDword((uint32_t)jpegLen);  // Size sem padding

    // Escrever dados JPEG
    _writeBytes(jpegData, jpegLen);

    // Escrever padding
    if (filler > 0) {
        uint8_t pad[3] = {0, 0, 0};
        _writeBytes(pad, filler);
    }

    // Atualizar índice
    if (_indexCount >= _indexCapacity) {
        _indexCapacity *= 2;
        AviIndexEntry* newIndex = (AviIndexEntry*)realloc(_index,
            _indexCapacity * sizeof(AviIndexEntry));
        if (!newIndex) {
            ESP_LOGE(TAG, "Falha ao redimensionar índice");
            return false;
        }
        _index = newIndex;
    }
    _index[_indexCount].offset = (uint32_t)frameOffset;
    _index[_indexCount].size = (uint32_t)jpegLen;
    _indexCount++;
    _frameCount++;

    return true;
}

bool AviMjpegWriter::finalize(uint8_t* output, size_t* outputSize) {
    if (!_initialized || !output || !outputSize) {
        return false;
    }

    // Calcular tamanho total
    size_t indexSize = _indexCount * AVI_INDEX_ENTRY_SIZE + 8; // idx1 header + entries
    size_t totalSize = _offset + indexSize;

    if (totalSize > *outputSize) {
        ESP_LOGE(TAG, "Buffer de saída insuficiente: %zu < %zu", *outputSize, totalSize);
        return false;
    }

    // Escrever índice
    _writeBytes((const uint8_t*)"idx1", 4);
    _writeDword((uint32_t)(_indexCount * AVI_INDEX_ENTRY_SIZE));

    for (uint32_t i = 0; i < _indexCount; i++) {
        _writeBytes((const uint8_t*)"00dc", 4);  // ckid
        _writeDword(0x10);                         // dwFlags (AVIIF_KEYFRAME)
        _writeDword(_index[i].offset);             // dwOffset
        _writeDword(_index[i].size);               // dwSize
    }

    // Atualizar cabeçalho com contagem final de frames
    // avih dwTotalFrames está em offset 20 do RIFF
    if (_buffer && _offset >= 24) {
        _buffer[20] = (uint8_t)(_frameCount & 0xFF);
        _buffer[21] = (uint8_t)((_frameCount >> 8) & 0xFF);
        _buffer[22] = (uint8_t)((_frameCount >> 16) & 0xFF);
        _buffer[23] = (uint8_t)((_frameCount >> 24) & 0xFF);

        // strh dwLength está no offset correspondente
        // Atualizar também o tamanho total do RIFF
        uint32_t riffSize = (uint32_t)(totalSize - 8);
        _buffer[4] = (uint8_t)(riffSize & 0xFF);
        _buffer[5] = (uint8_t)((riffSize >> 8) & 0xFF);
        _buffer[6] = (uint8_t)((riffSize >> 16) & 0xFF);
        _buffer[7] = (uint8_t)((riffSize >> 24) & 0xFF);
    }

    // Copiar para output
    memcpy(output, _buffer, _offset);
    *outputSize = _offset;

    ESP_LOGI(TAG, "AVI finalizado: %lu frames, %zu bytes", _frameCount, _offset);
    return true;
}

void AviMjpegWriter::reset() {
    _frameCount = 0;
    _offset = 0;
    _indexCount = 0;
    if (_initialized) {
        _writeHeader();
    }
}

size_t AviMjpegWriter::getCurrentSize() const {
    return _offset;
}

uint32_t AviMjpegWriter::getFrameCount() const {
    return _frameCount;
}

uint16_t AviMjpegWriter::getWidth() const { return _width; }
uint16_t AviMjpegWriter::getHeight() const { return _height; }
uint32_t AviMjpegWriter::getFps() const { return _fps; }

bool AviMjpegWriter::_ensureCapacity(size_t additionalBytes) {
    if (_offset + additionalBytes <= _bufferCapacity) {
        return true;
    }
    size_t newSize = _bufferCapacity * 2;
    while (newSize < _offset + additionalBytes) {
        newSize *= 2;
    }
    uint8_t* newBuffer = (uint8_t*)realloc(_buffer, newSize);
    if (!newBuffer) return false;
    _buffer = newBuffer;
    _bufferCapacity = newSize;
    return true;
}

void AviMjpegWriter::_writeHeader() {
    if (!_buffer) return;
    _offset = 0;

    // RIFF header
    _writeBytes((const uint8_t*)"RIFF", 4);
    _writeDword(0);  // Placeholder para tamanho total
    _writeBytes((const uint8_t*)"AVI ", 4);

    // hdrl LIST
    _writeBytes((const uint8_t*)"LIST", 4);
    _writeDword(228);  // Tamanho do hdrl
    _writeBytes((const uint8_t*)"hdrl", 4);

    // avih (Main AVI Header) — 56 bytes de dados
    _writeBytes((const uint8_t*)"avih", 4);
    _writeDword(56);  // Tamanho do chunk
    _writeDword(1000000 / _fps);  // dwMicroSecPerFrame
    _writeDword(0);                // dwMaxBytesPerSec
    _writeDword(0);                // dwPaddingGranularity
    _writeDword(0x10);             // dwFlags (AVIF_HASINDEX)
    _writeDword(0);                // dwTotalFrames (placeholder)
    _writeDword(0);                // dwInitialFrames
    _writeDword(1);                // dwStreams
    _writeDword(0);                // dwSuggestedBufferSize
    _writeDword(_width);           // dwWidth
    _writeDword(_height);          // dwHeight
    _writeDword(0);                // dwReserved[0]
    _writeDword(0);                // dwReserved[1]
    _writeDword(0);                // dwReserved[2]
    _writeDword(0);                // dwReserved[3]

    // strl LIST
    _writeBytes((const uint8_t*)"LIST", 4);
    _writeDword(120);  // Tamanho do strl
    _writeBytes((const uint8_t*)"strl", 4);

    // strh (Stream Header) — 56 bytes de dados
    _writeBytes((const uint8_t*)"strh", 4);
    _writeDword(56);
    _writeBytes((const uint8_t*)"vids", 4);  // fccType
    _writeBytes((const uint8_t*)"MJPG", 4);  // fccHandler
    _writeDword(0);                // dwFlags
    _writeWord(0);                 // wPriority
    _writeWord(0);                 // wLanguage
    _writeDword(0);                // dwInitialFrames
    _writeDword(1);                // dwScale
    _writeDword(_fps);             // dwRate
    _writeDword(0);                // dwStart
    _writeDword(0);                // dwLength (placeholder)
    _writeDword(0);                // dwSuggestedBufferSize
    _writeDword(0);                // dwQuality
    _writeDword(0);                // dwSampleSize
    _writeWord(0);                 // rcFrame.left
    _writeWord(0);                 // rcFrame.top
    _writeWord(_width);            // rcFrame.right
    _writeWord(_height);           // rcFrame.bottom

    // strf (Stream Format) — BITMAPINFOHEADER
    _writeBytes((const uint8_t*)"strf", 4);
    _writeDword(40);               // biSize
    _writeDword(_width);           // biWidth
    _writeDword(_height);          // biHeight
    _writeWord(1);                 // biPlanes
    _writeWord(24);                // biBitCount
    _writeBytes((const uint8_t*)"MJPG", 4);  // biCompression
    _writeDword(_width * _height * 3);  // biSizeImage
    _writeDword(0);                // biXPelsPerMeter
    _writeDword(0);                // biYPelsPerMeter
    _writeDword(0);                // biClrUsed
    _writeDword(0);                // biClrImportant

    // odml LIST
    _writeBytes((const uint8_t*)"LIST", 4);
    _writeDword(16);
    _writeBytes((const uint8_t*)"odml", 4);
    _writeBytes((const uint8_t*)"dmlh", 4);
    _writeDword(4);
    _writeDword(0);  // dwTotalFrames (placeholder)

    // movi LIST
    _moviListOffset = _offset;
    _writeBytes((const uint8_t*)"LIST", 4);
    _writeDword(0);  // Placeholder para tamanho
    _writeBytes((const uint8_t*)"movi", 4);
}

void AviMjpegWriter::_writeDword(uint32_t value) {
    if (_offset + 4 > _bufferCapacity) return;
    _buffer[_offset++] = (uint8_t)(value & 0xFF);
    _buffer[_offset++] = (uint8_t)((value >> 8) & 0xFF);
    _buffer[_offset++] = (uint8_t)((value >> 16) & 0xFF);
    _buffer[_offset++] = (uint8_t)((value >> 24) & 0xFF);
}

void AviMjpegWriter::_writeWord(uint16_t value) {
    if (_offset + 2 > _bufferCapacity) return;
    _buffer[_offset++] = (uint8_t)(value & 0xFF);
    _buffer[_offset++] = (uint8_t)((value >> 8) & 0xFF);
}

void AviMjpegWriter::_writeBytes(const uint8_t* data, size_t len) {
    if (_offset + len > _bufferCapacity) return;
    memcpy(_buffer + _offset, data, len);
    _offset += len;
}

void AviMjpegWriter::_writeString(const char* str, size_t len) {
    _writeBytes((const uint8_t*)str, len);
}

#else
// Stub para compilação sem ESP32
AviMjpegWriter::AviMjpegWriter() : _initialized(false), _width(640), _height(480),
    _fps(30), _frameCount(0), _buffer(nullptr), _bufferCapacity(0), _offset(0),
    _moviListOffset(0), _index(nullptr), _indexCount(0), _indexCapacity(0) {}
AviMjpegWriter::~AviMjpegWriter() { if(_buffer) free(_buffer); if(_index) free(_index); }
bool AviMjpegWriter::begin(uint16_t w, uint16_t h, uint32_t f) { _width=w; _height=h; _fps=f; return true; }
bool AviMjpegWriter::writeFrame(const uint8_t*, size_t) { return false; }
bool AviMjpegWriter::finalize(uint8_t*, size_t*) { return false; }
void AviMjpegWriter::reset() {}
size_t AviMjpegWriter::getCurrentSize() const { return 0; }
uint32_t AviMjpegWriter::getFrameCount() const { return 0; }
uint16_t AviMjpegWriter::getWidth() const { return _width; }
uint16_t AviMjpegWriter::getHeight() const { return _height; }
uint32_t AviMjpegWriter::getFps() const { return _fps; }
bool AviMjpegWriter::_ensureCapacity(size_t) { return false; }
void AviMjpegWriter::_writeHeader() {}
void AviMjpegWriter::_writeDword(uint32_t) {}
void AviMjpegWriter::_writeWord(uint16_t) {}
void AviMjpegWriter::_writeBytes(const uint8_t*, size_t) {}
void AviMjpegWriter::_writeString(const char*, size_t) {}
#endif
