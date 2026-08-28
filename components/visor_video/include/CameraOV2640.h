/**
 * @file CameraOV2640.h
 * @brief Driver de camera para o sensor OV2640.
 *
 * Implementa a interface Camera para o módulo OV2640 utilizando
 * o componente esp_camera do ESP-IDF. Suporta JPEG (hardware encoder),
 * RGB565, YUV422 e grayscale.
 *
 * Configurações padrão:
 * - Resolução: VGA (640x480)
 * - Formato: JPEG
 * - Qualidade: 10 (≈ JPEG Q85, range 80-90)
 *
 * @version 2.0.0
 * @author ShegaPT
 * @license GPLv3
 */

#ifndef CAMERA_OV2640_H
#define CAMERA_OV2640_H

#include "Camera.h"

#ifdef ESP32
#include "esp_camera.h"
#endif

#ifdef __cplusplus

class CameraOV2640 : public Camera {
public:
    CameraOV2640();
    ~CameraOV2640() override;

    bool begin(const CameraPinConfig& pins) override;
    bool capture(uint8_t* buffer, size_t* length) override;
    bool setResolution(CameraResolution res) override;
    bool setFormat(CameraFormat fmt) override;
    bool setQuality(uint8_t quality) override;
    bool isReady() const override;
    size_t getMaxFrameSize() const override;
    void end() override;

private:
    bool _initialized;
    CameraResolution _currentResolution;
    CameraFormat _currentFormat;
    uint8_t _currentQuality;
    size_t _maxFrameSize;

    bool _configureSensor();
#ifdef ESP32
    int _resolutionToEsp(framesize_t* esp_size) const;
    int _formatToEsp(pixformat_t* esp_format) const;
#else
    int _resolutionToEsp(void* esp_size) const;
    int _formatToEsp(void* esp_format) const;
#endif
};

#endif /* __cplusplus */
#endif /* CAMERA_OV2640_H */
