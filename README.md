<div align="center">
  <img src="assets/thumbnail.png" width="100%" alt="ShallowHost" />

  <p>A native Windows host for real-time audio processing with VST2 and VST3 plug-ins.</p>

  <p>
    <picture><source media="(prefers-color-scheme: dark)" srcset="https://www.shieldcn.dev/github/ci/Noktomezo/ShallowHost.svg?variant=secondary&amp;size=xs&amp;mode=dark&amp;theme=neutral"><img alt="CI" src="https://www.shieldcn.dev/github/ci/Noktomezo/ShallowHost.svg?variant=secondary&amp;size=xs&amp;mode=light&amp;theme=neutral"></picture>
    <picture><source media="(prefers-color-scheme: dark)" srcset="https://www.shieldcn.dev/github/release/Noktomezo/ShallowHost.svg?size=xs&amp;mode=dark&amp;theme=neutral"><img alt="Release" src="https://www.shieldcn.dev/github/release/Noktomezo/ShallowHost.svg?size=xs&amp;mode=light&amp;theme=neutral"></picture>
    <picture><source media="(prefers-color-scheme: dark)" srcset="https://www.shieldcn.dev/github/license/Noktomezo/ShallowHost.svg?variant=ghost&amp;size=xs&amp;mode=dark&amp;theme=neutral"><img alt="License" src="https://www.shieldcn.dev/github/license/Noktomezo/ShallowHost.svg?variant=ghost&amp;size=xs&amp;mode=light&amp;theme=neutral"></picture>
    <picture><source media="(prefers-color-scheme: dark)" srcset="https://www.shieldcn.dev/github/stars/Noktomezo/ShallowHost.svg?variant=secondary&amp;size=xs&amp;mode=dark&amp;theme=neutral"><img alt="Stars" src="https://www.shieldcn.dev/github/stars/Noktomezo/ShallowHost.svg?variant=secondary&amp;size=xs&amp;mode=light&amp;theme=neutral"></picture>
  </p>
</div>

> [!WARNING]
> ShallowHost is under active development. Audio and interface bugs are still possible.

ShallowHost processes microphone or other audio input through a configurable plug-in chain. It is inspired by [LightHost](https://github.com/opencma/LightHost), with a new interface built in Rust and GPUI and an audio engine based on JUCE 9.

## ✨ Features

- VST2/VST3 scanning, native plug-in editors, bypass controls, and drag-and-drop chain ordering
- Windows Audio and ASIO with channel selection, mono/stereo routing, and level meters
- Persistent configuration, plug-in cache, and chain state stored beside the executable
- Tray integration, autostart, themes, Russian and English localization, and signed updates

## 📦 Installation

Download the latest package from [GitHub Releases](https://github.com/Noktomezo/ShallowHost/releases):

- `.msi` — regular Windows installation with automatic update support
- `.zip` — portable version

The Windows C runtime is linked statically, so no additional redistributable is required.

ShallowHost stores its runtime data beside `ShallowHost.exe`:

```text
config.toml
cache/
├── plugins.xml
└── chain.json
```

## 📸 Screenshots

<p align="center">
  <img width="100%" alt="ShallowHost main page" src="https://github.com/user-attachments/assets/97382120-45d5-40eb-adf7-f11ac1084d40" />
  <br><br>
  <img width="100%" alt="ShallowHost plug-ins page" src="https://github.com/user-attachments/assets/0256705c-7fb6-4533-a7a0-c4a492118dc4" />
</p>

## 🏗️ Architecture

The GPUI application communicates with the JUCE audio engine through a safe Rust facade and a `cxx` bridge.

```text
GPUI application (Rust)
        │
safe engine facade
        │
   cxx bridge
        │
JUCE AudioProcessorGraph (C++)
```

The main source directories are:

- `src/domain/` — application data and preferences
- `src/infrastructure/` — persistence, updates, Windows integration, and the audio-engine facade
- `src/ui/` — shared controls, application shell, state, and pages
- `cpp/` — JUCE audio engine and plug-in hosting

Legacy VST2 hosting uses the BSD-licensed clean-room interface from
[Xaymar/vst2sdk](https://github.com/Xaymar/vst2sdk); the discontinued Steinberg
VST2 SDK is not included.

## 🚀 Development

Requirements:

- Windows 10 or 11
- stable [Rust](https://rustup.rs/) toolchain
- Visual Studio Build Tools with the MSVC C++ workload
- [CMake](https://cmake.org/), Git, and [Just](https://just.systems/)
- [watchexec](https://watchexec.github.io/) for automatic restart during development

```bash
git clone https://github.com/Noktomezo/ShallowHost.git
cd ShallowHost
just check
just dev
```

Useful commands:

| Command | Purpose |
| --- | --- |
| `just dev` | Run the development build with automatic restart |
| `just build` | Build the optimized portable application |
| `just strict` | Run checks, tests, Clippy, and formatting |
| `just clean` | Remove Cargo build artifacts |

JUCE is loaded from `vendor/juce` when available. Otherwise CMake downloads the pinned version during configuration.

## 🙏 Acknowledgments

[LightHost](https://github.com/opencma/LightHost), [GPUI](https://www.gpui.rs/), [JUCE](https://github.com/juce-framework/JUCE), [gpui-updater](https://github.com/AprilNEA/gpui-updater), and the [Flexoki](https://stephango.com/flexoki) palette.

ShallowHost is distributed under the [MIT license](LICENSE).
