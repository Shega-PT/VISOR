/**
 * @file test_transport_camera.cpp
 * @brief Testes de integração — Transporte e Camera.
 */

#include <unity.h>
#include <cstring>
#include "transport.h"
#include "CameraConfig.h"
#include "CameraOV2640.h"
#include "StoredVideo.h"

static bool g_callback_called = false;
static uint8_t g_callback_buffer[1024];
static size_t g_callback_length = 0;

static int test_send_callback(const uint8_t* data, size_t length, void* user_data) {
    (void)user_data;
    g_callback_called = true;
    g_callback_length = length;
    if (length <= sizeof(g_callback_buffer)) {
        memcpy(g_callback_buffer, data, length);
    }
    return 0;
}

static int test_send_fail_callback(const uint8_t* data, size_t length, void* user_data) {
    (void)data; (void)length; (void)user_data;
    return -1;
}

void setUp(void) {
    g_callback_called = false;
    g_callback_length = 0;
    memset(g_callback_buffer, 0, sizeof(g_callback_buffer));
}

void tearDown(void) {}

void test_transport_is_valid_with_callback(void) {
    TransportInterface transport;
    transport.send = test_send_callback;
    transport.user_data = nullptr;
    TEST_ASSERT_TRUE(transport_is_valid(&transport));
}

void test_transport_is_valid_without_callback(void) {
    TransportInterface transport;
    transport.send = nullptr;
    transport.user_data = nullptr;
    TEST_ASSERT_FALSE(transport_is_valid(&transport));
}

void test_transport_is_valid_null(void) {
    TEST_ASSERT_FALSE(transport_is_valid(nullptr));
}

void test_transport_send_success(void) {
    TransportInterface transport;
    transport.send = test_send_callback;
    transport.user_data = nullptr;
    uint8_t data[] = {0x01, 0x02, 0x03, 0x04};
    int result = transport_send(&transport, data, sizeof(data));
    TEST_ASSERT_EQUAL(0, result);
    TEST_ASSERT_TRUE(g_callback_called);
    TEST_ASSERT_EQUAL(4, g_callback_length);
    TEST_ASSERT_EQUAL_MEMORY(data, g_callback_buffer, 4);
}

void test_transport_send_failure(void) {
    TransportInterface transport;
    transport.send = test_send_fail_callback;
    transport.user_data = nullptr;
    uint8_t data[] = {0x01, 0x02, 0x03, 0x04};
    int result = transport_send(&transport, data, sizeof(data));
    TEST_ASSERT_EQUAL(-1, result);
}

void test_transport_send_null_transport(void) {
    uint8_t data[] = {0x01, 0x02};
    int result = transport_send(nullptr, data, sizeof(data));
    TEST_ASSERT_EQUAL(-1, result);
}

void test_transport_send_empty_data(void) {
    TransportInterface transport;
    transport.send = test_send_callback;
    transport.user_data = nullptr;
    int result = transport_send(&transport, nullptr, 0);
    TEST_ASSERT_EQUAL(-1, result);
}

void test_camera_preset_ai_thinker(void) {
    const CameraPinConfig& pins = CAMERA_PRESET_AI_THINKER_ESP32_CAM;
    TEST_ASSERT_EQUAL(10, pins.pwdn);
    TEST_ASSERT_EQUAL(-1, pins.reset);
    TEST_ASSERT_EQUAL(11, pins.xclk);
    TEST_ASSERT_EQUAL(46, pins.d0);
    TEST_ASSERT_EQUAL(42, pins.d1);
    TEST_ASSERT_EQUAL(48, pins.d2);
}

void test_camera_preset_esp32s3(void) {
    const CameraPinConfig& pins = CAMERA_PRESET_ESP32_S3_CAM;
    TEST_ASSERT_EQUAL(8, pins.d0);
    TEST_ASSERT_EQUAL(9, pins.d1);
    TEST_ASSERT_EQUAL(4, pins.d2);
    TEST_ASSERT_EQUAL(5, pins.d3);
}

void test_camera_ov2640_creation(void) {
    CameraOV2640 camera;
    TEST_ASSERT_FALSE(camera.isActive());
}

void test_camera_ov2640_begin(void) {
    CameraOV2640 camera;
    const CameraPinConfig& pins = CAMERA_PRESET_AI_THINKER_ESP32_CAM;
    bool result = camera.begin(pins);
    TEST_ASSERT_FALSE(result);
}

void test_camera_ov2640_capture(void) {
    CameraOV2640 camera;
    uint8_t buffer[100];
    size_t len = sizeof(buffer);
    bool result = camera.capture(buffer, &len);
    TEST_ASSERT_FALSE(result);
}

void test_camera_ov2640_end(void) {
    CameraOV2640 camera;
    camera.end();
    TEST_ASSERT_FALSE(camera.isActive());
}

void test_stored_video_creation(void) {
    StoredVideo video;
    TEST_ASSERT_FALSE(video.isActive());
}

void test_stored_video_begin(void) {
    StoredVideo video;
    bool result = video.begin();
    TEST_ASSERT_FALSE(result);
}

void test_stored_video_capture(void) {
    StoredVideo video;
    uint8_t buffer[100];
    size_t len = sizeof(buffer);
    bool result = video.capture(buffer, &len);
    TEST_ASSERT_FALSE(result);
}
