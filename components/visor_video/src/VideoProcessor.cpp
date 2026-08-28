/**
 * @file VideoProcessor.cpp
 * @brief Implementação do processador de imagem para o módulo VISOR.
 *
 * Implementa o pipeline de processamento de vídeo com filtros de imagem
 * (brilho, contraste, gamma) e redimensionamento.
 *
 * @version 2.0.0
 * @author ShegaPT
 * @license GPLv3
 */

#include "VideoProcessor.h"
#include <cstring>
#include <cmath>

#ifdef ESP32
#include "esp_log.h"

static const char* TAG = "VideoProcessor";

// Configuração padrão
static const VideoConfig DEFAULT_CONFIG = {
    .targetWidth = 640,
    .targetHeight = 480,
    .jpegQuality = 10,
    .gammaCorrection = 1.0f,
    .brightnessOffset = 0,
    .contrastFactor = 1.0f,
    .enableResize = false,
    .enableFilters = false
};

VideoProcessor::VideoProcessor()
    : _initialized(false)
    , _config(DEFAULT_CONFIG)
    , _gammaLutValid(false)
{
    memset(_gammaLut, 0, sizeof(_gammaLut));
}

VideoProcessor::~VideoProcessor() {
    deinit();
}

bool VideoProcessor::init(const VideoConfig& config) {
    _config = config;

    // Validar configuração
    if (_config.targetWidth == 0 || _config.targetHeight == 0) {
        ESP_LOGE(TAG, "Dimensões alvo inválidas");
        return false;
    }

    // Construir LUT de gamma se necessário
    if (_config.enableFilters) {
        _buildGammaLut(_config.gammaCorrection);
    }

    _initialized = true;
    ESP_LOGI(TAG, "Processador inicializado (%ux%u, Q%d, filters=%s)",
             _config.targetWidth, _config.targetHeight,
             _config.jpegQuality,
             _config.enableFilters ? "ON" : "OFF");
    return true;
}

bool VideoProcessor::processFrame(const uint8_t* input, size_t inputLen,
                                  uint8_t* output, size_t* outputLen) {
    if (!_initialized || !input || !output || !outputLen) {
        return false;
    }

    if (inputLen == 0 || *outputLen == 0) {
        return false;
    }

    // Por agora, passagem direta (JPEG da camera já processado pelo sensor)
    // O processamento real de filtros requer decode/encode JPEG
    // que será implementado quando necessário

    size_t copyLen = inputLen;
    if (copyLen > *outputLen) {
        copyLen = *outputLen;
    }

    memcpy(output, input, copyLen);
    *outputLen = copyLen;

    // Aplicar filtros se habilitados (simplificado — requer decode RGB)
    if (_config.enableFilters && copyLen > 0) {
        // Nota: Para filtros reais, é necessário:
        // 1. Decode JPEG → RGB
        // 2. Aplicar filtros pixel-a-pixel
        // 3. Encode RGB → JPEG
        // Isto será implementado quando a camera fornecer RGB
    }

    return true;
}

void VideoProcessor::setConfig(const VideoConfig& config) {
    _config = config;
    if (_config.enableFilters) {
        _buildGammaLut(_config.gammaCorrection);
    }
    ESP_LOGI(TAG, "Configuração atualizada");
}

const VideoConfig& VideoProcessor::getConfig() const {
    return _config;
}

void VideoProcessor::deinit() {
    _initialized = false;
    _gammaLutValid = false;
    ESP_LOGI(TAG, "Processador desligado");
}

bool VideoProcessor::isInitialized() const {
    return _initialized;
}

void VideoProcessor::_buildGammaLut(float gamma) {
    if (gamma <= 0.0f) gamma = 1.0f;
    float invGamma = 1.0f / gamma;

    for (int i = 0; i < 256; i++) {
        float normalized = (float)i / 255.0f;
        float corrected = powf(normalized, invGamma);
        _gammaLut[i] = (uint8_t)(corrected * 255.0f + 0.5f);
    }
    _gammaLutValid = true;
}

uint8_t VideoProcessor::_applyBrightness(uint8_t pixel) const {
    int16_t result = (int16_t)pixel + _config.brightnessOffset;
    if (result < 0) return 0;
    if (result > 255) return 255;
    return (uint8_t)result;
}

uint8_t VideoProcessor::_applyContrast(uint8_t pixel) const {
    float midpoint = 128.0f;
    float result = _config.contrastFactor * ((float)pixel - midpoint) + midpoint;
    if (result < 0.0f) return 0;
    if (result > 255.0f) return 255;
    return (uint8_t)(result + 0.5f);
}

uint8_t VideoProcessor::_applyGamma(uint8_t pixel) const {
    if (!_gammaLutValid) return pixel;
    return _gammaLut[pixel];
}

void VideoProcessor::_processPixelRow(const uint8_t* rowIn, uint8_t* rowOut, uint16_t width) {
    for (uint16_t x = 0; x < width; x++) {
        uint8_t pixel = rowIn[x];
        pixel = _applyBrightness(pixel);
        pixel = _applyContrast(pixel);
        pixel = _applyGamma(pixel);
        rowOut[x] = pixel;
    }
}

#else
// Stub para compilação sem ESP32
VideoProcessor::VideoProcessor() : _initialized(false), _gammaLutValid(false) {
    memset(&_config, 0, sizeof(_config));
    memset(_gammaLut, 0, sizeof(_gammaLut));
}
VideoProcessor::~VideoProcessor() { deinit(); }
bool VideoProcessor::init(const VideoConfig& c) { _config = c; _initialized = true; return true; }
bool VideoProcessor::processFrame(const uint8_t* in, size_t inLen, uint8_t* out, size_t* outLen) {
    if (inLen > *outLen) inLen = *outLen;
    memcpy(out, in, inLen);
    *outLen = inLen;
    return true;
}
void VideoProcessor::setConfig(const VideoConfig& c) { _config = c; }
const VideoConfig& VideoProcessor::getConfig() const { return _config; }
void VideoProcessor::deinit() { _initialized = false; }
bool VideoProcessor::isInitialized() const { return _initialized; }
void VideoProcessor::_buildGammaLut(float) {}
uint8_t VideoProcessor::_applyBrightness(uint8_t p) const { return p; }
uint8_t VideoProcessor::_applyContrast(uint8_t p) const { return p; }
uint8_t VideoProcessor::_applyGamma(uint8_t p) const { return p; }
void VideoProcessor::_processPixelRow(const uint8_t*, uint8_t*, uint16_t) {}
#endif
