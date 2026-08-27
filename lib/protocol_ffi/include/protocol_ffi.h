/**
 * @file protocol_ffi.h
 * @brief Header C para FFI do protocolo ACP (AERUS Communication Protocol) v3.0.0.
 *
 * Este header expõe todas as funções e structs do protocolo ACP implementado
 * em Rust para uso a partir de código C/C++.
 *
 * Formato da Mensagem ACP v3.0.0:
 * [START_BYTE][VERSION][NODE_ID][MSG_ID][SEQ_NUM(2)][TLV_COUNT]
 * [TLV_FIELDS...][SIGNATURE][CRC16(2)]
 *
 * ATENÇÃO: Este ficheiro é gerido manualmente e deve ser mantido em
 * sincronia com o Rust FFI (rust/src/protocol/ffi.rs e rust/src/parser/ffi.rs).
 * Qualquer alteração no Rust requer atualização correspondente neste header.
 *
 * @version 3.0.0
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
 * CONSTANTES — ACP v3.0.0
 * ======================================================================== */

#define ACP_START_BYTE          0xAA
#define ACP_VERSION             0x03
#define ACP_MAX_TLV_DATA        32
#define ACP_MAX_TLV_VIDEO_DATA  128
#define ACP_MAX_TLV_FIELDS      32
#define ACP_HEADER_SIZE         7
#define ACP_SIGNATURE_SIZE      1
#define ACP_CRC16_SIZE          2
#define ACP_OVERHEAD            10
#define ACP_MAX_MESSAGE_SIZE    1098
#define ACP_TLV_HEADER_SIZE     2

/* ========================================================================
 * IDENTIFICADORES DE MENSAGEM (MsgID)
 * ======================================================================== */

#define ACP_MSG_HEARTBEAT       0x10
#define ACP_MSG_TELEMETRY       0x11
#define ACP_MSG_COMMAND         0x12
#define ACP_MSG_ACK             0x13
#define ACP_MSG_FAILSAFE        0x14
#define ACP_MSG_DEBUG           0x15
#define ACP_MSG_VIDEO           0x16
#define ACP_MSG_SHELL           0x17
#define ACP_MSG_SI_DATA         0x18
#define ACP_MSG_WATCHDOG        0x19
#define ACP_MSG_PING            0x1A
#define ACP_MSG_CLOCK           0x1B

/* ========================================================================
 * TIPOS DE DADO TLV (3 bits = 8 valores)
 * ======================================================================== */

#define ACP_TYPE_RAW            0
#define ACP_TYPE_FLOAT32        1
#define ACP_TYPE_FLOAT16        2
#define ACP_TYPE_INT32          3
#define ACP_TYPE_UINT32         4
#define ACP_TYPE_UINT16         5
#define ACP_TYPE_UINT8          6
#define ACP_TYPE_BOOL           7

/* ========================================================================
 * GRUPOS COMPUTACIONAIS (CAN)
 * ======================================================================== */

#define ACP_GROUP_NONE          0x0
#define ACP_GROUP_RASPBERRYPI   0x1
#define ACP_GROUP_ESP32S        0x2
#define ACP_GROUP_ESP32A        0x3
#define ACP_GROUP_ESP32FS       0x4
#define ACP_GROUP_ESP32FSA      0x5
#define ACP_GROUP_VISOR         0x6

/* ========================================================================
 * TIPOS DE MENSAGEM CAN
 * ======================================================================== */

#define ACP_CAN_MSG_DATA        0x0
#define ACP_CAN_MSG_CMD         0x1
#define ACP_CAN_MSG_ACK         0x2
#define ACP_CAN_MSG_EVENT       0x3
#define ACP_CAN_MSG_SYNC        0x4
#define ACP_CAN_MSG_STATE       0x5
#define ACP_CAN_MSG_HEART       0x6
#define ACP_CAN_MSG_SAFETY      0x7

/* ========================================================================
 * PRIORIDADES
 * ======================================================================== */

#define ACP_PRIORITY_SUPER_CRITICAL  0
#define ACP_PRIORITY_CRITICAL        1
#define ACP_PRIORITY_HIGH            2
#define ACP_PRIORITY_MEDIUM          3
#define ACP_PRIORITY_LOW             4

/* ========================================================================
 * IDENTIFICADORES DE CAMPOS TLV — FieldID com tipo embutido [TYPE:3][ID:5]
 * ======================================================================== */

/* GPS — Tipo 1 (f32) */
#define ACP_FLD_GPS_LAT         0x26    /* TYPE=1(f32) + ID=6 */
#define ACP_FLD_GPS_LON         0x27    /* TYPE=1(f32) + ID=7 */
#define ACP_FLD_GPS_ALT         0x28    /* TYPE=1(f32) + ID=8 */
#define ACP_FLD_GPS_SPEED       0x29    /* TYPE=1(f32) + ID=9 */
#define ACP_FLD_GPS_COURSE      0x2A    /* TYPE=1(f32) + ID=10 */
#define ACP_FLD_GPS_SATS        0xC7    /* TYPE=6(u8)  + ID=7 */
#define ACP_FLD_GPS_HDOP        0x2B    /* TYPE=1(f32) + ID=11 */

/* IMU — Tipo 1 (f32) */
#define ACP_FLD_IMU_ROLL        0x30    /* TYPE=1(f32) + ID=0x10 */
#define ACP_FLD_IMU_PITCH       0x31    /* TYPE=1(f32) + ID=0x11 */
#define ACP_FLD_IMU_YAW         0x32    /* TYPE=1(f32) + ID=0x12 */
#define ACP_FLD_IMU_ACCEL_X     0x33    /* TYPE=1(f32) + ID=0x13 */
#define ACP_FLD_IMU_ACCEL_Y     0x34    /* TYPE=1(f32) + ID=0x14 */
#define ACP_FLD_IMU_ACCEL_Z     0x35    /* TYPE=1(f32) + ID=0x15 */
#define ACP_FLD_IMU_GYRO_X      0x36    /* TYPE=1(f32) + ID=0x16 */
#define ACP_FLD_IMU_GYRO_Y      0x37    /* TYPE=1(f32) + ID=0x17 */
#define ACP_FLD_IMU_GYRO_Z      0x38    /* TYPE=1(f32) + ID=0x18 */
#define ACP_FLD_IMU_YAW_RATE    0x39    /* TYPE=1(f32) + ID=0x19 */

/* Voo — Tipos mistos */
#define ACP_FLD_FLIGHT_ALT_GPS  0x40    /* TYPE=1(f32) + ID=0x20 */
#define ACP_FLD_FLIGHT_ALT_BARO 0x41    /* TYPE=1(f32) + ID=0x21 */
#define ACP_FLD_FLIGHT_VSPEED   0x42    /* TYPE=1(f32) + ID=0x22 */
#define ACP_FLD_FLIGHT_AIRSPEED 0x43    /* TYPE=1(f32) + ID=0x23 */
#define ACP_FLD_FLIGHT_LOOP     0xA2    /* TYPE=5(u16) + ID=2 */

/* Energia — Tipo 1 (f32) */
#define ACP_FLD_POWER_BATT_V    0x50    /* TYPE=1(f32) + ID=0x30 */
#define ACP_FLD_POWER_BATT_I    0x51    /* TYPE=1(f32) + ID=0x31 */
#define ACP_FLD_POWER_BATT_CONS 0x52    /* TYPE=1(f32) + ID=0x32 */
#define ACP_FLD_POWER_BATT_TEMP 0x53    /* TYPE=1(f32) + ID=0x33 */
#define ACP_FLD_POWER_BATT_SOC  0x54    /* TYPE=1(f32) + ID=0x34 */

/* Temperatura — Tipo 1 (f32) */
#define ACP_FLD_TEMP1           0x60    /* TYPE=1(f32) + ID=0x40 */
#define ACP_FLD_TEMP2           0x61    /* TYPE=1(f32) + ID=0x41 */
#define ACP_FLD_TEMP3           0x62    /* TYPE=1(f32) + ID=0x42 */
#define ACP_FLD_TEMP4           0x63    /* TYPE=1(f32) + ID=0x43 */
#define ACP_FLD_TEMP_ESP1       0x64    /* TYPE=1(f32) + ID=0x44 */
#define ACP_FLD_TEMP_ESP2       0x65    /* TYPE=1(f32) + ID=0x45 */

/* Sistema — Tipos mistos (u8, u32) */
#define ACP_FLD_SYS_STATE       0xC0    /* TYPE=6(u8)  + ID=0 */
#define ACP_FLD_SYS_MODE        0xC1    /* TYPE=6(u8)  + ID=1 */
#define ACP_FLD_SYS_UPTIME      0x82    /* TYPE=4(u32) + ID=2 */
#define ACP_FLD_SYS_FREE_HEAP   0x83    /* TYPE=4(u32) + ID=3 */
#define ACP_FLD_SYS_CPU_LOAD    0xC4    /* TYPE=6(u8)  + ID=4 */
#define ACP_FLD_SYS_ESP1_LOAD   0xC5    /* TYPE=6(u8)  + ID=5 */
#define ACP_FLD_SYS_ESP2_LOAD   0xC6    /* TYPE=6(u8)  + ID=6 */

/* Failsafe — Tipo 6 (u8) */
#define ACP_FLD_FS_REASON       0xC8    /* TYPE=6(u8) + ID=8 */
#define ACP_FLD_FS_ACTION       0xC9    /* TYPE=6(u8) + ID=9 */
#define ACP_FLD_FS_STATE        0xCA    /* TYPE=6(u8) + ID=10 */

/* Vídeo — Tipos mistos */
#define ACP_FLD_VIDEO_FRAME_ID  0xA0    /* TYPE=5(u16) + ID=0 */
#define ACP_FLD_VIDEO_CHUNK_ID  0xC3    /* TYPE=6(u8)  + ID=3 */
#define ACP_FLD_VIDEO_TOTAL     0xCB    /* TYPE=6(u8)  + ID=11 */
#define ACP_FLD_VIDEO_PAYLOAD   0x00    /* TYPE=0(raw) + ID=0 */

/* ========================================================================
 * COMANDOS
 * ======================================================================== */

#define ACP_CMD_ARM             0xC0
#define ACP_CMD_DISARM          0xC1
#define ACP_CMD_SET_MODE        0xC2
#define ACP_CMD_EMERGENCY       0xC3
#define ACP_CMD_SHUTDOWN        0xC4
#define ACP_CMD_SET_ALT         0xD0
#define ACP_CMD_SET_SPEED       0xD1
#define ACP_CMD_SET_PITCH       0xD2
#define ACP_CMD_SET_ROLL        0xD3
#define ACP_CMD_SET_YAW         0xD4
#define ACP_CMD_SET_HEADING     0xD5
#define ACP_CMD_SENSOR_CALIB    0xE0
#define ACP_CMD_START_LOG       0xE1
#define ACP_CMD_STOP_LOG        0xE2
#define ACP_CMD_GET_ALL         0xE3
#define ACP_CMD_NEXT_WPT        0xF0
#define ACP_CMD_SET_RET_POINT   0xF1
#define ACP_CMD_SET_POSITION    0xF2

/* ========================================================================
 * ESTRUTURAS (layout compatível com Rust #[repr(C)])
 * ======================================================================== */

/**
 * @brief Campo TLV genérico (34 bytes total).
 *
 * FieldID: [TYPE:3][ID:5]
 */
typedef struct {
    uint8_t id;
    uint8_t len;
    uint8_t data[ACP_MAX_TLV_DATA];
} TLVField;

/**
 * @brief Mensagem ACP v3.0.0 completa.
 *
 * Layout:
 * [start_byte][version][node_id][msg_id][seq_num(2)][tlv_count]
 * [tlvs[0]...tlvs[N]][signature][checksum(2)]
 */
typedef struct {
    uint8_t start_byte;
    uint8_t version;
    uint8_t node_id;
    uint8_t msg_id;
    uint16_t seq_num;
    uint8_t tlv_count;
    TLVField tlvs[ACP_MAX_TLV_FIELDS];
    uint8_t signature;
    uint16_t checksum;
} TLVMessage;

/* ========================================================================
 * FUNÇÕES FFI — CRC
 * ======================================================================== */

uint16_t visor_calc_crc16(const uint8_t* data, size_t len);
uint8_t visor_calc_crc8(const uint8_t* data, size_t len);

/* ========================================================================
 * FUNÇÕES FFI — ASSINATURA
 * ======================================================================== */

uint8_t visor_compute_signature(uint8_t key, uint8_t msg_id, uint8_t seq_lo, uint8_t seq_hi);
uint8_t visor_validate_signature(uint8_t signature, uint8_t key, uint8_t msg_id,
                                 uint8_t seq_lo, uint8_t seq_hi);

/* ========================================================================
 * FUNÇÕES FFI — FIELDID COM TIPO
 * ======================================================================== */

uint8_t visor_field_id_encode(uint8_t field_type, uint8_t field_id);
void visor_field_id_decode(uint8_t field_id, uint8_t* type_out, uint8_t* id_out);
uint8_t visor_is_valid_field_id(uint8_t field_id);

/* ========================================================================
 * FUNÇÕES FFI — CAN ID
 * ======================================================================== */

uint32_t visor_make_can_id(uint8_t priority, uint8_t src_group, uint8_t dst_group, uint8_t msg_type);
uint8_t visor_can_id_priority(uint32_t can_id);
uint8_t visor_can_id_src_group(uint32_t can_id);
uint8_t visor_can_id_dst_group(uint32_t can_id);
uint8_t visor_can_id_msg_type(uint32_t can_id);
uint8_t visor_is_safety_bus_id(uint32_t can_id);

/* ========================================================================
 * FUNÇÕES FFI — INICIALIZAÇÃO DE MENSAGEM
 * ======================================================================== */

void visor_acp_init(TLVMessage* msg, uint8_t node_id, uint8_t msg_id);
void visor_acp_set_seq(TLVMessage* msg, uint16_t seq);
void visor_acp_clear(TLVMessage* msg);

/* ========================================================================
 * FUNÇÕES FFI — SERIALIZAÇÃO
 * ======================================================================== */

ssize_t visor_build_message(const TLVMessage* msg, uint8_t msg_id,
                            uint8_t signature_key, uint8_t* buffer, size_t buffer_size);

uint8_t visor_validate_message(const uint8_t* buffer, size_t length);

uint8_t visor_validate_signature_in_message(const uint8_t* buffer, size_t length,
                                            uint8_t signature_key);

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
uint8_t visor_get_msg_priority(uint8_t msg_id, uint8_t failsafe_active);

/* ========================================================================
 * FUNÇÕES FFI — PARSER
 * ======================================================================== */

typedef struct Parser Parser;

Parser* visor_parser_new(uint8_t key);
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
uint8_t visor_parser_get_key(const Parser* parser);
void visor_parser_set_key(Parser* parser, uint8_t key);
const char* visor_parser_state_to_string(uint8_t state);
const char* visor_parser_error_to_string(uint8_t error);

/* ========================================================================
 * FUNÇÕES FFI — UTILITÁRIOS
 * ======================================================================== */

const char* visor_get_version(void);
size_t visor_get_overhead(void);
size_t visor_get_max_message_size(void);

/* ========================================================================
 * CONSTANTES LEGADO (manter por retrocompatibilidade)
 * ======================================================================== */

#define VISOR_START_BYTE        ACP_START_BYTE
#define VISOR_MAX_TLV_DATA      ACP_MAX_TLV_DATA
#define VISOR_MAX_TLV_FIELDS    ACP_MAX_TLV_FIELDS
#define VISOR_MAX_MESSAGE_SIZE  ACP_MAX_MESSAGE_SIZE
#define VISOR_MSG_HEARTBEAT     ACP_MSG_HEARTBEAT
#define VISOR_MSG_TELEMETRY     ACP_MSG_TELEMETRY
#define VISOR_MSG_COMMAND       ACP_MSG_COMMAND
#define VISOR_MSG_ACK           ACP_MSG_ACK
#define VISOR_MSG_FAILSAFE      ACP_MSG_FAILSAFE
#define VISOR_MSG_DEBUG         ACP_MSG_DEBUG
#define VISOR_MSG_VIDEO         ACP_MSG_VIDEO

#ifdef __cplusplus
}
#endif

#endif /* PROTOCOL_FFI_H */
