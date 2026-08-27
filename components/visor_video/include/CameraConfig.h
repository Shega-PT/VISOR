/**
 * @file CameraConfig.h
 * @brief Configuração de pinos de camera modular.
 *
 * Define a estrutura CameraPinConfig para configuração de pinos de
 * módulos de camera diferentes. Suporta múltiplos presets pré-definidos
 * e configuração manual para placas/módulos personalizados.
 *
 * @version 2.0.0
 * @author ShegaPT
 * @license GPLv3
 */

#ifndef CAMERA_CONFIG_H
#define CAMERA_CONFIG_H

#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

/**
 * @brief Configuração de pinos para um módulo de camera.
 *
 * Contém todos os pinos necessários para inicializar um módulo de camera
 * via DVP (Digital Video Port) e SCCB (Serial Camera Control Bus).
 *
 * Utilizar -1 para pinos não utilizados (ex: reset ligado a NC).
 */
struct CameraPinConfig {
    int8_t pwdn;      /**< Pino de power down (-1 se não utilizado) */
    int8_t reset;     /**< Pino de reset (-1 se não utilizado) */
    int8_t xclk;      /**< Pino de clock externo (ESP32 → Camera) */
    int8_t siod;      /**< Pino de dados SCCB/I2C (SDA) */
    int8_t sioc;      /**< Pino de clock SCCB/I2C (SCL) */
    int8_t d0;        /**< Pino de dados DVP D0 */
    int8_t d1;        /**< Pino de dados DVP D1 */
    int8_t d2;        /**< Pino de dados DVP D2 */
    int8_t d3;        /**< Pino de dados DVP D3 */
    int8_t d4;        /**< Pino de dados DVP D4 */
    int8_t d5;        /**< Pino de dados DVP D5 */
    int8_t d6;        /**< Pino de dados DVP D6 */
    int8_t d7;        /**< Pino de dados DVP D7 */
    int8_t vsync;     /**< Pino de sincronização vertical */
    int8_t href;      /**< Pino de referência horizontal */
    int8_t pclk;      /**< Pino de pixel clock (Camera → ESP32) */
    uint32_t xclk_freq_hz; /**< Frequência do clock externo em Hz */
};

/* ========================================================================
 * PRESETS PRÉ-DEFINIDOS
 * ======================================================================== */

/**
 * @brief Pinos para o módulo AI-Thinker ESP32-CAM.
 *
 * Configuração padrão para a placa ESP32-CAM com módulo OV2640.
 * Esta é a configuração mais comum e testada.
 */
static const struct CameraPinConfig CAMERA_PRESET_AI_THINKER_ESP32_CAM = {
    .pwdn = 32,
    .reset = -1,
    .xclk = 0,
    .siod = 26,
    .sioc = 27,
    .d0 = 4,
    .d1 = 5,
    .d2 = 18,
    .d3 = 19,
    .d4 = 36,
    .d5 = 39,
    .d6 = 34,
    .d7 = 35,
    .vsync = 25,
    .href = 23,
    .pclk = 22,
    .xclk_freq_hz = 20000000
};

/**
 * @brief Pinos para o módulo ESP-S3-CAM (genérico).
 *
 * Configuração para placas ESP32-S3 com interface de camera DVP.
 * NOTA: Verificar pinagem específica da placa antes de utilizar.
 */
static const struct CameraPinConfig CAMERA_PRESET_ESP32_S3_CAM = {
    .pwdn = -1,
    .reset = -1,
    .xclk = 15,
    .siod = 4,
    .sioc = 5,
    .d0 = 11,
    .d1 = 9,
    .d2 = 8,
    .d3 = 10,
    .d4 = 12,
    .d5 = 18,
    .d6 = 17,
    .d7 = 16,
    .vsync = 6,
    .href = 7,
    .pclk = 13,
    .xclk_freq_hz = 20000000
};

/* ========================================================================
 * UTILITÁRIOS
 * ======================================================================== */

/**
 * @brief Cria uma configuração de pinos personalizada.
 *
 * Função auxiliar para criar uma CameraPinConfig com valores específicos.
 *
 * @return Estrutura CameraPinConfig inicializada.
 */
static inline struct CameraPinConfig camera_config_create(
    int8_t pwdn, int8_t reset, int8_t xclk,
    int8_t siod, int8_t sioc,
    int8_t d0, int8_t d1, int8_t d2, int8_t d3,
    int8_t d4, int8_t d5, int8_t d6, int8_t d7,
    int8_t vsync, int8_t href, int8_t pclk,
    uint32_t xclk_freq_hz
) {
    struct CameraPinConfig config;
    config.pwdn = pwdn;
    config.reset = reset;
    config.xclk = xclk;
    config.siod = siod;
    config.sioc = sioc;
    config.d0 = d0;
    config.d1 = d1;
    config.d2 = d2;
    config.d3 = d3;
    config.d4 = d4;
    config.d5 = d5;
    config.d6 = d6;
    config.d7 = d7;
    config.vsync = vsync;
    config.href = href;
    config.pclk = pclk;
    config.xclk_freq_hz = xclk_freq_hz;
    return config;
}

#ifdef __cplusplus
}
#endif

#endif /* CAMERA_CONFIG_H */
