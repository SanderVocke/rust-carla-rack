cmake_minimum_required(VERSION 3.15)
project(carla-sys-native LANGUAGES NONE)

if(NOT DEFINED CARLA_SOURCE_DIR)
  message(FATAL_ERROR "CARLA_SOURCE_DIR is required")
endif()

add_subdirectory("${CARLA_SOURCE_DIR}/cmake" carla)

target_compile_definitions(carla-audio-decoder
  PRIVATE
    MINIMP3_NO_SIMD
    DR_MP3_NO_SIMD
    __aarch64__
)
