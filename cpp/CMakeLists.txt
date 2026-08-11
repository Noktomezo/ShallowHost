cmake_minimum_required(VERSION 3.22)

# Keep the distributable self-contained instead of requiring users to install
# the Microsoft Visual C++ Redistributable separately. Rust's `crt-static`
# C++ dependencies use the release static CRT and iterator ABI in every Cargo
# profile, so JUCE must do the same even when its own optimisation level is Debug.
set(CMAKE_MSVC_RUNTIME_LIBRARY "MultiThreaded")

project(ShallowHostCpp VERSION 0.1.0 LANGUAGES C CXX)

set(CMAKE_CXX_STANDARD 20)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_CXX_EXTENSIONS OFF)

include(FetchContent)

# JUCE still expects the discontinued legacy header layout when VST2 hosting is
# enabled. Xaymar/vst2sdk is a BSD-3-Clause clean-room ABI definition; the
# project-local compatibility headers only translate its names to JUCE's API.
FetchContent_Declare(
    xaymar_vst2sdk
    GIT_REPOSITORY https://github.com/Xaymar/vst2sdk.git
    GIT_TAG        339d4f31590bf77c0d0d248e09a380ac6285e069
    GIT_SHALLOW    FALSE
    SOURCE_SUBDIR  cmake-skip
)
FetchContent_MakeAvailable(xaymar_vst2sdk)

set(JUCE_LOCAL_DIR "${CMAKE_CURRENT_SOURCE_DIR}/../tooling/vendor/juce")

if(EXISTS "${JUCE_LOCAL_DIR}/CMakeLists.txt")
    add_subdirectory(${JUCE_LOCAL_DIR} juce)
    set(JUCE_SOURCE_ROOT "${JUCE_LOCAL_DIR}")
    set(JUCE_MODULES_DIR "${JUCE_LOCAL_DIR}/modules")
else()
    message(STATUS "JUCE not found at ${JUCE_LOCAL_DIR}, downloading via FetchContent...")
    FetchContent_Declare(
        juce
        GIT_REPOSITORY https://github.com/juce-framework/JUCE.git
        GIT_TAG        9.0.0
    )
    FetchContent_MakeAvailable(juce)

    FetchContent_GetProperties(juce)
    set(JUCE_SOURCE_ROOT "${juce_SOURCE_DIR}")
    set(JUCE_MODULES_DIR "${juce_SOURCE_DIR}/modules")
endif()

# WaveShell changes the factory's visible class ordering after receiving the
# host context. JUCE normally resolves a numeric class index before that
# call, so the index can point at another Waves sub-plugin during creation.
set(JUCE_WAVESHELL_PATCH
    "${CMAKE_CURRENT_SOURCE_DIR}/patches/juce-vst3-waveshell-class-index.patch")
execute_process(
    COMMAND git apply --check "${JUCE_WAVESHELL_PATCH}"
    WORKING_DIRECTORY "${JUCE_SOURCE_ROOT}"
    RESULT_VARIABLE JUCE_PATCH_CAN_APPLY
    ERROR_VARIABLE JUCE_PATCH_CHECK_ERROR
)
if(JUCE_PATCH_CAN_APPLY EQUAL 0)
    execute_process(
        COMMAND git apply --whitespace=nowarn "${JUCE_WAVESHELL_PATCH}"
        WORKING_DIRECTORY "${JUCE_SOURCE_ROOT}"
        RESULT_VARIABLE JUCE_PATCH_RESULT
        ERROR_VARIABLE JUCE_PATCH_ERROR
    )
    if(NOT JUCE_PATCH_RESULT EQUAL 0)
        message(FATAL_ERROR "Failed to apply JUCE WaveShell patch: ${JUCE_PATCH_ERROR}")
    endif()
    message(STATUS "Applied JUCE WaveShell class-index patch")
else()
    execute_process(
        COMMAND git apply --reverse --check "${JUCE_WAVESHELL_PATCH}"
        WORKING_DIRECTORY "${JUCE_SOURCE_ROOT}"
        RESULT_VARIABLE JUCE_PATCH_ALREADY_APPLIED
        ERROR_VARIABLE JUCE_PATCH_REVERSE_ERROR
    )
    if(NOT JUCE_PATCH_ALREADY_APPLIED EQUAL 0)
        message(FATAL_ERROR
            "JUCE WaveShell patch is incompatible with this JUCE source. "
            "Apply check: ${JUCE_PATCH_CHECK_ERROR}; "
            "reverse check: ${JUCE_PATCH_REVERSE_ERROR}")
    endif()
endif()

add_library(engine STATIC
    src/host.cpp
    src/host_audio.cpp
    src/host_chain.cpp
    src/host_scan.cpp
    src/host_params.cpp
    src/ffi.cpp
)

target_compile_definitions(engine PRIVATE
    JUCE_PLUGINHOST_VST=1
    JUCE_PLUGINHOST_VST3=1
    JUCE_WASAPI=1
    JUCE_ASIO=1
    JUCE_ASIO_USE_EXTERNAL_SDK=0
    JUCE_MODAL_LOOPS_PERMITTED=1
)

if(MSVC)
    # `cxx-build` uses the release MSVC STL ABI in every Cargo profile.
    # CMake defines `_DEBUG` for its Debug configuration, so pin the iterator
    # ABI explicitly instead of letting the STL select the incompatible level 2.
    target_compile_definitions(engine PRIVATE _ITERATOR_DEBUG_LEVEL=0)
    # Preserve per-function/data COMDATs so lld-link can discard unreachable
    # JUCE code from the static archive. /GL is deliberately unavailable here:
    # lld-link cannot consume MSVC LTCG objects.
    target_compile_options(engine PRIVATE /Gy /Gw /Zc:inline)
endif()

# Rust links this static library with lld-link, which cannot consume MSVC LTCG
# objects produced by /GL. Rust-side release LTO remains enabled independently.
set_target_properties(engine PROPERTIES
    INTERPROCEDURAL_OPTIMIZATION_DEBUG FALSE
    INTERPROCEDURAL_OPTIMIZATION_RELEASE FALSE
)

target_include_directories(engine PRIVATE
    ${CMAKE_CURRENT_SOURCE_DIR}/src
    ${CMAKE_CURRENT_SOURCE_DIR}/third_party/xaymar_vst2_juce/include
    ${xaymar_vst2sdk_SOURCE_DIR}/include
    ${JUCE_MODULES_DIR}
)

# Xaymar's header must be parsed at global scope. JUCE includes the legacy
# compatibility headers inside namespace Vst2, where parsing vst.h directly
# would incorrectly move the clean-room C ABI declarations into that namespace.
set(SHALLOW_VST2_PREINCLUDE
    "${CMAKE_CURRENT_SOURCE_DIR}/third_party/xaymar_vst2_juce/xaymar_vst2_preinclude.h")
if(MSVC)
    target_compile_options(engine PRIVATE "/FI${SHALLOW_VST2_PREINCLUDE}")
else()
    target_compile_options(engine PRIVATE "-include" "${SHALLOW_VST2_PREINCLUDE}")
endif()

target_link_libraries(engine PRIVATE
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
