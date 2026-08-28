/**
 * @file main.cpp
 * @brief Entry point do módulo VISOR — teste USB serial (ESP32 DevKitV1).
 *
 * Modo de teste para ESP32 DevKitV1 sem camera real:
 * - Usa TestCamera com frames sintéticos gerados em runtime
 * - Envia dados ACP via USB serial (UART0)
 * - Protocolo completo ACP v3.0.0 com CRC16 + Assinatura
 *
 * Pipeline de teste:
 * 1. Inicializar TestCamera (frames sintéticos)
 * 2. Configurar transporte USB serial
 * 3. Loop: capturar frame → processar → fragmentar TLV → enviar via serial
 *
 * @version 3.0.0
 * @author ShegaPT
 * @license GPLv3
 */

#include <stdio.h>
#include <freertos/FreeRTOS.h>
#include <freertos/task.h>
#include "esp_log.h"
#include "esp_timer.h"
#include "driver/uart.h"

#include "Camera.h"
#include "TestCamera.h"
#include "CameraConfig.h"
#include "Video.h"
#include "VideoProcessor.h"
#include "transport.h"
#include "protocol_ffi.h"

static const char* TAG = "VISOR";

/* ========================================================================
 * CONFIGURAÇÃO
 * ======================================================================== */

/** Frame rate alvo em FPS (reduzido para teste). */
#define TARGET_FPS          10

/** Intervalo entre frames em milissegundos. */
#define FRAME_INTERVAL_MS   (1000 / TARGET_FPS)

/** UART para envio de dados (UART0 = USB Serial). */
#define UART_PORT           UART_NUM_0
#define UART_BAUD_RATE      921600

/** Tamanho do buffer de envio. */
#define SEND_BUFFER_SIZE    1024

/* ========================================================================
 * CALLBACK DE TRANSPORTE — USB SERIAL
 * ======================================================================== */

/**
 * @brief Callback de envio via USB serial (UART0).
 *
 * Envia dados ACP serializados diretamente pela UART0 (USB).
 * Formato: [length_4bytes_LE][data_bytes]
 * O Python reader sincroniza pelo start byte 0xAA dentro do bloco.
 */
static int serial_send_callback(const uint8_t* data, size_t length, void* user_data) {
    (void)user_data;

    if (!data || length == 0) {
        return -1;
    }

    // Enviar tamanho (4 bytes little-endian) para o Python reader saber
    uint8_t len_buf[4];
    len_buf[0] = (uint8_t)(length & 0xFF);
    len_buf[1] = (uint8_t)((length >> 8) & 0xFF);
    len_buf[2] = (uint8_t)((length >> 16) & 0xFF);
    len_buf[3] = (uint8_t)((length >> 24) & 0xFF);
    uart_write_bytes(UART_PORT, len_buf, 4);

    // Enviar dados ACP
    int sent = uart_write_bytes(UART_PORT, data, length);

    return (sent == (int)length) ? 0 : -1;
}

/* ========================================================================
 * ENTRY POINT
 * ======================================================================== */

extern "C" void app_main(void) {
    ESP_LOGI(TAG, "========================================");
    ESP_LOGI(TAG, "  VISOR — USB Serial Test Mode");
    ESP_LOGI(TAG, "  AERUS Project v3.0.0");
    ESP_LOGI(TAG, "========================================");

    // Configurar UART para USB serial
    const uart_config_t uart_config = {
        .baud_rate = UART_BAUD_RATE,
        .data_bits = UART_DATA_8_BITS,
        .parity = UART_PARITY_DISABLE,
        .stop_bits = UART_STOP_BITS_1,
        .flow_ctrl = UART_HW_FLOWCTRL_DISABLE,
        .source_clk = UART_SCLK_DEFAULT,
    };

    esp_err_t uart_err = uart_param_config(UART_PORT, &uart_config);
    if (uart_err != ESP_OK) {
        ESP_LOGE(TAG, "Falha ao configurar UART: %d", uart_err);
        return;
    }

    uart_err = uart_driver_install(UART_PORT, 2048, 2048, 0, NULL, 0);
    if (uart_err != ESP_OK) {
        ESP_LOGE(TAG, "Falha ao instalar driver UART: %d", uart_err);
        return;
    }

    ESP_LOGI(TAG, "UART configurada: %d baud", UART_BAUD_RATE);

    // Verificar protocolo
    ESP_LOGI(TAG, "A verificar protocolo ACP v%s...", visor_get_version());
    uint8_t test_crc = visor_calc_crc8((const uint8_t*)"VISOR", 5);
    ESP_LOGI(TAG, "CRC8 teste: 0x%02X", test_crc);

    // Configurar TestCamera (modo teste sem camera real)
    ESP_LOGI(TAG, "A inicializar TestCamera (modo teste)...");
    TestCamera testCamera;
    CameraPinConfig dummyPins = {};

    if (!testCamera.begin(dummyPins)) {
        ESP_LOGE(TAG, "Falha ao inicializar TestCamera");
        return;
    }

    testCamera.setResolution(CameraResolution::QQVGA);
    testCamera.setFormat(CameraFormat::JPEG);
    testCamera.setQuality(10);
    testCamera.setFrameCount(20);

    ESP_LOGI(TAG, "TestCamera pronto (modo sintético, 20 frames)");

    // Configurar processador de vídeo
    VideoConfig videoConfig;
    videoConfig.targetWidth = 160;
    videoConfig.targetHeight = 120;
    videoConfig.jpegQuality = 10;
    videoConfig.gammaCorrection = 1.0f;
    videoConfig.brightnessOffset = 0;
    videoConfig.contrastFactor = 1.0f;
    videoConfig.enableResize = false;
    videoConfig.enableFilters = false;

    // Configurar interface de transporte (USB serial)
    TransportInterface transport;
    transport.send = serial_send_callback;
    transport.user_data = nullptr;

    // Inicializar módulo de vídeo
    Video videoModule;
    if (!videoModule.begin(&testCamera, transport, videoConfig)) {
        ESP_LOGE(TAG, "Falha ao inicializar módulo de vídeo");
        testCamera.end();
        return;
    }

    videoModule.setDebug(true);
    ESP_LOGI(TAG, "Módulo de vídeo inicializado");

    // Enviar packet de inicialização (heartbeat) para o Python reader
    ESP_LOGI(TAG, "A enviar packet de inicialização...");
    {
        TLVMessage init_msg;
        visor_acp_init(&init_msg, ACP_GROUP_VISOR, ACP_MSG_HEARTBEAT);
        visor_add_tlv_uint8(&init_msg, ACP_FLD_SYS_STATE, 2);  // Ready
        uint8_t init_buf[256];
        ssize_t init_size = visor_build_message(&init_msg, ACP_MSG_HEARTBEAT, 0x00,
                                                 init_buf, sizeof(init_buf));
        if (init_size > 0) {
            serial_send_callback(init_buf, init_size, nullptr);
        }
    }

    // Loop principal
    ESP_LOGI(TAG, "A iniciar loop principal (target: %d FPS)...", TARGET_FPS);
    ESP_LOGI(TAG, "Dados ACP a ser enviados via USB serial @ %d baud", UART_BAUD_RATE);
    ESP_LOGI(TAG, "Executar: python3 scripts/visor_reader.py");

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
                ESP_LOGI(TAG, "Stats: frames=%lu chunks_sent=%lu chunks_dropped=%lu heap=%lu",
                         videoModule.getFramesProcessed(),
                         videoModule.getChunksSent(),
                         videoModule.getChunksDropped(),
                         (unsigned long)esp_get_free_heap_size());
                statsCounter = 0;
            }

            lastFrameTime = now;
        }

        // Yield para o FreeRTOS
        vTaskDelay(1);
    }
}
