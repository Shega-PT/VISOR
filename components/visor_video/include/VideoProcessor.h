/**
 * @file VideoProcessor.h
 * @brief Processador de imagem para o módulo VISOR.
 *
 * Implementa o pipeline de processamento de vídeo incluindo:
 * - Redimensionamento (resize) para VGA (640x480)
 * - Filtros de imagem (brilho, contraste, gamma)
 * - Compressão MJPEG
 *
 * O processador recebe frames JPEG da camera, aplica processamento
 * se necessário, e produz frames prontas para fragmentação TLV.
 *
 * Configurações padrão:
 * - Resolução alvo: VGA (640x480)
 * - Qualidade JPEG: 10 (≈ JPEG Q85)
 * - Gamma: 1.0 (neutro)
 * - Brilho: 0 (neutro)
 * - Contraste: 1.0 (neutro)
 *
 * @version 2.0.0
 * @author ShegaPT
 * @license GPLv3
 */

#ifndef VIDEO_PROCESSOR_H
#define VIDEO_PROCESSOR_H

#include <stdint.h>
#include <stddef.h>

#ifdef __cplusplus

/**
 * @brief Configuração do processador de vídeo.
 */
struct VideoConfig {
    uint16_t targetWidth;      /**< Largura alvo em pixels (padrão: 640) */
    uint16_t targetHeight;     /**< Altura alvo em pixels (padrão: 480) */
    uint8_t jpegQuality;       /**< Qualidade OV2640 (0-63, padrão: 10) */
    float gammaCorrection;     /**< Correção gamma (1.0 = neutro) */
    int8_t brightnessOffset;   /**< Offset de brilho (-128 a +127) */
    float contrastFactor;      /**< Fator de contraste (1.0 = neutro) */
    bool enableResize;         /**< Ativar redimensionamento */
    bool enableFilters;        /**< Ativar filtros de imagem */
};

class VideoProcessor {
public:
    VideoProcessor();
    ~VideoProcessor();

    /**
     * @brief Inicializa o processador com a configuração especificada.
     * @param config Configuração do processador.
     * @return true em sucesso, false em erro.
     */
    bool init(const VideoConfig& config);

    /**
     * @brief Processa uma frame de vídeo.
     *
     * Pipeline completo:
     * 1. Validação de entrada
     * 2. Redimensionamento (se necessário e habilitado)
     * 3. Filtros (se habilitados)
     * 4. Saída (frame processada)
     *
     * @param input Buffer de entrada (frame original).
     * @param inputLen Tamanho da frame de entrada em bytes.
     * @param output Buffer de saída (frame processada).
     * @param outputLen Input: tamanho do buffer. Output: tamanho real da frame.
     * @return true em sucesso, false em erro.
     */
    bool processFrame(const uint8_t* input, size_t inputLen,
                      uint8_t* output, size_t* outputLen);

    /**
     * @brief Atualiza a configuração do processador.
     * @param config Nova configuração.
     */
    void setConfig(const VideoConfig& config);

    /**
     * @brief Retorna a configuração atual.
     */
    const VideoConfig& getConfig() const;

    /**
     * @brief Desliga o processador e liberta recursos.
     */
    void deinit();

    /**
     * @brief Retorna true se o processador está inicializado.
     */
    bool isInitialized() const;

private:
    bool _initialized;
    VideoConfig _config;

    // Tabelas LUT para filtros
    uint8_t _gammaLut[256];
    bool _gammaLutValid;

    void _buildGammaLut(float gamma);
    uint8_t _applyBrightness(uint8_t pixel) const;
    uint8_t _applyContrast(uint8_t pixel) const;
    uint8_t _applyGamma(uint8_t pixel) const;
    void _processPixelRow(const uint8_t* rowIn, uint8_t* rowOut, uint16_t width);
};

#endif /* __cplusplus */
#endif /* VIDEO_PROCESSOR_H */
