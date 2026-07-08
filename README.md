<div align="center">
  <img src="assets/app-logo.svg" width="180" height="180" alt="ShallowHost" />

  <h1>ShallowHost</h1>

  <p><a href="https://github.com/opencma/LightHost">LightHost</a>'s spiritual successor for real-time audio processing via a VST2/VST3 plug-in chain</p>

  <p>
    <picture><source media="(prefers-color-scheme: dark)" srcset="https://www.shieldcn.dev/github/ci/Noktomezo/ShallowHost.svg?variant=secondary&amp;size=sm&amp;mode=dark"><img alt="CI" src="https://www.shieldcn.dev/github/ci/Noktomezo/ShallowHost.svg?variant=secondary&amp;size=sm&amp;mode=light"></picture>
    <picture><source media="(prefers-color-scheme: dark)" srcset="https://www.shieldcn.dev/github/release/Noktomezo/ShallowHost.svg?size=sm&amp;mode=dark"><img alt="Release" src="https://www.shieldcn.dev/github/release/Noktomezo/ShallowHost.svg?size=sm&amp;mode=light"></picture>
    <picture><source media="(prefers-color-scheme: dark)" srcset="https://www.shieldcn.dev/github/license/Noktomezo/ShallowHost.svg?variant=ghost&amp;size=sm&amp;mode=dark"><img alt="License" src="https://www.shieldcn.dev/github/license/Noktomezo/ShallowHost.svg?variant=ghost&amp;size=sm&amp;mode=light"></picture>
    <picture><source media="(prefers-color-scheme: dark)" srcset="https://www.shieldcn.dev/github/stars/Noktomezo/ShallowHost.svg?variant=secondary&amp;size=sm&amp;mode=dark"><img alt="Stars" src="https://www.shieldcn.dev/github/stars/Noktomezo/ShallowHost.svg?variant=secondary&amp;size=sm&amp;mode=light"></picture>
    <picture><source media="(prefers-color-scheme: dark)" srcset="https://www.shieldcn.dev/github/last-commit/Noktomezo/ShallowHost.svg?variant=secondary&amp;size=sm&amp;mode=dark"><img alt="Last commit" src="https://www.shieldcn.dev/github/last-commit/Noktomezo/ShallowHost.svg?variant=secondary&amp;size=sm&amp;mode=light"></picture>
    <picture><source media="(prefers-color-scheme: dark)" srcset="https://www.shieldcn.dev/badge/Language-TypeScript-3178C6.svg?logo=typescript&amp;variant=branded&amp;size=sm&amp;mode=dark"><img alt="TypeScript" src="https://www.shieldcn.dev/badge/Language-TypeScript-3178C6.svg?logo=typescript&amp;variant=branded&amp;size=sm&amp;mode=light"></picture>
    <picture><source media="(prefers-color-scheme: dark)" srcset="https://www.shieldcn.dev/badge/Package_mgr-Bun-000000.svg?logo=bun&amp;variant=branded&amp;size=sm&amp;mode=dark"><img alt="Bun" src="https://www.shieldcn.dev/badge/Package_mgr-Bun-000000.svg?logo=bun&amp;variant=branded&amp;size=sm&amp;mode=light"></picture>
    <picture><source media="(prefers-color-scheme: dark)" srcset="https://www.shieldcn.dev/badge/Bundler-Vite-646CFF.svg?logo=vite&amp;variant=branded&amp;size=sm&amp;mode=dark"><img alt="Vite" src="https://www.shieldcn.dev/badge/Bundler-Vite-646CFF.svg?logo=vite&amp;variant=branded&amp;size=sm&amp;mode=light"></picture>
    <picture><source media="(prefers-color-scheme: dark)" srcset="https://www.shieldcn.dev/badge/Stack-React-61DAFB.svg?logo=react&amp;variant=branded&amp;size=sm&amp;mode=dark"><img alt="React" src="https://www.shieldcn.dev/badge/Stack-React-61DAFB.svg?logo=react&amp;variant=branded&amp;size=sm&amp;mode=light"></picture>
    <picture><source media="(prefers-color-scheme: dark)" srcset="https://www.shieldcn.dev/badge/Stack-Tailwind_CSS-06B6D4.svg?logo=tailwindcss&amp;variant=branded&amp;size=sm&amp;mode=dark"><img alt="Tailwind CSS" src="https://www.shieldcn.dev/badge/Stack-Tailwind_CSS-06B6D4.svg?logo=tailwindcss&amp;variant=branded&amp;size=sm&amp;mode=light"></picture>
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

&nbsp;

<div align="center">
  <img src="./assets/footer.svg" alt="heartbeat" width="600px">
  <p>Made with 💜. Published under <a href="LICENSE">MIT license</a>.</p>
</div>
