/**
 * @file StoredVideo.h
 * @brief Módulo de vídeo armazenado para testes.
 *
 * Implementa a interface Camera utilizando frames pré-armazenadas
 * em memória. Utilizado para testar todo o pipeline de processamento
 * sem necessidade de hardware de camera real.
 *
 * Modos de operação:
 * - Imagem estática: Repete a mesma frame com sobreposição de
 *   contagem regressiva (10→0).
 * - Vídeo: Reproduz frames de um AVI armazenado em loop.
 *
 * @version 2.0.0
 * @author ShegaPT
 * @license GPLv3
 */

#ifndef STORED_VIDEO_H
#define STORED_VIDEO_H

#include "Camera.h"
#include <vector>

#ifdef __cplusplus

class StoredVideo : public Camera {
public:
    StoredVideo();
    ~StoredVideo() override;

    bool begin(const CameraPinConfig& pins) override;
    bool capture(uint8_t* buffer, size_t* length) override;
    bool setResolution(CameraResolution res) override;
    bool setFormat(CameraFormat fmt) override;
    bool setQuality(uint8_t quality) override;
    bool isReady() const override;
    size_t getMaxFrameSize() const override;
    void end() override;

    /**
     * @brief Carrega uma imagem estática JPEG para uso em testes.
     * @param data Ponteiro para os dados JPEG.
     * @param len Tamanho dos dados em bytes.
     * @return true em sucesso, false em erro.
     */
    bool loadStaticImage(const uint8_t* data, size_t len);

    /**
     * @brief Carrega um pequeno vídeo AVI para uso em testes.
     * @param data Ponteiro para os dados do vídeo.
     * @param len Tamanho total do vídeo em bytes.
     * @param frameCount Número de frames no vídeo.
     * @param fps Frame rate do vídeo.
     * @return true em sucesso, false em erro.
     */
    bool loadVideo(const uint8_t* data, size_t len, uint32_t frameCount, uint32_t fps);

    /**
     * @brief Retorna o número de frames processadas desde o início.
     */
    uint32_t getFrameCount() const;

private:
    bool _initialized;
    CameraResolution _currentResolution;
    CameraFormat _currentFormat;
    uint8_t _currentQuality;
    size_t _maxFrameSize;

    // Imagem estática
    std::vector<uint8_t> _staticImage;
    bool _hasStaticImage;

    // Vídeo
    std::vector<uint8_t> _videoData;
    bool _hasVideo;
    uint32_t _videoFrameCount;
    uint32_t _videoFps;
    uint32_t _currentFrameIndex;

    // Contagem regressiva
    uint32_t _countdownValue;
    uint32_t _lastCountdownTime;

    bool _isCountdownActive() const;
    void _updateCountdown();
};

#endif /* __cplusplus */
#endif /* STORED_VIDEO_H */
