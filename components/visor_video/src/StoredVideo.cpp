/**
 * @file StoredVideo.cpp
 * @brief Implementação do módulo de vídeo armazenado para testes.
 *
 * @version 2.0.0
 * @author ShegaPT
 * @license GPLv3
 */

#include "StoredVideo.h"

#ifdef ESP32
#include "esp_log.h"
#include <cstring>
#include <cmath>

static const char* TAG = "StoredVideo";

StoredVideo::StoredVideo()
    : _initialized(false)
    , _currentResolution(CameraResolution::VGA)
    , _currentFormat(CameraFormat::JPEG)
    , _currentQuality(10)
    , _maxFrameSize(30 * 1024)
    , _hasStaticImage(false)
    , _hasVideo(false)
    , _videoFrameCount(0)
    , _videoFps(10)
    , _currentFrameIndex(0)
    , _countdownValue(10)
    , _lastCountdownTime(0)
{
}

StoredVideo::~StoredVideo() {
    end();
}

bool StoredVideo::begin(const CameraPinConfig& pins) {
    (void)pins;  // Ignorado — não há hardware real
    _initialized = true;
    _currentFrameIndex = 0;
    _countdownValue = 10;
    _lastCountdownTime = 0;
    ESP_LOGI(TAG, "StoredVideo inicializado (modo teste)");
    return true;
}

bool StoredVideo::capture(uint8_t* buffer, size_t* length) {
    if (!_initialized || !buffer || !length) {
        return false;
    }

    if (_hasStaticImage) {
        // Modo imagem estática com contagem regressiva
        if (_staticImage.size() > *length) {
            ESP_LOGE(TAG, "Buffer insuficiente para imagem estática");
            return false;
        }
        memcpy(buffer, _staticImage.data(), _staticImage.size());
        *length = _staticImage.size();

        // Atualizar contagem regressiva
        _updateCountdown();
        return true;
    }

    if (_hasVideo && _videoFrameCount > 0) {
        // Modo vídeo — avançar frame a frame
        if (_currentFrameIndex >= _videoFrameCount) {
            _currentFrameIndex = 0;  // Loop
        }

        // Simplificação: cada frame é copiada do mesmo buffer
        // Em produção, o AVI seria parseado corretamente
        size_t frameSize = _videoData.size() / _videoFrameCount;
        size_t offset = _currentFrameIndex * frameSize;

        if (offset + frameSize > _videoData.size()) {
            frameSize = _videoData.size() - offset;
        }

        if (frameSize > *length) {
            ESP_LOGE(TAG, "Buffer insuficiente para frame de vídeo");
            return false;
        }

        memcpy(buffer, _videoData.data() + offset, frameSize);
        *length = frameSize;
        _currentFrameIndex++;

        ESP_LOGD(TAG, "Frame %lu/%lu enviada (%zu bytes)",
                 _currentFrameIndex, _videoFrameCount, frameSize);
        return true;
    }

    ESP_LOGW(TAG, "Nenhum vídeo ou imagem carregado");
    return false;
}

bool StoredVideo::setResolution(CameraResolution res) {
    _currentResolution = res;
    return true;
}

bool StoredVideo::setFormat(CameraFormat fmt) {
    _currentFormat = fmt;
    return true;
}

bool StoredVideo::setQuality(uint8_t quality) {
    if (quality > 63) return false;
    _currentQuality = quality;
    return true;
}

bool StoredVideo::isReady() const {
    return _initialized && (_hasStaticImage || _hasVideo);
}

size_t StoredVideo::getMaxFrameSize() const {
    return _maxFrameSize;
}

void StoredVideo::end() {
    _initialized = false;
    _staticImage.clear();
    _videoData.clear();
    _hasStaticImage = false;
    _hasVideo = false;
    _currentFrameIndex = 0;
    ESP_LOGI(TAG, "StoredVideo desligado");
}

bool StoredVideo::loadStaticImage(const uint8_t* data, size_t len) {
    if (!data || len == 0) {
        return false;
    }
    _staticImage.assign(data, data + len);
    _hasStaticImage = true;
    _countdownValue = 10;
    ESP_LOGI(TAG, "Imagem estática carregada (%zu bytes)", len);
    return true;
}

bool StoredVideo::loadVideo(const uint8_t* data, size_t len,
                            uint32_t frameCount, uint32_t fps) {
    if (!data || len == 0 || frameCount == 0) {
        return false;
    }
    _videoData.assign(data, data + len);
    _hasVideo = true;
    _videoFrameCount = frameCount;
    _videoFps = fps;
    _currentFrameIndex = 0;
    ESP_LOGI(TAG, "Vídeo carregado (%zu bytes, %lu frames, %lu fps)",
             len, frameCount, fps);
    return true;
}

uint32_t StoredVideo::getFrameCount() const {
    return _currentFrameIndex;
}

bool StoredVideo::_isCountdownActive() const {
    return _countdownValue > 0;
}

void StoredVideo::_updateCountdown() {
    // Decrementar countdown a cada segundo
    uint32_t now = millis();
    if (now - _lastCountdownTime >= 1000) {
        _lastCountdownTime = now;
        if (_countdownValue > 0) {
            _countdownValue--;
        }
    }
}

#else
// Stub para compilação sem ESP32
StoredVideo::StoredVideo() : _initialized(false), _currentResolution(CameraResolution::VGA),
    _currentFormat(CameraFormat::JPEG), _currentQuality(10), _maxFrameSize(30*1024),
    _hasStaticImage(false), _hasVideo(false), _videoFrameCount(0), _videoFps(10),
    _currentFrameIndex(0), _countdownValue(10), _lastCountdownTime(0) {}
StoredVideo::~StoredVideo() { end(); }
bool StoredVideo::begin(const CameraPinConfig&) { _initialized = true; return true; }
bool StoredVideo::capture(uint8_t*, size_t*) { return false; }
bool StoredVideo::setResolution(CameraResolution res) { _currentResolution = res; return true; }
bool StoredVideo::setFormat(CameraFormat fmt) { _currentFormat = fmt; return true; }
bool StoredVideo::setQuality(uint8_t q) { _currentQuality = q; return true; }
bool StoredVideo::isReady() const { return _initialized; }
size_t StoredVideo::getMaxFrameSize() const { return _maxFrameSize; }
void StoredVideo::end() { _initialized = false; }
bool StoredVideo::loadStaticImage(const uint8_t*, size_t) { return false; }
bool StoredVideo::loadVideo(const uint8_t*, size_t, uint32_t, uint32_t) { return false; }
uint32_t StoredVideo::getFrameCount() const { return 0; }
bool StoredVideo::_isCountdownActive() const { return false; }
void StoredVideo::_updateCountdown() {}
#endif
