<div align="center">
  <img src="assets/thumbnail.png" width="100%" alt="ShallowHost" />

  <p><a href="https://github.com/opencma/LightHost">LightHost</a>'s spiritual successor for real-time audio processing through a VST3 plug-in chain</p>

  <p>
    <picture><source media="(prefers-color-scheme: dark)" srcset="https://www.shieldcn.dev/github/ci/Noktomezo/ShallowHost.svg?variant=secondary&amp;size=xs&amp;mode=dark&amp;theme=neutral"><img alt="CI" src="https://www.shieldcn.dev/github/ci/Noktomezo/ShallowHost.svg?variant=secondary&amp;size=xs&amp;mode=light&amp;theme=neutral"></picture>
    <picture><source media="(prefers-color-scheme: dark)" srcset="https://www.shieldcn.dev/github/release/Noktomezo/ShallowHost.svg?size=xs&amp;mode=dark&amp;theme=neutral"><img alt="Release" src=""></picture>
    <picture><source media="(prefers-color-scheme: dark)" srcset="https://www.shieldcn.dev/github/license/Noktomezo/ShallowHost.svg?variant=ghost&amp;size=xs&amp;mode=dark&amp;theme=neutral"><img alt="License" src="https://www.shieldcn.dev/github/license/Noktomezo/ShallowHost.svg?variant=ghost&amp;size=xs&amp;mode=light&amp;theme=neutral"></picture>
    <picture><source media="(prefers-color-scheme: dark)" srcset="https://www.shieldcn.dev/github/stars/Noktomezo/ShallowHost.svg?variant=secondary&amp;size=xs&amp;mode=dark&amp;theme=neutral"><img alt="Stars" src=""></picture>
    <picture><source media="(prefers-color-scheme: dark)" srcset=""><img alt="Last commit" src=""></picture>
  </p>
</div>

---

> [!WARNING]
> Early development stage. Expect UI and audio bugs and unhandled crashes.

A native Windows host for scanning, loading, and running VST3 effects in a real-time microphone/audio chain. The interface is written in Rust with GPUI; JUCE 9 provides the C++ audio engine and `AudioProcessorGraph`.

## ✨ Features

- 🎛️ **Plugin chain** — load VST3 plugins, drag-and-drop reordering, bypass controls, and native plugin editors
- 🔊 **Windows Audio + ASIO** — shared, exclusive, and low-latency Windows Audio modes plus ASIO channel routing
- 🎚️ **Mono/stereo and meters** — switch input routing and monitor live input/output levels
- 📁 **Cached scanning** — custom VST3 paths, persistent scan cache, and correct WaveShell sub-plugin loading
- 💾 **Portable state** — configuration, plugin cache, and chain state are stored beside the executable
- 🔄 **Signed auto-updater** — verifies release checksums and minisign signatures before installation
- 📌 **System integration** — tray controls, close-to-tray behavior, and autostart on sign-in
- 🌍 **i18n and themes** — Russian/English localization, system theme detection, and optional acrylic shell

## 📸 Screenshots

<div align="center" styles="display: flex; flex-direction: row;">
  <img width="100%" alt="ShallowHost_kvmkez9VJV" src="https://github.com/user-attachments/assets/97382120-45d5-40eb-adf7-f11ac1084d40" />
  <hr>
  <img width="100%" alt="ShallowHost_G3zdsMmyVf" src="https://github.com/user-attachments/assets/0256705c-7fb6-4533-a7a0-c4a492118dc4" />
</div>


## 🏗️ Architecture

```text
GPUI interface and application state (Rust)
                  │
          safe Engine facade
                  │
               cxx bridge
                  │
JUCE 9 AudioProcessorGraph engine (C++)
                  │
     VST3 · Windows Audio · ASIO
```

- `src/app.rs` is the composition root: application startup, fonts, and the main window.
- `src/domain/` contains UI-independent value types and application preferences.
- `src/infrastructure/` owns configuration, the safe `cxx` engine facade, single-instance handling, Windows integration, and updates.
- `src/ui/foundation/` contains shared assets, colors, localization, and motion primitives.
- `src/ui/components/` contains reusable GPUI controls, overlays, meters, and scrolling.
- `src/ui/state/`, `src/ui/shell/`, and `src/ui/pages/` contain interaction state, the application shell, and feature pages respectively.
- `cpp/` contains the JUCE engine and the WaveShell compatibility patch behind the `cxx` bridge.

## 🚀 Development

### Prerequisites

- Windows 10 or 11
- [Rust](https://rustup.rs/) stable toolchain
- Visual Studio Build Tools with the MSVC C++ workload
- [CMake](https://cmake.org/) and Git
- [Just](https://just.systems/) command runner
- [watchexec](https://watchexec.github.io/) for `just dev`
- [UPX](https://upx.github.io/) optionally, for compressed local release builds

JUCE 9 is used from `vendor/juce` when present; otherwise CMake fetches the pinned version automatically.

### Setup

```bash
git clone https://github.com/Noktomezo/ShallowHost.git
cd ShallowHost
just check
```

### Development

```bash
just dev
```

This runs `cargo run` and restarts the native application when Rust sources change.

### Build

```bash
just build
```

The optimized portable build is written to `target/release/` as `ShallowHost.exe` with its `assets/` directory. UPX compression is skipped automatically when UPX is unavailable.

## 🔧 Scripts

| Command | Description |
| --- | --- |
| `just dev` | Run the app with automatic restart on Rust changes |
| `just build` | Build the optimized portable application |
| `just check` | Check all Rust targets |
| `just test` | Run all tests |
| `just clippy` | Run Clippy with warnings denied |
| `just fmt` | Check Rust formatting |
| `just strict` | Run check, tests, Clippy, and formatting |
| `just clean` | Remove Cargo build artifacts |

## 📦 Releases and portable data

GitHub releases provide `ShallowHost-vX.Y.Z-windows-x86_64.zip` and `.msi` packages. The Windows C runtime is linked statically, so users do not need to install a separate redistributable.

Runtime data lives beside `ShallowHost.exe`:

```text
config.toml
cache/
├── plugins.xml
└── chain.json
```

## 🙏 Acknowledgments

- [LightHost](https://github.com/opencma/LightHost) — original inspiration and behavior reference
- [GPUI](https://www.gpui.rs/) — native GPU-accelerated application framework
- [gpui-component](https://github.com/longbridge/gpui-component) — GPUI component library
- [JUCE](https://github.com/juce-framework/JUCE) — cross-platform C++ audio framework
- [gpui-updater](https://github.com/AprilNEA/gpui-updater) — native update workflow
- [Flexoki](https://stephango.com/flexoki) — color palette inspiration

&nbsp;

<div align="center">
  <img src="assets/footer.svg" alt="heartbeat" width="600px">
  <p>Made with 💜. Published under <a href="LICENSE">MIT license</a>.</p>
</div>
