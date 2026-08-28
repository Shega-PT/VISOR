/**
 * @file Video.h
 * @brief Módulo principal de vídeo do VISOR.
 *
 * Classe principal do módulo VISOR responsável por:
 * 1. Capturar frames de uma camera (interface Camera)
 * 2. Processar imagens (VideoProcessor)
 * 3. Empacotar em AVI MJPEG (AviMjpegWriter)
 * 4. Fragmentar em chunks TLV
 * 5. Enviar ao Master via TransportInterface (callback)
 *
 * O VISOR é unidirecional: apenas envia dados ao Master,
 * não recebe comunicação de retorno.
 *
 * @version 2.0.0
 * @author ShegaPT
 * @license GPLv3
 */

#ifndef VIDEO_H
#define VIDEO_H

#include <stdint.h>
#include <stddef.h>
#include <vector>

#include "Camera.h"
#include "transport.h"
#include "VideoProcessor.h"
#include "AviMjpegWriter.h"
#include "protocol_ffi.h"

#ifdef __cplusplus

/* ========================================================================
 * CONSTANTES
 * ======================================================================== */

/** Tamanho máximo de dados em cada chunk TLV (bytes). */
#define VIDEO_MAX_CHUNK_SIZE    128

/** Número máximo de chunks na fila de transmissão. */
#define VIDEO_MAX_QUEUE_SIZE    50

/** Tamanho máximo de uma frame processada (bytes). */
#ifdef BOARD_HAS_PSRAM
#define VIDEO_MAX_FRAME_SIZE    (640 * 480 * 2)  // ~600KB para VGA (com PSRAM)
#else
#define VIDEO_MAX_FRAME_SIZE    (320 * 240 * 2)  // ~150KB para QVGA (sem PSRAM)
#endif

/** Timeout para frames incompletas em milissegundos. */
#define VIDEO_FRAGMENT_TIMEOUT_MS  5000

/* ========================================================================
 * ENUMS
 * ======================================================================== */

/** Estado do módulo de vídeo. */
enum class VideoStatus : uint8_t {
    IDLE = 0,       /**< Aguardando dados */
    RECEIVING = 1,  /**< Recebendo frame */
    PROCESSING = 2, /**< Processando fragmentação */
    SENDING = 3,    /**< Enviando chunks */
    ERROR = 4       /**< Erro */
};

/* ========================================================================
 * ESTRUTURAS
 * ======================================================================== */

/** Chunk de vídeo para transmissão. */
struct VideoChunk {
    uint16_t frameId;       /**< ID da frame */
    uint8_t chunkId;        /**< Índice do chunk na frame */
    uint8_t totalChunks;    /**< Total de chunks na frame */
    uint8_t data[VIDEO_MAX_CHUNK_SIZE]; /**< Dados do chunk */
    size_t dataLen;         /**< Tamanho dos dados */
    uint32_t timestamp;     /**< Timestamp em ms */
};

/* ========================================================================
 * CLASSE PRINCIPAL
 * ======================================================================== */

/**
 * @brief Módulo principal de vídeo do VISOR.
 *
 * Coordena todo o pipeline de processamento de vídeo desde a captura
 * da camera até o envio de chunks TLV ao Master.
 */
class Video {
public:
    Video();
    ~Video();

    /**
     * @brief Inicializa o módulo de vídeo.
     * @param camera Ponteiro para a camera a utilizar.
     * @param transport Interface de transporte para envio ao Master.
     * @param config Configuração do processador de vídeo.
     * @return true em sucesso, false em erro.
     */
    bool begin(Camera* camera, TransportInterface transport, const VideoConfig& config);

    /**
     * @brief Processa e envia uma frame completa.
     *
     * Pipeline:
     * 1. Captura frame da camera
     * 2. Processa imagem (resize, filtros)
     * 3. Empacota em AVI
     * 4. Fragmenta em chunks TLV
     * 5. Envia via callback
     *
     * @return true em sucesso, false em erro.
     */
    bool processAndSend();

    /**
     * @brief Verifica se o módulo está habilitado.
     */
    bool isEnabled() const;

    /**
     * @brief Ativa ou desativa o módulo.
     */
    void setEnabled(bool enable);

    /**
     * @brief Ativa ou desativa a saída de debug.
     */
    void setDebug(bool enable);

    /** @brief Retorna o número de frames processadas. */
    uint32_t getFramesProcessed() const;

    /** @brief Retorna o número de chunks enviados com sucesso. */
    uint32_t getChunksSent() const;

    /** @brief Retorna o número de chunks descartados (fila cheia). */
    uint32_t getChunksDropped() const;

    /** @brief Reseta as estatísticas. */
    void resetStats();

    /** @brief Retorna o estado atual. */
    VideoStatus getStatus() const;

    /** @brief Retorna o tamanho da fila de chunks. */
    size_t getQueueSize() const;

    /** @brief Desliga o módulo e liberta recursos. */
    void end();

private:
    Camera* _camera;
    TransportInterface _transport;
    VideoProcessor _processor;
    AviMjpegWriter _aviWriter;

    std::vector<VideoChunk> _queue;
    uint16_t _nextFrameId;
    bool _enabled;
    bool _debug;
    VideoStatus _status;

    // Estatísticas
    uint32_t _framesProcessed;
    uint32_t _chunksSent;
    uint32_t _chunksDropped;
    uint32_t _chunksFailed;

    bool _fragmentFrame(const uint8_t* data, size_t len, uint16_t frameId);
    bool _sendChunk(const VideoChunk& chunk);
    void _sendChunks();
    size_t _buildVideoPacket(const VideoChunk& chunk, uint8_t* buffer, size_t size);
    bool _enqueueChunk(const VideoChunk& chunk);
    void _dequeueChunk();
};

#endif /* __cplusplus */
#endif /* VIDEO_H */
