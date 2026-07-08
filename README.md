<div align="center">
  <img src="assets/app-logo.svg" width="180" height="180" alt="ShallowHost" />

  <h1>ShallowHost</h1>

  <p><a href="https://github.com/opencma/LightHost">LightHost</a>'s spiritual successor for real-time audio processing via a VST2/VST3 plug-in chain</p>

  <p>
    <a href="https://github.com/Noktomezo/ShallowHost/actions/workflows/release.yml"><img src="https://github.com/Noktomezo/ShallowHost/actions/workflows/release.yml/badge.svg" alt="Release CI" /></a>
    <a href="https://github.com/Noktomezo/ShallowHost/releases/latest"><img src="https://img.shields.io/github/v/release/Noktomezo/ShallowHost?display_name=tag&style=flat&label=latest" alt="Latest Release" /></a>
    <a href="./LICENSE"><img src="https://img.shields.io/badge/license-MIT-blue.svg" alt="MIT License" /></a>
    <img src="https://img.shields.io/badge/platform-Windows-blue" alt="Platform" />
  </p>
</div>

---

> [!CAUTION]
> Very early stage. Please do not touch this or your pc go boom or whatever

A graphical shell for scanning, loading, and hosting VST2/VST3 plugins with real-time microphone/audio effect chains. Built on a JUCE `AudioProcessorGraph` backend with a Tauri + React frontend.

## ✨ Features

- 🎛️ **Plugin chain** — load VST2/VST3 plugins, drag-and-drop reordering, bypass/active toggle
- 🔊 **WASAPI + ASIO** — switchable audio drivers with per-device channel selection
- 🎚️ **Mono/stereo** — max-per-sample mono summing without phase cancellation
- 📁 **Custom scan paths** — configure where to look for VST2/VST3 plugins
- 🔌 **Audio hotplug** — auto-recovery on device disconnect, dropdown refresh on connect
- 🔄 **Auto-updater** — checks for updates, downloads + installs with progress toast
- 📌 **System tray** — close-to-tray, show/quit menu, autostart on OS login
- 🎯 **Single-instance** — focuses the existing window on second launch
- 🌍 **i18n** — Russian + English with system language detection
- 🛡️ **Hardened release** — context menu, devtools, shortcuts disabled in production builds

## 🧱 Tech Stack

| Layer | Technology |
| --- | --- |
| Desktop shell | Tauri v2 |
| Frontend | React 19, TypeScript, Vite, TailwindCSS v4, shadcn/ui |
| Routing | TanStack Router |
| State | Zustand |
| i18n | i18next + react-i18next |
| Backend | Rust (Tauri commands) |
| Audio engine | C++ / JUCE 8 `AudioProcessorGraph` |

## 🚀 Getting Started

### Prerequisites

- [Bun](https://bun.sh/) runtime
- [Rust](https://rustup.rs/) stable toolchain
- [Just](https://just.system.ms/) command runner
- CMake + C++ compiler (MSVC on Windows)
- [Node.js](https://nodejs.org/) (for ctx7/codegraph tooling, optional)

### Setup

```bash
git clone https://github.com/Noktomezo/ShallowHost.git
cd ShallowHost
just boot     # bun install + cargo check
```

### Development

```bash
just dev      # bun run tauri dev (HMR frontend, Rust recompile on change)
```

### Build

```bash
just build    # generate icons + bun tauri build --no-sign + UPX compress
```

> Release builds with updater artifacts are produced by CI on tag push (`v*`).

## 📂 Project Structure

```
ShallowHost/
├── src/                    # Frontend (Feature-Sliced Design)
│   ├── app/                # App shell, router, global styles
│   ├── pages/             # Route pages (home, plugins, settings)
│   ├── shared/            # Shared UI, lib, config, model, api
│   └── entities/          # Domain entities
├── src-tauri/             # Backend
│   ├── src/               # Rust (commands, engine, audio)
│   ├── cpp/               # C++ audio engine (JUCE)
│   ├── capabilities/      # Tauri permissions
│   └── icons/             # App icons (generated)
├── assets/                # Source logo + design assets
├── .github/workflows/     # CI (release, bump, react-doctor)
└── justfile               # Task runner
```

## 🔧 Scripts

| Command | Description |
| --- | --- |
| `just dev` | Run in dev mode with hot reload |
| `just build` | Local installer build (no signing) |
| `just lint` | Lint backend + frontend |
| `just format` | Format backend + frontend |
| `just gen-icons` | Regenerate app icons from `assets/app-logo.svg` |
| `just clean` | Remove build artifacts |

## 🙏 Acknowledgments

- [LightHost](https://github.com/opencma/LightHost) — original inspiration and behavior reference
- [JUCE](https://github.com/juce-framework/JUCE) — cross-platform C++ audio framework
- [Tauri](https://tauri.app/) — secure, fast desktop application framework
- [shadcn/ui](https://ui.shadcn.com/) — beautifully designed component library
- [Flexoki](https://stephango.com/flexoki) — color palette inspiration

## 📄 License

[MIT](./LICENSE) © Noktomezo
