/**
 * @file Video.cpp
 * @brief Implementação do módulo principal de vídeo do VISOR.
 *
 * Coordena o pipeline completo de processamento de vídeo:
 * Camera → Processamento → AVI → Fragmentação TLV → Envio via Callback.
 *
 * Utiliza as funções FFI do protocolo Rust para serialização TLV.
 *
 * @version 2.0.0
 * @author ShegaPT
 * @license GPLv3
 */

#include "Video.h"

#ifdef ESP32
#include "esp_log.h"
#include "esp_timer.h"
#include <cstring>

static const char* TAG = "Video";

Video::Video()
    : _camera(nullptr)
    , _transport({nullptr, nullptr})
    , _nextFrameId(0)
    , _enabled(false)
    , _debug(false)
    , _status(VideoStatus::IDLE)
    , _framesProcessed(0)
    , _chunksSent(0)
    , _chunksDropped(0)
    , _chunksFailed(0)
{
    _queue.reserve(VIDEO_MAX_QUEUE_SIZE);
}

Video::~Video() {
    end();
}

bool Video::begin(Camera* camera, TransportInterface transport, const VideoConfig& config) {
    if (!camera) {
        ESP_LOGE(TAG, "Camera nula");
        return false;
    }

    if (!transport_is_valid(&transport)) {
        ESP_LOGE(TAG, "Interface de transporte inválida");
        return false;
    }

    _camera = camera;
    _transport = transport;

    // Inicializar processador
    if (!_processor.init(config)) {
        ESP_LOGE(TAG, "Falha ao inicializar processador");
        return false;
    }

    // Inicializar escritor AVI
    if (!_aviWriter.begin(config.targetWidth, config.targetHeight, 30)) {
        ESP_LOGE(TAG, "Falha ao inicializar escritor AVI");
        _processor.deinit();
        return false;
    }

    _enabled = true;
    _status = VideoStatus::IDLE;
    _nextFrameId = 0;
    _queue.clear();
    resetStats();

    ESP_LOGI(TAG, "Módulo Video inicializado (transport: %s)",
             transport.send ? "callback" : "none");
    return true;
}

bool Video::processAndSend() {
    if (!_enabled || !_camera || _status == VideoStatus::ERROR) {
        return false;
    }

    _status = VideoStatus::RECEIVING;

    // 1. Capturar frame da camera — alocar em heap ( VIDEO_MAX_FRAME_SIZE > stack )
    uint8_t* rawBuffer = (uint8_t*)malloc(VIDEO_MAX_FRAME_SIZE);
    if (!rawBuffer) {
        ESP_LOGE(TAG, "Falha ao alocar rawBuffer (%d bytes)", VIDEO_MAX_FRAME_SIZE);
        _status = VideoStatus::ERROR;
        return false;
    }
    size_t rawLen = VIDEO_MAX_FRAME_SIZE;

    if (!_camera->capture(rawBuffer, &rawLen)) {
        ESP_LOGE(TAG, "Falha ao capturar frame");
        free(rawBuffer);
        _status = VideoStatus::ERROR;
        return false;
    }

    // 2. Processar imagem
    _status = VideoStatus::PROCESSING;
    uint8_t* processedBuffer = (uint8_t*)malloc(VIDEO_MAX_FRAME_SIZE);
    if (!processedBuffer) {
        ESP_LOGE(TAG, "Falha ao alocar processedBuffer (%d bytes)", VIDEO_MAX_FRAME_SIZE);
        free(rawBuffer);
        _status = VideoStatus::ERROR;
        return false;
    }
    size_t processedLen = VIDEO_MAX_FRAME_SIZE;

    if (!_processor.processFrame(rawBuffer, rawLen, processedBuffer, &processedLen)) {
        ESP_LOGE(TAG, "Falha ao processar frame");
        free(rawBuffer);
        free(processedBuffer);
        _status = VideoStatus::ERROR;
        return false;
    }

    // 3. Empacotar em AVI
    if (!_aviWriter.writeFrame(processedBuffer, processedLen)) {
        ESP_LOGW(TAG, "Falha ao escrever frame no AVI (continuando)");
    }

    // 4. Fragmentar em chunks TLV
    uint16_t frameId = _nextFrameId++;
    if (_nextFrameId == 0) _nextFrameId = 1;  // Evitar 0

    if (!_fragmentFrame(processedBuffer, processedLen, frameId)) {
        ESP_LOGE(TAG, "Falha ao fragmentar frame");
        free(rawBuffer);
        free(processedBuffer);
        _status = VideoStatus::ERROR;
        return false;
    }

    _framesProcessed++;

    // 5. Enviar chunks
    _sendChunks();

    free(rawBuffer);
    free(processedBuffer);
    _status = VideoStatus::IDLE;
    return true;
}

bool Video::isEnabled() const {
    return _enabled;
}

void Video::setEnabled(bool enable) {
    _enabled = enable;
    if (!enable) {
        _queue.clear();
        _status = VideoStatus::IDLE;
    }
}

void Video::setDebug(bool enable) {
    _debug = enable;
}

uint32_t Video::getFramesProcessed() const {
    return _framesProcessed;
}

uint32_t Video::getChunksSent() const {
    return _chunksSent;
}

uint32_t Video::getChunksDropped() const {
    return _chunksDropped;
}

void Video::resetStats() {
    _framesProcessed = 0;
    _chunksSent = 0;
    _chunksDropped = 0;
    _chunksFailed = 0;
}

VideoStatus Video::getStatus() const {
    return _status;
}

size_t Video::getQueueSize() const {
    return _queue.size();
}

void Video::end() {
    _enabled = false;
    _queue.clear();
    _status = VideoStatus::IDLE;
    _camera = nullptr;
    _transport = {nullptr, nullptr};
    _processor.deinit();
    ESP_LOGI(TAG, "Módulo Video desligado");
}

/* ========================================================================
 * MÉTODOS PRIVADOS
 * ======================================================================== */

bool Video::_fragmentFrame(const uint8_t* data, size_t len, uint16_t frameId) {
    if (len == 0) {
        return false;
    }

    uint32_t numChunks = (len + VIDEO_MAX_CHUNK_SIZE - 1) / VIDEO_MAX_CHUNK_SIZE;

    for (uint32_t i = 0; i < numChunks; i++) {
        VideoChunk chunk;
        chunk.frameId = frameId;
        chunk.chunkId = (uint8_t)i;
        chunk.totalChunks = (uint8_t)numChunks;
        chunk.timestamp = (uint32_t)(esp_timer_get_time() / 1000);

        size_t offset = i * VIDEO_MAX_CHUNK_SIZE;
        size_t remaining = len - offset;
        chunk.dataLen = (remaining < VIDEO_MAX_CHUNK_SIZE) ? remaining : VIDEO_MAX_CHUNK_SIZE;
        memcpy(chunk.data, data + offset, chunk.dataLen);

        if (!_enqueueChunk(chunk)) {
            ESP_LOGW(TAG, "Fila cheia, chunk %lu/%lu descartado", i, numChunks);
            return (i > 0);  // Sucesso parcial se pelo menos 1 chunk foi enfileirado
        }
    }

    if (_debug) {
        ESP_LOGI(TAG, "Frame %u fragmentada em %lu chunks", frameId, numChunks);
    }

    return true;
}

bool Video::_sendChunk(const VideoChunk& chunk) {
    if (!_enabled) {
        return false;
    }

    uint8_t buffer[256];
    size_t len = _buildVideoPacket(chunk, buffer, sizeof(buffer));

    if (len == 0) {
        ESP_LOGE(TAG, "Falha ao construir pacote TLV");
        return false;
    }

    // Enviar via callback
    int result = transport_send(&_transport, buffer, len);

    if (result == 0) {
        _chunksSent++;
        if (_debug) {
            ESP_LOGI(TAG, "Chunk %u/%u enviado (%zu bytes)",
                     chunk.chunkId, chunk.totalChunks, len);
        }
        return true;
    } else {
        _chunksFailed++;
        ESP_LOGW(TAG, "Falha ao enviar chunk: %d", result);
        return false;
    }
}

void Video::_sendChunks() {
    _status = VideoStatus::SENDING;

    while (!_queue.empty()) {
        VideoChunk chunk = _queue.front();
        if (_sendChunk(chunk)) {
            _dequeueChunk();
        } else {
            break;  // Erro — manter chunk na fila
        }
    }

    _status = _queue.empty() ? VideoStatus::IDLE : VideoStatus::SENDING;
}

size_t Video::_buildVideoPacket(const VideoChunk& chunk, uint8_t* buffer, size_t size) {
    TLVMessage msg;
    visor_acp_init(&msg, ACP_GROUP_VISOR, ACP_MSG_VIDEO);
    visor_acp_set_seq(&msg, _nextFrameId);

    visor_add_tlv_uint16(&msg, ACP_FLD_VIDEO_FRAME_ID, chunk.frameId);
    visor_add_tlv_uint8(&msg, ACP_FLD_VIDEO_CHUNK_ID, chunk.chunkId);
    visor_add_tlv_uint8(&msg, ACP_FLD_VIDEO_TOTAL, chunk.totalChunks);
    visor_add_tlv(&msg, ACP_FLD_VIDEO_PAYLOAD, chunk.data, (uint8_t)chunk.dataLen);

    ssize_t result = visor_build_message(&msg, ACP_MSG_VIDEO, 0x00, buffer, size);
    return (result > 0) ? (size_t)result : 0;
}

bool Video::_enqueueChunk(const VideoChunk& chunk) {
    if (_queue.size() >= VIDEO_MAX_QUEUE_SIZE) {
        _chunksDropped++;
        return false;
    }
    _queue.push_back(chunk);
    return true;
}

void Video::_dequeueChunk() {
    if (!_queue.empty()) {
        _queue.erase(_queue.begin());
    }
}

#else
// Stub para compilação sem ESP32
Video::Video() : _camera(nullptr), _transport({nullptr, nullptr}),
    _nextFrameId(0), _enabled(false), _debug(false), _status(VideoStatus::IDLE),
    _framesProcessed(0), _chunksSent(0), _chunksDropped(0), _chunksFailed(0) {}
Video::~Video() { end(); }
bool Video::begin(Camera*, TransportInterface, const VideoConfig&) { return false; }
bool Video::processAndSend() { return false; }
bool Video::isEnabled() const { return false; }
void Video::setEnabled(bool) {}
void Video::setDebug(bool) {}
uint32_t Video::getFramesProcessed() const { return 0; }
uint32_t Video::getChunksSent() const { return 0; }
uint32_t Video::getChunksDropped() const { return 0; }
void Video::resetStats() {}
VideoStatus Video::getStatus() const { return VideoStatus::IDLE; }
size_t Video::getQueueSize() const { return 0; }
void Video::end() {}
bool Video::_fragmentFrame(const uint8_t*, size_t, uint16_t) { return false; }
bool Video::_sendChunk(const VideoChunk&) { return false; }
void Video::_sendChunks() {}
size_t Video::_buildVideoPacket(const VideoChunk&, uint8_t*, size_t) { return 0; }
bool Video::_enqueueChunk(const VideoChunk&) { return false; }
void Video::_dequeueChunk() {}
#endif
