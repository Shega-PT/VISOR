/**
 * @file transport.h
 * @brief Interface de transporte abstrata para o módulo VISOR.
 *
 * Define o tipo de callback function pointer utilizado pelo módulo Video
 * para enviar dados processados ao Master.
 *
 * O VISOR é um módulo unidirecional: apenas envia dados ao Master,
 * não recebe comunicação de retorno. O Master gerencia toda a
 * transmissão ao TELLUS (Ground Station).
 *
 * @version 2.0.0
 * @author ShegaPT
 * @license GPLv3
 */

#ifndef TRANSPORT_H
#define TRANSPORT_H

#include <stdint.h>
#include <stddef.h>

#ifdef __cplusplus
extern "C" {
#endif

/**
 * @brief Tipo da função de callback para envio de dados.
 *
 * Esta função é chamada pelo módulo Video para enviar chunks TLV
 * ao Master. O Master é responsável por toda a transmissão ao TELLUS.
 *
 * @param data Ponteiro para os dados a enviar (chunk TLV serializado).
 * @param length Tamanho dos dados em bytes.
 * @param user_data Ponteiro para dados utilizador (contexto da chamada).
 *
 * @return 0 em sucesso, valor negativo em erro.
 */
typedef int (*transport_send_fn)(const uint8_t* data, size_t length, void* user_data);

/**
 * @brief Estrutura de interface de transporte.
 *
 * Contém o ponteiro da função de callback e um ponteiro para dados
 * utilizador que será passado como contexto em cada chamada.
 */
typedef struct {
    transport_send_fn send;  /**< Função de callback para envio */
    void* user_data;        /**< Dados utilizador (contexto) */
} TransportInterface;

/* ========================================================================
 * MACROS AUXILIARES
 * ======================================================================== */

/**
 * @brief Verifica se a interface de transporte é válida.
 * @param iface Ponteiro para a interface.
 * @return true se válida, false caso contrário.
 */
static inline bool transport_is_valid(const TransportInterface* iface) {
    return (iface != NULL && iface->send != NULL);
}

/**
 * @brief Envia dados através da interface de transporte.
 * @param iface Ponteiro para a interface.
 * @param data Ponteiro para os dados.
 * @param length Tamanho dos dados.
 * @return 0 em sucesso, valor negativo em erro.
 */
static inline int transport_send(const TransportInterface* iface,
                                 const uint8_t* data, size_t length) {
    if (!transport_is_valid(iface)) {
        return -1;
    }
    return iface->send(data, length, iface->user_data);
}

#ifdef __cplusplus
}
#endif

#endif /* TRANSPORT_H */
