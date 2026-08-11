#pragma once

#include "rust/cxx.h"

#include <cstdint>

namespace shallow_host {

struct NativeAudioLevels;

void init();
void shutdown();
void set_data_dir(rust::Str path);
bool audio_start(rust::Str driver, rust::Str input, rust::Str output,
                 std::int32_t sample_rate, std::int32_t buffer_size,
                 std::int32_t input_mask, std::int32_t output_mask, bool mono);
bool audio_stop();
NativeAudioLevels audio_levels();
rust::String audio_devices(rust::Str driver, rust::Str device);
rust::String scan_plugins(rust::Str plugin_paths_json);
rust::String start_plugin_scan(rust::Str plugin_paths_json);
rust::String scan_next_plugin();
rust::String add_to_chain(rust::Str unique_id);
void clear_chain();
bool remove_from_chain(rust::Str node_id);
bool reorder_chain(rust::Str node_id, std::int32_t to_index);
bool bypass_plugin(rust::Str node_id, bool bypassed);
rust::String chain();
rust::String parameters(rust::Str node_id);
bool open_plugin_gui(rust::Str node_id, rust::Str title_prefix);
rust::String save_state();
bool load_state(rust::Str state);
std::uint64_t state_revision();
void set_mono_mode(bool mono);

} // namespace shallow_host
