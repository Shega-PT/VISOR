/**
 * @file protocol_ffi.h
 * @brief Header C para FFI do protocolo TLV Rust.
 *
 * Este header expõe todas as funções e structs do protocolo TLV implementado
 * em Rust para uso a partir de código C/C++.
 *
 * ATENÇÃO: Este ficheiro é gerido manualmente e deve ser mantido em
 * sincronia com o Rust FFI (rust/src/protocol/ffi.rs).
 * Qualquer alteração no Rust requer atualização correspondente neste header.
 *
 * @version 2.0.0
 * @author ShegaPT
 * @license GPLv3
 */

#ifndef PROTOCOL_FFI_H
#define PROTOCOL_FFI_H

#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>

#ifdef __cplusplus
extern "C" {
#endif

/* ========================================================================
 * CONSTANTES
 * ======================================================================== */

#define VISOR_START_BYTE        0xAA
#define VISOR_MAX_TLV_DATA      32
#define VISOR_MAX_TLV_VIDEO_DATA 128
#define VISOR_MAX_TLV_FIELDS    32
#define VISOR_MAX_MESSAGE_SIZE  1093
#define VISOR_MESSAGE_HEADER_SIZE 3
#define VISOR_CHECKSUM_SIZE     1
#define VISOR_TLV_HEADER_SIZE   2

/* ========================================================================
 * IDENTIFICADORES DE MENSAGEM (MsgID)
 * ======================================================================== */

#define VISOR_MSG_HEARTBEAT     0x10
#define VISOR_MSG_TELEMETRY     0x11
#define VISOR_MSG_COMMAND       0x12
#define VISOR_MSG_ACK           0x13
#define VISOR_MSG_FAILSAFE      0x14
#define VISOR_MSG_DEBUG         0x15
#define VISOR_MSG_VIDEO         0x16
#define VISOR_MSG_SHELL         0x17
#define VISOR_MSG_SI_DATA       0x18

/* ========================================================================
 * IDENTIFICADORES DE CAMPOS TLV — GPS (0x20-0x2F)
 * ======================================================================== */

#define VISOR_FLD_GPS_LAT       0x20
#define VISOR_FLD_GPS_LON       0x21
#define VISOR_FLD_GPS_ALT       0x22
#define VISOR_FLD_GPS_SPEED     0x23
#define VISOR_FLD_GPS_COURSE    0x24
#define VISOR_FLD_GPS_SATS      0x25
#define VISOR_FLD_GPS_HDOP      0x26

/* ========================================================================
 * IDENTIFICADORES DE CAMPOS TLV — IMU (0x30-0x3F)
 * ======================================================================== */

#define VISOR_FLD_ROLL          0x30
#define VISOR_FLD_PITCH         0x31
#define VISOR_FLD_YAW           0x32
#define VISOR_FLD_ACCEL_X       0x33
#define VISOR_FLD_ACCEL_Y       0x34
#define VISOR_FLD_ACCEL_Z       0x35
#define VISOR_FLD_GYRO_X        0x36
#define VISOR_FLD_GYRO_Y        0x37
#define VISOR_FLD_GYRO_Z        0x38
#define VISOR_FLD_YAW_RATE      0x39

/* ========================================================================
 * IDENTIFICADORES DE CAMPOS TLV — VOO (0x40-0x4F)
 * ======================================================================== */

#define VISOR_FLD_ALT_GPS       0x40
#define VISOR_FLD_ALT_BARO      0x41
#define VISOR_FLD_VSPEED        0x42
#define VISOR_FLD_AIRSPEED      0x43
#define VISOR_FLD_LOOP_TIME     0x44

/* ========================================================================
 * IDENTIFICADORES DE CAMPOS TLV — ENERGIA (0x50-0x5F)
 * ======================================================================== */

#define VISOR_FLD_BATT_V        0x50
#define VISOR_FLD_BATT_I        0x51
#define VISOR_FLD_BATT_CONS     0x52
#define VISOR_FLD_BATT_TEMP     0x53
#define VISOR_FLD_BATT_SOC      0x54

/* ========================================================================
 * IDENTIFICADORES DE CAMPOS TLV — TEMPERATURA (0x60-0x6F)
 * ======================================================================== */

#define VISOR_FLD_TEMP1         0x60
#define VISOR_FLD_TEMP2         0x61
#define VISOR_FLD_TEMP3         0x62
#define VISOR_FLD_TEMP4         0x63
#define VISOR_FLD_ESP1_TEMP     0x64
#define VISOR_FLD_ESP2_TEMP     0x65

/* ========================================================================
 * IDENTIFICADORES DE CAMPOS TLV — SISTEMA (0x70-0x7F)
 * ======================================================================== */

#define VISOR_FLD_STATE         0x70
#define VISOR_FLD_MODE          0x71
#define VISOR_FLD_UPTIME        0x72
#define VISOR_FLD_FREE_HEAP     0x73
#define VISOR_FLD_CPU_LOAD      0x74
#define VISOR_FLD_ESP1_LOAD     0x75
#define VISOR_FLD_ESP2_LOAD     0x76

/* ========================================================================
 * IDENTIFICADORES DE CAMPOS TLV — FAILSAFE (0xA1-0xAF)
 * ======================================================================== */

#define VISOR_FLD_FS_REASON     0xA1
#define VISOR_FLD_FS_ACTION     0xA2
#define VISOR_FLD_FS_STATE      0xA3

/* ========================================================================
 * IDENTIFICADORES DE CAMPOS TLV — VÍDEO (0xB0-0xBF)
 * ======================================================================== */

#define VISOR_FLD_VIDEO_FRAME_ID    0xB0
#define VISOR_FLD_VIDEO_CHUNK_ID    0xB1
#define VISOR_FLD_VIDEO_TOTAL       0xB2
#define VISOR_FLD_VIDEO_PAYLOAD     0xB3

/* ========================================================================
 * COMANDOS
 * ======================================================================== */

#define VISOR_CMD_ARM           0xC0
#define VISOR_CMD_DISARM        0xC1
#define VISOR_CMD_SET_MODE      0xC2
#define VISOR_CMD_EMERGENCY     0xC3
#define VISOR_CMD_SHUTDOWN      0xC4
#define VISOR_CMD_SET_ALT       0xD0
#define VISOR_CMD_SET_SPEED     0xD1
#define VISOR_CMD_SET_PITCH     0xD2
#define VISOR_CMD_SET_ROLL      0xD3
#define VISOR_CMD_SET_YAW       0xD4
#define VISOR_CMD_SET_HEADING   0xD5
#define VISOR_CMD_SENSOR_CALIB  0xE0
#define VISOR_CMD_START_LOG     0xE1
#define VISOR_CMD_STOP_LOG      0xE2
#define VISOR_CMD_GET_ALL       0xE3
#define VISOR_CMD_NEXT_WPT      0xF0
#define VISOR_CMD_SET_RET_POINT 0xF1
#define VISOR_CMD_SET_POSITION  0xF2

/* ========================================================================
 * PRIORIDADES
 * ======================================================================== */

#define VISOR_PRIORITY_SUPER_CRITICAL   0
#define VISOR_PRIORITY_HIGH             1
#define VISOR_PRIORITY_NORMAL           2
#define VISOR_PRIORITY_LOW              3
#define VISOR_PRIORITY_SUPER_LOW        4

/* ========================================================================
 * ESTRUTURAS (layout compatível com Rust #[repr(C)])
 * ======================================================================== */

/**
 * @brief Campo TLV genérico (34 bytes total).
 *
 * Estrutura com layout idêntico ao Rust TLVField #[repr(C)].
 */
typedef struct {
    uint8_t id;
    uint8_t len;
    uint8_t data[VISOR_MAX_TLV_DATA];
} TLVField;

/**
 * @brief Mensagem TLV completa (1093 bytes total).
 *
 * Estrutura com layout idêntico ao Rust TLVMessage #[repr(C)].
 */
typedef struct {
    uint8_t start_byte;
    uint8_t msg_id;
    uint8_t tlv_count;
    TLVField tlvs[VISOR_MAX_TLV_FIELDS];
    uint8_t checksum;
} TLVMessage;

/* ========================================================================
 * FUNÇÕES FFI — CRC8
 * ======================================================================== */

uint8_t visor_calc_crc8(const uint8_t* data, size_t len);

/* ========================================================================
 * FUNÇÕES FFI — SERIALIZAÇÃO
 * ======================================================================== */

ssize_t visor_build_message(const TLVMessage* msg, uint8_t msg_id,
                            uint8_t* buffer, size_t buffer_size);

uint8_t visor_validate_message(const uint8_t* buffer, size_t length);

void visor_parse_tlv(const uint8_t* data, size_t length,
                     TLVField* output, size_t* count);

/* ========================================================================
 * FUNÇÕES FFI — ADICIONAR CAMPOS TLV
 * ======================================================================== */

void visor_add_tlv(TLVMessage* msg, uint8_t id, const uint8_t* data, uint8_t len);
void visor_add_tlv_float(TLVMessage* msg, uint8_t id, float value);
void visor_add_tlv_int32(TLVMessage* msg, uint8_t id, int32_t value);
void visor_add_tlv_uint32(TLVMessage* msg, uint8_t id, uint32_t value);
void visor_add_tlv_uint16(TLVMessage* msg, uint8_t id, uint16_t value);
void visor_add_tlv_uint8(TLVMessage* msg, uint8_t id, uint8_t value);

/* ========================================================================
 * FUNÇÕES FFI — CONVERSÃO DE BYTES
 * ======================================================================== */

void visor_float_to_bytes(float value, uint8_t* bytes);
float visor_bytes_to_float(const uint8_t* bytes);
void visor_int32_to_bytes(int32_t value, uint8_t* bytes);
int32_t visor_bytes_to_int32(const uint8_t* bytes);
void visor_uint32_to_bytes(uint32_t value, uint8_t* bytes);
uint32_t visor_bytes_to_uint32(const uint8_t* bytes);
void visor_uint16_to_bytes(uint16_t value, uint8_t* bytes);
uint16_t visor_bytes_to_uint16(const uint8_t* bytes);

/* ========================================================================
 * FUNÇÕES FFI — VALIDAÇÃO
 * ======================================================================== */

uint8_t visor_is_valid_msg_id(uint8_t id);
uint8_t visor_is_valid_field_id(uint8_t id);
uint8_t visor_get_msg_priority(uint8_t msg_id, uint8_t failsafe_active);

/* ========================================================================
 * FUNÇÕES FFI — PARSER
 * ======================================================================== */

typedef struct Parser Parser;

Parser* visor_parser_new(void);
void visor_parser_free(Parser* parser);
uint8_t visor_parser_feed(Parser* parser, uint8_t byte);
uint8_t visor_parser_has_message(const Parser* parser);
const TLVMessage* visor_parser_get_message(const Parser* parser);
uint8_t visor_parser_copy_message(const Parser* parser, TLVMessage* output);
void visor_parser_acknowledge(Parser* parser);
void visor_parser_reset(Parser* parser);
void visor_parser_set_max_frame_gap(Parser* parser, uint32_t micros);
uint8_t visor_parser_is_timed_out(const Parser* parser);
uint8_t visor_parser_get_last_error(const Parser* parser);
uint8_t visor_parser_get_current_state(const Parser* parser);
uint32_t visor_parser_get_success_count(const Parser* parser);
uint32_t visor_parser_get_error_count(const Parser* parser);
void visor_parser_set_debug(Parser* parser, uint8_t enable);
const char* visor_parser_state_to_string(uint8_t state);
const char* visor_parser_error_to_string(uint8_t error);

/* ========================================================================
 * FUNÇÕES FFI — UTILITÁRIOS
 * ======================================================================== */

const char* visor_get_version(void);

#ifdef __cplusplus
}
#endif

#endif /* PROTOCOL_FFI_H */
