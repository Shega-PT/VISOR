/**
 * @file CameraOV2640.cpp
 * @brief Implementação do driver de camera OV2640.
 *
 * Utiliza o componente esp_camera do ESP-IDF para aceder ao sensor
 * OV2640 via DVP (Digital Video Port) e SCCB (I2C).
 *
 * @version 2.0.0
 * @author ShegaPT
 * @license GPLv3
 */

#include "CameraOV2640.h"
#include <cstring>

#ifdef ESP32
#include "esp_camera.h"
#include "esp_log.h"
#include "sensor.h"

static const char* TAG = "CameraOV2640";

// Configuração de pinos padrão para AI-Thinker ESP32-CAM
#define CAM_PIN_PWDN    32
#define CAM_PIN_RESET   -1
#define CAM_PIN_XCLK    0
#define CAM_PIN_SIOD    26
#define CAM_PIN_SIOC    27
#define CAM_PIN_D7      35
#define CAM_PIN_D6      34
#define CAM_PIN_D5      39
#define CAM_PIN_D4      36
#define CAM_PIN_D3      19
#define CAM_PIN_D2      18
#define CAM_PIN_D1      5
#define CAM_PIN_D0      4
#define CAM_PIN_VSYNC   25
#define CAM_PIN_HREF    23
#define CAM_PIN_PCLK    22

CameraOV2640::CameraOV2640()
    : _initialized(false)
    , _currentResolution(CameraResolution::VGA)
    , _currentFormat(CameraFormat::JPEG)
    , _currentQuality(10)
    , _maxFrameSize(30 * 1024)  // 30 KB estimado para VGA JPEG
{
}

CameraOV2640::~CameraOV2640() {
    end();
}

bool CameraOV2640::begin(const CameraPinConfig& pins) {
    if (_initialized) {
        ESP_LOGW(TAG, "Camera já inicializada");
        return true;
    }

    camera_config_t config;
    config.ledc_channel = LEDC_CHANNEL_0;
    config.ledc_timer = LEDC_TIMER_0;
    config.pin_d0 = pins.d0;
    config.pin_d1 = pins.d1;
    config.pin_d2 = pins.d2;
    config.pin_d3 = pins.d3;
    config.pin_d4 = pins.d4;
    config.pin_d5 = pins.d5;
    config.pin_d6 = pins.d6;
    config.pin_d7 = pins.d7;
    config.pin_xclk = pins.xclk;
    config.pin_pclk = pins.pclk;
    config.pin_vsync = pins.vsync;
    config.pin_href = pins.href;
    config.pin_sccb_sda = pins.siod;
    config.pin_sccb_scl = pins.sioc;
    config.pin_pwdn = pins.pwdn;
    config.pin_reset = pins.reset;
    config.xclk_freq_hz = pins.xclk_freq_hz;
    config.pixel_format = PIXFORMAT_JPEG;
    config.grab_mode = CAMERA_GRAB_LATEST;
    config.fb_location = CAMERA_FB_IN_PSRAM;
    config.jpeg_quality = _currentQuality;
    config.fb_count = 2;

    // Configurar resolução
    framesize_t esp_size;
    if (_resolutionToEsp(&esp_size) != 0) {
        ESP_LOGE(TAG, "Resolução não suportada");
        return false;
    }
    config.frame_size = esp_size;

    esp_err_t err = esp_camera_init(&config);
    if (err != ESP_OK) {
        ESP_LOGE(TAG, "Falha ao inicializar camera: 0x%x", err);
        return false;
    }

    _initialized = true;
    _configureSensor();

    ESP_LOGI(TAG, "Camera OV2640 inicializada (VGA JPEG Q%d)", _currentQuality);
    return true;
}

bool CameraOV2640::capture(uint8_t* buffer, size_t* length) {
    if (!_initialized || !buffer || !length) {
        return false;
    }

    camera_fb_t* fb = esp_camera_fb_get();
    if (!fb) {
        ESP_LOGE(TAG, "Falha ao capturar frame");
        return false;
    }

    if (fb->len > *length) {
        ESP_LOGE(TAG, "Buffer insuficiente: %zu < %zu", *length, fb->len);
        esp_camera_fb_return(fb);
        return false;
    }

    memcpy(buffer, fb->buf, fb->len);
    *length = fb->len;

    esp_camera_fb_return(fb);
    return true;
}

bool CameraOV2640::setResolution(CameraResolution res) {
    if (!_initialized) {
        _currentResolution = res;
        return true;
    }

    framesize_t esp_size;
    if (_resolutionToEsp(&esp_size) != 0) {
        return false;
    }

    sensor_t* s = esp_camera_sensor_get();
    if (!s) {
        return false;
    }

    int ret = s->set_framesize(s, esp_size);
    if (ret == 0) {
        _currentResolution = res;
        ESP_LOGI(TAG, "Resolução alterada para %d", (int)res);
    }
    return ret == 0;
}

bool CameraOV2640::setFormat(CameraFormat fmt) {
    if (!_initialized) {
        _currentFormat = fmt;
        return true;
    }

    pixformat_t esp_fmt;
    if (_formatToEsp(&esp_fmt) != 0) {
        return false;
    }

    sensor_t* s = esp_camera_sensor_get();
    if (!s) {
        return false;
    }

    int ret = s->set_pixformat(s, esp_fmt);
    if (ret == 0) {
        _currentFormat = fmt;
        ESP_LOGI(TAG, "Formato alterado para %d", (int)fmt);
    }
    return ret == 0;
}

bool CameraOV2640::setQuality(uint8_t quality) {
    if (quality > 63) {
        return false;
    }

    _currentQuality = quality;

    if (_initialized) {
        sensor_t* s = esp_camera_sensor_get();
        if (s) {
            s->set_quality(s, quality);
            ESP_LOGI(TAG, "Qualidade alterada para %d", quality);
        }
    }

    return true;
}

bool CameraOV2640::isReady() const {
    return _initialized;
}

size_t CameraOV2640::getMaxFrameSize() const {
    return _maxFrameSize;
}

void CameraOV2640::end() {
    if (_initialized) {
        esp_camera_deinit();
        _initialized = false;
        ESP_LOGI(TAG, "Camera OV2640 desligada");
    }
}

bool CameraOV2640::_configureSensor() {
    sensor_t* s = esp_camera_sensor_get();
    if (!s) {
        return false;
    }

    // Configurações adicionais do sensor
    s->set_brightness(s, 0);     // -2 a 2
    s->set_contrast(s, 0);       // -2 a 2
    s->set_saturation(s, 0);     // -2 a 2
    s->set_hmirror(s, 0);        // Espelhamento horizontal
    s->set_vflip(s, 0);          // Espelhamento vertical
    s->set_dcw(s, 1);            // Downsize enable

    return true;
}

int CameraOV2640::_resolutionToEsp(framesize_t* esp_size) const {
    switch (_currentResolution) {
        case CameraResolution::QQVGA: *esp_size = FRAMESIZE_QQVGA; return 0;
        case CameraResolution::QVGA:  *esp_size = FRAMESIZE_QVGA;  return 0;
        case CameraResolution::VGA:   *esp_size = FRAMESIZE_VGA;   return 0;
        case CameraResolution::SVGA:  *esp_size = FRAMESIZE_SVGA;  return 0;
        case CameraResolution::HD:    *esp_size = FRAMESIZE_HD;    return 0;
        default: return -1;
    }
}

int CameraOV2640::_formatToEsp(pixformat_t* esp_format) const {
    switch (_currentFormat) {
        case CameraFormat::JPEG:     *esp_format = PIXFORMAT_JPEG;     return 0;
        case CameraFormat::RGB565:   *esp_format = PIXFORMAT_RGB565;   return 0;
        case CameraFormat::YUV422:   *esp_format = PIXFORMAT_YUV422;   return 0;
        case CameraFormat::GRAYSCALE: *esp_format = PIXFORMAT_GRAYSCALE; return 0;
        default: return -1;
    }
}

#else
// Stub para compilação sem ESP32 (testes no host)
CameraOV2640::CameraOV2640() : _initialized(false), _currentResolution(CameraResolution::VGA),
    _currentFormat(CameraFormat::JPEG), _currentQuality(10), _maxFrameSize(30*1024) {}
CameraOV2640::~CameraOV2640() { end(); }
bool CameraOV2640::begin(const CameraPinConfig&) { return false; }
bool CameraOV2640::capture(uint8_t*, size_t*) { return false; }
bool CameraOV2640::setResolution(CameraResolution res) { _currentResolution = res; return true; }
bool CameraOV2640::setFormat(CameraFormat fmt) { _currentFormat = fmt; return true; }
bool CameraOV2640::setQuality(uint8_t q) { _currentQuality = q; return true; }
bool CameraOV2640::isReady() const { return false; }
size_t CameraOV2640::getMaxFrameSize() const { return _maxFrameSize; }
void CameraOV2640::end() { _initialized = false; }
bool CameraOV2640::_configureSensor() { return false; }
int CameraOV2640::_resolutionToEsp(void*) const { return -1; }
int CameraOV2640::_formatToEsp(void*) const { return -1; }
#endif
