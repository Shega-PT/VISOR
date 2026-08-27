/**
 * @file main.cpp
 * @brief Entry point do módulo VISOR (PlatformIO/ESP-IDF).
 *
 * Ponto de entrada principal do firmware do módulo VISOR.
 * Inicializa todos os módulos e entra no loop principal de
 * processamento de vídeo.
 *
 * Pipeline principal:
 * 1. Inicializar camera (OV2640 via DVP)
 * 2. Inicializar processador de vídeo
 * 3. Inicializar escritor AVI MJPEG
 * 4. Configurar interface de transporte (callback para Master)
 * 5. Loop: capturar → processar → fragmentar → enviar
 *
 * @version 2.0.0
 * @author ShegaPT
 * @license GPLv3
 */

#include <stdio.h>
#include <freertos/FreeRTOS.h>
#include <freertos/task.h>
#include "esp_log.h"
#include "esp_timer.h"

#include "Camera.h"
#include "CameraOV2640.h"
#include "CameraConfig.h"
#include "Video.h"
#include "VideoProcessor.h"
#include "transport.h"
#include "protocol_ffi.h"

static const char* TAG = "VISOR";

/* ========================================================================
 * CONFIGURAÇÃO
 * ======================================================================== */

/** Frame rate alvo em FPS. */
#define TARGET_FPS          30

/** Intervalo entre frames em milissegundos. */
#define FRAME_INTERVAL_MS   (1000 / TARGET_FPS)

/** Tamanho do buffer de envio. */
#define SEND_BUFFER_SIZE    1024

/* ========================================================================
 * CALLBACK DE TRANSPORTE (PLACEHOLDER)
 * ======================================================================== */

/**
 * @brief Callback de envio de dados ao Master.
 *
 * Este é um placeholder que deve ser substituído pela implementação
 * real de comunicação com o Master (I2C, SPI, etc.).
 *
 * @param data Dados a enviar.
 * @param length Tamanho dos dados.
 * @param user_data Contexto (não utilizado).
 * @return 0 em sucesso.
 */
static int master_send_callback(const uint8_t* data, size_t length, void* user_data) {
    (void)user_data;
    ESP_LOGI(TAG, "Master send: %zu bytes", length);
    // TODO: Implementar comunicação real com o Master
    // Exemplo: I2C, SPI, UART, etc.
    return 0;
}

/* ========================================================================
 * ENTRY POINT
 * ======================================================================== */

extern "C" void app_main(void) {
    ESP_LOGI(TAG, "========================================");
    ESP_LOGI(TAG, "  VISOR — Sistema de Visão Computacional");
    ESP_LOGI(TAG, "  AERUS Project v2.0.0");
    ESP_LOGI(TAG, "========================================");

    // Inicializar protocolo (verificação de integridade)
    ESP_LOGI(TAG, "A inicializar protocolo TLV v2.0.0...");
    uint8_t test_crc = visor_calc_crc8((const uint8_t*)"VISOR", 5);
    ESP_LOGI(TAG, "CRC8 teste: 0x%02X", test_crc);

    // Configurar camera
    ESP_LOGI(TAG, "A inicializar camera OV2640...");
    CameraOV2640 camera;
    const CameraPinConfig& pins = CAMERA_PRESET_AI_THINKER_ESP32_CAM;

    if (!camera.begin(pins)) {
        ESP_LOGE(TAG, "Falha ao inicializar camera");
        return;
    }

    // Configurar resolução VGA e qualidade
    camera.setResolution(CameraResolution::VGA);
    camera.setFormat(CameraFormat::JPEG);
    camera.setQuality(10);  // ≈ JPEG Q85

    ESP_LOGI(TAG, "Camera pronta (VGA JPEG Q10)");

    // Configurar processador de vídeo
    VideoConfig videoConfig;
    videoConfig.targetWidth = 640;
    videoConfig.targetHeight = 480;
    videoConfig.jpegQuality = 10;
    videoConfig.gammaCorrection = 1.0f;
    videoConfig.brightnessOffset = 0;
    videoConfig.contrastFactor = 1.0f;
    videoConfig.enableResize = false;
    videoConfig.enableFilters = false;

    // Configurar interface de transporte
    TransportInterface transport;
    transport.send = master_send_callback;
    transport.user_data = nullptr;

    // Inicializar módulo de vídeo
    Video videoModule;
    if (!videoModule.begin(&camera, transport, videoConfig)) {
        ESP_LOGE(TAG, "Falha ao inicializar módulo de vídeo");
        camera.end();
        return;
    }

    videoModule.setDebug(true);
    ESP_LOGI(TAG, "Módulo de vídeo inicializado");

    // Loop principal
    ESP_LOGI(TAG, "A iniciar loop principal (target: %d FPS)...", TARGET_FPS);

    int64_t lastFrameTime = esp_timer_get_time();

    while (true) {
        int64_t now = esp_timer_get_time();
        int64_t elapsed = (now - lastFrameTime) / 1000;  // ms

        if (elapsed >= FRAME_INTERVAL_MS) {
            // Processar e enviar frame
            videoModule.processAndSend();

            // Estatísticas periódicas
            static uint32_t statsCounter = 0;
            statsCounter++;
            if (statsCounter >= TARGET_FPS * 5) {  // A cada 5 segundos
                ESP_LOGI(TAG, "Stats: frames=%lu chunks_sent=%lu chunks_dropped=%lu",
                         videoModule.getFramesProcessed(),
                         videoModule.getChunksSent(),
                         videoModule.getChunksDropped());
                statsCounter = 0;
            }

            lastFrameTime = now;
        }

        // Yield para o FreeRTOS
        vTaskDelay(1);
    }
}
