/**
 * @file AviMjpegWriter.h
 * @brief Escritor de container AVI com codec MJPEG.
 *
 * Implementa a criação de ficheiros AVI contendo vídeo MJPEG
 * (Motion JPEG). Cada frame é um JPEG completo (I-frame only),
 * sem compressão inter-frame.
 *
 * Formato AVI suportado:
 * - Container: RIFF AVI 1.0 + OpenDML
 * - Codec: MJPG (Motion JPEG)
 * - Resolução: Configurável (padrão VGA 640x480)
 * - Frame rate: Configurável
 *
 * @version 2.0.0
 * @author ShegaPT
 * @license GPLv3
 */

#ifndef AVI_MJPEG_WRITER_H
#define AVI_MJPEG_WRITER_H

#include <stdint.h>
#include <stddef.h>

#ifdef __cplusplus

class AviMjpegWriter {
public:
    AviMjpegWriter();
    ~AviMjpegWriter();

    /**
     * @brief Inicializa o escritor AVI.
     * @param width Largura da imagem em pixels.
     * @param height Altura da imagem em pixels.
     * @param fps Frame rate pretendido.
     * @return true em sucesso, false em erro.
     */
    bool begin(uint16_t width, uint16_t height, uint32_t fps);

    /**
     * @brief Escreve uma frame JPEG no container AVI.
     * @param jpegData Ponteiro para os dados JPEG da frame.
     * @param jpegLen Tamanho dos dados JPEG em bytes.
     * @return true em sucesso, false em erro.
     */
    bool writeFrame(const uint8_t* jpegData, size_t jpegLen);

    /**
     * @brief Finaliza o container AVI e retorna o resultado.
     *
     * Este método completa o ficheiro AVI com o cabeçalho final
     * (incluindo contagem de frames) e o índice.
     *
     * @param output Buffer de saída para o AVI completo.
     * @param outputSize Output: tamanho total do AVI em bytes.
     * @return true em sucesso, false em erro.
     */
    bool finalize(uint8_t* output, size_t* outputSize);

    /**
     * @brief Reseta o escritor para começar um novo AVI.
     */
    void reset();

    /**
     * @brief Retorna o tamanho atual do buffer em construção.
     */
    size_t getCurrentSize() const;

    /**
     * @brief Retorna o número de frames escritas.
     */
    uint32_t getFrameCount() const;

    /**
     * @brief Retorna a largura configurada.
     */
    uint16_t getWidth() const;

    /**
     * @brief Retorna a altura configurada.
     */
    uint16_t getHeight() const;

    /**
     * @brief Retorna o frame rate configurado.
     */
    uint32_t getFps() const;

    bool isInitialized() const { return _initialized; }

private:
    bool _initialized;
    uint16_t _width;
    uint16_t _height;
    uint32_t _fps;
    uint32_t _frameCount;

    // Buffer interno para construção do AVI
    uint8_t* _buffer;
    size_t _bufferCapacity;
    size_t _offset;

    // Offset do início da lista 'movi'
    size_t _moviListOffset;

    // Índice
    struct AviIndexEntry {
        uint32_t offset;
        uint32_t size;
    };
    AviIndexEntry* _index;
    uint32_t _indexCount;
    uint32_t _indexCapacity;

    bool _ensureCapacity(size_t additionalBytes);
    void _writeHeader();
    void _writeDword(uint32_t value);
    void _writeWord(uint16_t value);
    void _writeBytes(const uint8_t* data, size_t len);
    void _writeString(const char* str, size_t len);
};

#endif /* __cplusplus */
#endif /* AVI_MJPEG_WRITER_H */
