/**
 * @file Camera.h
 * @brief Interface abstrata para módulos de camera.
 *
 * Define a interface virtual Camera que deve ser implementada por todos
 * os drivers de camera. Permite abstrair o hardware específico e suportar
 * múltiplos módulos de camera (OV2640, StoredVideo, etc.).
 *
 * @version 2.0.0
 * @author ShegaPT
 * @license GPLv3
 */

#ifndef CAMERA_H
#define CAMERA_H

#include <stdint.h>
#include <stddef.h>
#include "CameraConfig.h"

#ifdef __cplusplus

/**
 * @brief Formatos de pixel suportados pela camera.
 */
enum class CameraFormat : uint8_t {
    JPEG = 0,       /**< JPEG comprimido (hardware encoder no sensor) */
    RGB565 = 1,     /**< RGB 16-bit (5R-6G-5B) */
    YUV422 = 2,     /**< YUV 4:2:2 */
    GRAYSCALE = 3   /**< Escala de cinzentos 8-bit */
};

/**
 * @brief Resoluções de camera suportadas.
 */
enum class CameraResolution : uint8_t {
    QQVGA = 0,  /**< 160x120 */
    QVGA = 1,   /**< 320x240 */
    VGA = 2,    /**< 640x480 */
    SVGA = 3,   /**< 800x600 */
    HD = 4      /**< 1280x720 */
};

/**
 * @brief Interface abstrata para módulos de camera.
 *
 * Classe base virtual que define a interface para todos os drivers
 * de camera. Cada implementação concreta (OV2640, StoredVideo, etc.)
 * deve implementar todos os métodos virtuais puros.
 *
 * Esta abstração permite ao módulo Video utilizar qualquer fonte
 * de vídeo sem dependência de hardware específica.
 */
class Camera {
public:
    virtual ~Camera() = default;

    /**
     * @brief Inicializa a camera com a configuração de pinos especificada.
     * @param pins Configuração de pinos da camera.
     * @return true em sucesso, false em erro.
     */
    virtual bool begin(const CameraPinConfig& pins) = 0;

    /**
     * @brief Captura uma frame da camera.
     *
     * O buffer de saída deve ter tamanho suficiente para conter a frame.
     * Utilizar getMaxFrameSize() para obter o tamanho máximo.
     *
     * @param buffer Buffer de saída para os dados da frame.
     * @param length Input: tamanho do buffer. Output: tamanho real da frame.
     * @return true em sucesso, false em erro.
     */
    virtual bool capture(uint8_t* buffer, size_t* length) = 0;

    /**
     * @brief Define a resolução da camera.
     * @param res Resolução pretendida.
     * @return true em sucesso, false se a resolução não for suportada.
     */
    virtual bool setResolution(CameraResolution res) = 0;

    /**
     * @brief Define o formato de pixel da camera.
     * @param fmt Formato pretendido.
     * @return true em sucesso, false se o formato não for suportado.
     */
    virtual bool setFormat(CameraFormat fmt) = 0;

    /**
     * @brief Define a qualidade de compressão JPEG.
     *
     * Para OV2640: escala 0-63 (0 = máxima qualidade, 63 = mínima).
     * Valores recomendados: 8-12 (equivalente a JPEG Q80-90).
     *
     * @param quality Qualidade (significado específico do sensor).
     * @return true em sucesso, false em erro.
     */
    virtual bool setQuality(uint8_t quality) = 0;

    /**
     * @brief Verifica se a camera está pronta para captura.
     * @return true se pronta, false caso contrário.
     */
    virtual bool isReady() const = 0;

    /**
     * @brief Retorna o tamanho máximo de uma frame em bytes.
     * @return Tamanho máximo em bytes.
     */
    virtual size_t getMaxFrameSize() const = 0;

    /**
     * @brief Desliga a camera e liberta recursos.
     */
    virtual void end() = 0;
};

#endif /* __cplusplus */
#endif /* CAMERA_H */
