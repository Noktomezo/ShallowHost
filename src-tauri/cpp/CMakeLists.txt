cmake_minimum_required(VERSION 3.22)

project(ShallowHostCpp VERSION 0.1.0 LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_CXX_EXTENSIONS OFF)

include(FetchContent)

set(JUCE_LOCAL_DIR "${CMAKE_CURRENT_SOURCE_DIR}/../vendor/juce")

if(EXISTS "${JUCE_LOCAL_DIR}/CMakeLists.txt")
    add_subdirectory(${JUCE_LOCAL_DIR} juce)
    set(JUCE_MODULES_DIR "${JUCE_LOCAL_DIR}/modules")
else()
    message(STATUS "JUCE not found at ${JUCE_LOCAL_DIR}, downloading via FetchContent...")
    FetchContent_Declare(
        juce
        GIT_REPOSITORY https://github.com/juce-framework/JUCE.git
        GIT_TAG        8.0.4
    )
    FetchContent_MakeAvailable(juce)

    FetchContent_GetProperties(juce)
    set(JUCE_MODULES_DIR "${juce_SOURCE_DIR}/modules")
endif()

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
    ${JUCE_MODULES_DIR}
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
