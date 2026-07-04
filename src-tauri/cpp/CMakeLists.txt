cmake_minimum_required(VERSION 3.22)

project(ShallowHostCpp VERSION 0.1.0 LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_CXX_EXTENSIONS OFF)

if(NOT EXISTS "${CMAKE_CURRENT_SOURCE_DIR}/../vendor/juce/CMakeLists.txt")
    message(FATAL_ERROR "JUCE 8 must be available at ../vendor/juce")
endif()

add_subdirectory(../vendor/juce juce)

add_library(shallow_host_cpp SHARED
    src/host.cpp
    src/ffi.cpp
)

target_compile_definitions(shallow_host_cpp PRIVATE
    SHALLOW_HOST_DLL_BUILD=1
    JUCE_PLUGINHOST_VST3=1
    JUCE_WASAPI=1
    JUCE_ASIO=1
    JUCE_ASIO_USE_EXTERNAL_SDK=0
    JUCE_MODAL_LOOPS_PERMITTED=1
)

target_include_directories(shallow_host_cpp PRIVATE
    ${CMAKE_CURRENT_SOURCE_DIR}/src
    ${CMAKE_CURRENT_SOURCE_DIR}/../vendor/juce/modules
)

target_link_libraries(shallow_host_cpp PRIVATE
    juce::juce_audio_basics
    juce::juce_audio_devices
    juce::juce_audio_formats
    juce::juce_audio_processors
    juce::juce_audio_utils
    juce::juce_core
    juce::juce_data_structures
    juce::juce_events
    juce::juce_graphics
    juce::juce_gui_basics
    juce::juce_gui_extra
    juce::juce_recommended_config_flags
    juce::juce_recommended_warning_flags
)
