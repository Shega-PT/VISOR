idf_component_register(
    SRCS
        "src/Video.cpp"
        "src/VideoProcessor.cpp"
        "src/CameraOV2640.cpp"
        "src/StoredVideo.cpp"
        "src/AviMjpegWriter.cpp"
    INCLUDE_DIRS
        "include"
    REQUIRES
        driver
    PRIV_REQUIRES
        espressif__esp32-camera
)

# Linkar library estática do Rust (protocolo TLV)
set(RUST_LIB_DIR "${CMAKE_SOURCE_DIR}/lib/protocol_ffi/lib")
if(EXISTS "${RUST_LIB_DIR}/libvisor_protocol.a")
    add_prebuilt_library(rust_protocol "${RUST_LIB_DIR}/libvisor_protocol.a" REQUIRES "")
    target_link_libraries(${COMPONENT_LIB} PUBLIC rust_protocol)
    target_include_directories(${COMPONENT_LIB} PUBLIC "${CMAKE_SOURCE_DIR}/lib/protocol_ffi/include")
endif()
