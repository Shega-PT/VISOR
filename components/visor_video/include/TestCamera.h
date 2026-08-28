/**
 * @file TestCamera.h
 * @brief Camera de teste para ESP32 DevKitV1 sem hardware real.
 *
 * Implementa a interface Camera gerando frames sintéticas
 * com padrão de cores animado. Cada frame contém um JPEG mínimo
 * com cor baseada no índice da frame.
 *
 * Utilizado para testar o pipeline completo VISOR sem camera real.
 *
 * @version 3.0.0
 * @author ShegaPT
 * @license GPLv3
 */

#ifndef TEST_CAMERA_H
#define TEST_CAMERA_H

#include "Camera.h"
#include <stdint.h>
#include <stddef.h>
#include <string.h>

#ifdef __cplusplus

class TestCamera : public Camera {
public:
    TestCamera()
        : _initialized(false)
        , _resolution(CameraResolution::QQVGA)
        , _format(CameraFormat::JPEG)
        , _quality(10)
        , _frameIndex(0)
        , _frameCount(20)
    {
        _buildSyntheticJpeg();
    }

    ~TestCamera() override {
        end();
    }

    bool begin(const CameraPinConfig& pins) override {
        (void)pins;
        _initialized = true;
        _frameIndex = 0;
        return true;
    }

    bool capture(uint8_t* buffer, size_t* length) override {
        if (!_initialized || !buffer || !length) {
            return false;
        }

        if (*length < _jpegSize) {
            return false;
        }

        memcpy(buffer, _jpegData, _jpegSize);
        *length = _jpegSize;

        _frameIndex++;
        if (_frameIndex >= _frameCount) {
            _frameIndex = 0;
        }

        return true;
    }

    bool setResolution(CameraResolution res) override {
        _resolution = res;
        _buildSyntheticJpeg();
        return true;
    }

    bool setFormat(CameraFormat fmt) override {
        _format = fmt;
        return true;
    }

    bool setQuality(uint8_t quality) override {
        if (quality > 63) return false;
        _quality = quality;
        return true;
    }

    bool isReady() const override {
        return _initialized;
    }

    size_t getMaxFrameSize() const override {
        return sizeof(_jpegData);
    }

    void end() override {
        _initialized = false;
    }

    uint32_t getFrameIndex() const { return _frameIndex; }
    void setFrameCount(uint32_t count) { _frameCount = count; }

private:
    bool _initialized;
    CameraResolution _resolution;
    CameraFormat _format;
    uint8_t _quality;
    uint32_t _frameIndex;
    uint32_t _frameCount;

    uint8_t _jpegData[2048];
    size_t _jpegSize;

    void _buildSyntheticJpeg() {
        int w = 160, h = 120;
        if (_resolution == CameraResolution::QVGA) { w = 320; h = 240; }
        else if (_resolution == CameraResolution::VGA) { w = 640; h = 480; }

        int mcu_w = (w + 7) / 8;
        int mcu_h = (h + 7) / 8;

        uint8_t Y_val = 128;
        uint8_t Cb_val = 128;
        uint8_t Cr_val = 128;

        uint8_t* p = _jpegData;

        // SOI
        *p++ = 0xFF; *p++ = 0xD8;

        // APP0 JFIF
        *p++ = 0xFF; *p++ = 0xE0;
        *p++ = 0x00; *p++ = 0x10;
        *p++ = 'J'; *p++ = 'F'; *p++ = 'I'; *p++ = 'F'; *p++ = 0x00;
        *p++ = 0x01; *p++ = 0x01;
        *p++ = 0x00;
        *p++ = 0x00; *p++ = 0x01;
        *p++ = 0x00; *p++ = 0x01;
        *p++ = 0x00; *p++ = 0x00;

        // DQT Luma
        *p++ = 0xFF; *p++ = 0xDB;
        *p++ = 0x00; *p++ = 0x43;
        *p++ = 0x00;
        for (int i = 0; i < 64; i++) *p++ = 1;

        // DQT Chroma
        *p++ = 0xFF; *p++ = 0xDB;
        *p++ = 0x00; *p++ = 0x43;
        *p++ = 0x01;
        for (int i = 0; i < 64; i++) *p++ = 1;

        // SOF0
        *p++ = 0xFF; *p++ = 0xC0;
        *p++ = 0x00; *p++ = 0x11;
        *p++ = 0x08;
        *p++ = (uint8_t)(h >> 8); *p++ = (uint8_t)(h & 0xFF);
        *p++ = (uint8_t)(w >> 8); *p++ = (uint8_t)(w & 0xFF);
        *p++ = 0x03;
        *p++ = 0x01; *p++ = 0x11; *p++ = 0x00;
        *p++ = 0x02; *p++ = 0x11; *p++ = 0x01;
        *p++ = 0x03; *p++ = 0x11; *p++ = 0x01;

        // DHT DC Luma
        *p++ = 0xFF; *p++ = 0xC4;
        *p++ = 0x00; *p++ = 0x1F;
        *p++ = 0x00;
        uint8_t dc_bits[] = {0,1,5,1,1,1,1,1,1,0,0,0,0,0,0,0};
        memcpy(p, dc_bits, 16); p += 16;
        for (int i = 0; i < 12; i++) *p++ = i;

        // DHT AC Luma
        *p++ = 0xFF; *p++ = 0xC4;
        *p++ = 0x00; *p++ = 0xB5;
        *p++ = 0x10;
        uint8_t ac_bits[] = {0,2,1,3,3,2,4,3,5,5,4,4,0,0,1,0x7d};
        memcpy(p, ac_bits, 16); p += 16;
        int ac_count = 0;
        for (int i = 0; i < 16; i++) ac_count += ac_bits[i];
        for (int i = 0; i < ac_count && i < 162; i++) *p++ = i;

        // DHT DC Chroma
        *p++ = 0xFF; *p++ = 0xC4;
        *p++ = 0x00; *p++ = 0x1F;
        *p++ = 0x01;
        memcpy(p, dc_bits, 16); p += 16;
        for (int i = 0; i < 12; i++) *p++ = i;

        // DHT AC Chroma
        *p++ = 0xFF; *p++ = 0xC4;
        *p++ = 0x00; *p++ = 0xB5;
        *p++ = 0x11;
        uint8_t ac_chroma_bits[] = {0,2,1,2,4,4,3,4,7,5,4,4,0,1,2,0x77};
        memcpy(p, ac_chroma_bits, 16); p += 16;
        ac_count = 0;
        for (int i = 0; i < 16; i++) ac_count += ac_chroma_bits[i];
        for (int i = 0; i < ac_count && i < 162; i++) *p++ = i;

        // SOS
        *p++ = 0xFF; *p++ = 0xDA;
        *p++ = 0x00; *p++ = 0x0C;
        *p++ = 0x03;
        *p++ = 0x01; *p++ = 0x00;
        *p++ = 0x02; *p++ = 0x11;
        *p++ = 0x03; *p++ = 0x11;
        *p++ = 0x00; *p++ = 0x3F; *p++ = 0x00;

        // Scan data — MCUs com cor constante
        for (int i = 0; i < mcu_w * mcu_h; i++) {
            // DC Y
            int dc = Y_val - 128;
            if (dc == 0) { *p++ = 0x00; }
            else {
                uint8_t v = (dc < 0) ? (uint8_t)(dc + 256) : (uint8_t)dc;
                *p++ = 0x00;
                *p++ = v;
            }
            // DC Cb
            dc = Cb_val - 128;
            if (dc == 0) { *p++ = 0x00; }
            else {
                uint8_t v = (dc < 0) ? (uint8_t)(dc + 256) : (uint8_t)dc;
                *p++ = 0x00;
                *p++ = v;
            }
            // DC Cr
            dc = Cr_val - 128;
            if (dc == 0) { *p++ = 0x00; }
            else {
                uint8_t v = (dc < 0) ? (uint8_t)(dc + 256) : (uint8_t)dc;
                *p++ = 0x00;
                *p++ = v;
            }
        }

        // EOI
        *p++ = 0xFF; *p++ = 0xD9;

        _jpegSize = (size_t)(p - _jpegData);
    }
};

#endif /* __cplusplus */
#endif /* TEST_CAMERA_H */
