/**
 * @file test_video_ffi.cpp
 * @brief Testes de integração — Video e Protocolo FFI.
 */

#include <unity.h>
#include <cstring>
#include "Video.h"
#include "VideoProcessor.h"
#include "AviMjpegWriter.h"
#include "protocol_ffi.h"
#include "transport.h"

static bool g_callback_called = false;

static int test_send_callback(const uint8_t* data, size_t length, void* user_data) {
    (void)data; (void)length; (void)user_data;
    g_callback_called = true;
    return 0;
}

void setUp(void) { g_callback_called = false; }
void tearDown(void) {}

void test_video_processor_creation(void) {
    VideoProcessor processor;
    TEST_ASSERT_FALSE(processor.isActive());
}

void test_avi_writer_creation(void) {
    AviMjpegWriter writer;
    TEST_ASSERT_FALSE(writer.isActive());
}

void test_video_creation(void) {
    Video video;
    TEST_ASSERT_FALSE(video.isEnabled());
    TEST_ASSERT_EQUAL(0u, video.getFramesProcessed());
    TEST_ASSERT_EQUAL(0u, video.getChunksSent());
}

void test_video_set_enabled(void) {
    Video video;
    video.setEnabled(true);
    video.setEnabled(false);
    TEST_ASSERT_FALSE(video.isEnabled());
}

void test_video_process_and_send(void) {
    Video video;
    bool result = video.processAndSend();
    TEST_ASSERT_FALSE(result);
}

void test_video_reset_stats(void) {
    Video video;
    video.resetStats();
    TEST_ASSERT_EQUAL(0u, video.getFramesProcessed());
}

void test_video_get_status(void) {
    Video video;
    TEST_ASSERT_EQUAL(VideoStatus::IDLE, video.getStatus());
}

void test_video_end(void) {
    Video video;
    video.end();
    TEST_ASSERT_FALSE(video.isEnabled());
}

void test_ffi_version(void) {
    const char* version = visor_get_version();
    TEST_ASSERT_NOT_NULL(version);
}

void test_ffi_calc_crc8(void) {
    uint8_t data[] = "TEST";
    uint8_t result = visor_calc_crc8(data, 4);
    TEST_ASSERT_TRUE(result <= 0xFF);
}

void test_ffi_calc_crc8_empty(void) {
    uint8_t result = visor_calc_crc8(nullptr, 0);
    TEST_ASSERT_EQUAL(0x00, result);
}

void test_ffi_build_message(void) {
    TLVMessage msg;
    memset(&msg, 0, sizeof(msg));
    msg.start_byte = VISOR_START_BYTE;
    ssize_t result = visor_build_message(&msg, VISOR_MSG_HEARTBEAT, nullptr, 0);
    TEST_ASSERT_TRUE(result <= 0);
}

void test_ffi_validate_message_valid(void) {
    TLVMessage msg;
    memset(&msg, 0, sizeof(msg));
    msg.start_byte = VISOR_START_BYTE;
    msg.msg_id = VISOR_MSG_HEARTBEAT;
    msg.tlv_count = 0;
    uint8_t buffer[64];
    ssize_t built = visor_build_message(&msg, VISOR_MSG_HEARTBEAT, buffer, sizeof(buffer));
    TEST_ASSERT_TRUE(built > 0);
    uint8_t result = visor_validate_message(buffer, (size_t)built);
    TEST_ASSERT_NOT_EQUAL(0xFF, result);
}

void test_ffi_validate_message_invalid(void) {
    uint8_t buffer[] = {0x00, 0x10, 0x00};
    uint8_t result = visor_validate_message(buffer, sizeof(buffer));
    TEST_ASSERT_EQUAL(0xFF, result);
}

void test_ffi_add_tlv_uint8(void) {
    TLVMessage msg;
    memset(&msg, 0, sizeof(msg));
    visor_add_tlv_uint8(&msg, VISOR_FLD_VIDEO_CHUNK_ID, 5);
    TEST_ASSERT_EQUAL(1, msg.tlv_count);
}

void test_ffi_add_tlv_uint16(void) {
    TLVMessage msg;
    memset(&msg, 0, sizeof(msg));
    visor_add_tlv_uint16(&msg, VISOR_FLD_VIDEO_FRAME_ID, 42);
    TEST_ASSERT_EQUAL(1, msg.tlv_count);
}

void test_ffi_add_tlv_uint32(void) {
    TLVMessage msg;
    memset(&msg, 0, sizeof(msg));
    visor_add_tlv_uint32(&msg, VISOR_FLD_FREE_HEAP, 0xDEADBEEF);
    TEST_ASSERT_EQUAL(1, msg.tlv_count);
}

void test_ffi_add_tlv_int32(void) {
    TLVMessage msg;
    memset(&msg, 0, sizeof(msg));
    visor_add_tlv_int32(&msg, VISOR_FLD_LOOP_TIME, -12345);
    TEST_ASSERT_EQUAL(1, msg.tlv_count);
}

void test_ffi_add_tlv_float(void) {
    TLVMessage msg;
    memset(&msg, 0, sizeof(msg));
    visor_add_tlv_float(&msg, VISOR_FLD_GPS_LAT, 37.7749f);
    TEST_ASSERT_EQUAL(1, msg.tlv_count);
}

void test_ffi_add_tlv_raw(void) {
    TLVMessage msg;
    memset(&msg, 0, sizeof(msg));
    uint8_t data[] = {0x01, 0x02, 0x03};
    visor_add_tlv(&msg, VISOR_FLD_VIDEO_PAYLOAD, data, 3);
    TEST_ASSERT_EQUAL(1, msg.tlv_count);
}

void test_ffi_float_to_bytes(void) {
    uint8_t bytes[4];
    visor_float_to_bytes(3.14f, bytes);
    float recovered = visor_bytes_to_float(bytes);
    TEST_ASSERT_FLOAT_WITHIN(0.001f, 3.14f, recovered);
}

void test_ffi_int32_to_bytes(void) {
    uint8_t bytes[4];
    visor_int32_to_bytes(-12345, bytes);
    int32_t recovered = visor_bytes_to_int32(bytes);
    TEST_ASSERT_EQUAL(-12345, recovered);
}

void test_ffi_uint32_to_bytes(void) {
    uint8_t bytes[4];
    visor_uint32_to_bytes(0xDEADBEEF, bytes);
    uint32_t recovered = visor_bytes_to_uint32(bytes);
    TEST_ASSERT_EQUAL(0xDEADBEEFu, recovered);
}

void test_ffi_uint16_to_bytes(void) {
    uint8_t bytes[2];
    visor_uint16_to_bytes(65535, bytes);
    uint16_t recovered = visor_bytes_to_uint16(bytes);
    TEST_ASSERT_EQUAL(65535, recovered);
}

void test_ffi_is_valid_msg_id(void) {
    TEST_ASSERT_EQUAL(1, visor_is_valid_msg_id(VISOR_MSG_HEARTBEAT));
    TEST_ASSERT_EQUAL(0, visor_is_valid_msg_id(0xFF));
}

void test_ffi_is_valid_field_id(void) {
    TEST_ASSERT_EQUAL(1, visor_is_valid_field_id(VISOR_FLD_GPS_LAT));
    TEST_ASSERT_EQUAL(0, visor_is_valid_field_id(0xFF));
}

void test_ffi_get_msg_priority(void) {
    TEST_ASSERT_EQUAL(VISOR_PRIORITY_SUPER_CRITICAL, visor_get_msg_priority(VISOR_MSG_FAILSAFE, 0));
    TEST_ASSERT_EQUAL(VISOR_PRIORITY_HIGH, visor_get_msg_priority(VISOR_MSG_COMMAND, 0));
    TEST_ASSERT_EQUAL(VISOR_PRIORITY_NORMAL, visor_get_msg_priority(VISOR_MSG_HEARTBEAT, 0));
    TEST_ASSERT_EQUAL(VISOR_PRIORITY_LOW, visor_get_msg_priority(VISOR_MSG_VIDEO, 0));
    TEST_ASSERT_EQUAL(VISOR_PRIORITY_SUPER_LOW, visor_get_msg_priority(VISOR_MSG_DEBUG, 0));
}

void test_ffi_parser_lifecycle(void) {
    Parser* parser = visor_parser_new();
    TEST_ASSERT_NOT_NULL(parser);

    TEST_ASSERT_EQUAL(0, visor_parser_has_message(parser));

    visor_parser_feed(parser, VISOR_START_BYTE);
    visor_parser_feed(parser, VISOR_MSG_HEARTBEAT);
    visor_parser_feed(parser, 0);

    TEST_ASSERT_EQUAL(1, visor_parser_has_message(parser));

    visor_parser_acknowledge(parser);
    TEST_ASSERT_EQUAL(0, visor_parser_has_message(parser));

    visor_parser_free(parser);
}

int main(void) {
    UNITY_BEGIN();

    RUN_TEST(test_video_processor_creation);
    RUN_TEST(test_avi_writer_creation);
    RUN_TEST(test_video_creation);
    RUN_TEST(test_video_set_enabled);
    RUN_TEST(test_video_process_and_send);
    RUN_TEST(test_video_reset_stats);
    RUN_TEST(test_video_get_status);
    RUN_TEST(test_video_end);

    RUN_TEST(test_ffi_version);
    RUN_TEST(test_ffi_calc_crc8);
    RUN_TEST(test_ffi_calc_crc8_empty);
    RUN_TEST(test_ffi_build_message);
    RUN_TEST(test_ffi_validate_message_valid);
    RUN_TEST(test_ffi_validate_message_invalid);
    RUN_TEST(test_ffi_add_tlv_uint8);
    RUN_TEST(test_ffi_add_tlv_uint16);
    RUN_TEST(test_ffi_add_tlv_uint32);
    RUN_TEST(test_ffi_add_tlv_int32);
    RUN_TEST(test_ffi_add_tlv_float);
    RUN_TEST(test_ffi_add_tlv_raw);
    RUN_TEST(test_ffi_float_to_bytes);
    RUN_TEST(test_ffi_int32_to_bytes);
    RUN_TEST(test_ffi_uint32_to_bytes);
    RUN_TEST(test_ffi_uint16_to_bytes);
    RUN_TEST(test_ffi_is_valid_msg_id);
    RUN_TEST(test_ffi_is_valid_field_id);
    RUN_TEST(test_ffi_get_msg_priority);
    RUN_TEST(test_ffi_parser_lifecycle);

    return UNITY_END();
}
