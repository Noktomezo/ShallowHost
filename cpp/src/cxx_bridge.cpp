#include "shallow-host-gpui/src/infrastructure/engine/ffi.rs.h"
#include "native_host.h"

#include <string>

namespace shallow_host {
namespace {

std::string toStdString(rust::Str value)
{
    return std::string(value.data(), value.size());
}

} // namespace

void init() { native::init(); }
void shutdown() { native::shutdown(); }
void set_data_dir(rust::Str path) { native::setDataDir(toStdString(path)); }

bool audio_start(rust::Str driver, rust::Str input, rust::Str output,
                 std::int32_t sample_rate, std::int32_t buffer_size,
                 std::int32_t input_mask, std::int32_t output_mask, bool mono)
{
    return native::audioStart(toStdString(driver), toStdString(input), toStdString(output),
                              sample_rate, buffer_size, input_mask, output_mask, mono);
}

bool audio_stop() { return native::audioStop(); }

NativeAudioLevels audio_levels()
{
    const auto levels = native::audioLevels();
    return NativeAudioLevels { levels.input, levels.output };
}

rust::String audio_devices(rust::Str driver, rust::Str device)
{
    return native::audioDevices(toStdString(driver), toStdString(device));
}

rust::String scan_plugins(rust::Str paths) { return native::scanPlugins(toStdString(paths)); }
rust::String start_plugin_scan(rust::Str paths) { return native::startPluginScan(toStdString(paths)); }
rust::String scan_next_plugin() { return native::scanNextPlugin(); }
rust::String add_to_chain(rust::Str id) { return native::addToChain(toStdString(id)); }
void clear_chain() { native::clearChain(); }
bool remove_from_chain(rust::Str id) { return native::removeFromChain(toStdString(id)); }
bool reorder_chain(rust::Str id, std::int32_t index) { return native::reorderChain(toStdString(id), index); }
bool bypass_plugin(rust::Str id, bool bypassed) { return native::bypassPlugin(toStdString(id), bypassed); }
rust::String chain() { return native::chain(); }
rust::String parameters(rust::Str id) { return native::parameters(toStdString(id)); }
bool open_plugin_gui(rust::Str id, rust::Str title) { return native::openPluginGui(toStdString(id), toStdString(title)); }
bool plugin_gui_open(rust::Str id) { return native::pluginGuiOpen(toStdString(id)); }
rust::String save_state() { return native::saveState(); }
bool load_state(rust::Str state) { return native::loadState(toStdString(state)); }
std::uint64_t state_revision() { return native::stateRevision(); }
void set_mono_mode(bool mono) { native::setMonoMode(mono); }

} // namespace shallow_host
