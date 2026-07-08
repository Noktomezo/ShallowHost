# Changelog

All notable changes to ShallowHost are documented here.

## v0.1.2

Code quality pass + CI improvements.

### ⚡ Performance

- Parallelized plugin chain removal (`Promise.all` instead of sequential awaits)
- Sidebar animation optimized (transform-only, no layout thrashing)
- Channel checkbox lookups use `Set` for O(1) instead of `Array.includes`

### 🎨 UI/UX

- Stable channel checkbox keys (label-based instead of index)

### 🔧 Build & CI

- React Doctor + Fallow configs added (`fallow.toml`, `doctor.config.ts`) — both score 100/100
- React Doctor CI: `fetch-depth: 0` for proper PR baseline comparison

## v0.1.1

Quick fixes + README redesign + NSIS/updater config.

### 🐛 Bug Fixes

- Fixed `latest.json` not generated on release (enabled `createUpdaterArtifacts` in tauri.conf.json)

### 🔧 Build & CI

- NSIS-only bundle (dropped MSI)
- Updater endpoint configured (`releases/latest/download/latest.json`)
- `mainBinaryName: "ShallowHost"` (Tauri-native, replaces `[[bin]]` hack)
- Bundle metadata: publisher, descriptions, copyright
- Window: center + shadow + dark backgroundColor (no white flash on startup)
- Updater: passive install mode (progress bar, no dialogs)

### 📝 Docs

- Fancy README: centered SVG logo, shieldcn badges, features, getting started, project structure, scripts, acknowledgments, footer.svg

## v0.1.0

First public release. Graphical shell for scanning, loading, and hosting VST2/VST3 plugins with real-time audio processing.

### ✨ Features

- 🎛️ **VST2/VST3 plugin hosting** — JUCE `AudioProcessorGraph` with drag-and-drop chain reordering
- 🔊 **WASAPI + ASIO drivers** — switchable audio backends with per-driver device enumeration
- 🎚️ **Mono/stereo toggle** — sliding capsule design, max-per-sample mono summing (no phase cancellation)
- 📁 **Custom search paths** — configure VST2/VST3 scan directories
- 🌍 **i18n** — Russian + English with system language detection
- 🔗 **Plugin chain** — add, remove, reorder, bypass, clear chain from home card header
- 🔄 **Auto-updater** — manual + automatic update check with download progress toast
- 🔌 **Audio hotplug** — 500ms polling, auto-recovery on device disconnect (`__none` fallback), dropdown refresh on connect
- 📌 **System tray** — show/quit menu, close-to-tray, left-click to show
- 🚀 **Autostart** — launch on OS login with `--autostart` flag, tray-to-tray option
- 🎯 **Single-instance** — focuses existing window on second launch
- 🛡️ **Prevent-default** — disables context menu, devtools, shortcuts in release builds (dev keeps F12/Ctrl+R)
- 🏷️ **Plugin badges** — Active/Bypassed on chain cards, In-chain + format on plugin list
- ✨ **Page transitions** — subtle slide-up + fade on route change
- ⚡ **Route preload** — TanStack Router `preload: 'intent'` (hover/focus)
- 🗂️ **Device memory** — ASIO + WASAPI devices/channels remembered across driver switches

### 🐛 Bug Fixes

- Fixed WASAPI default device mute on startup
- Fixed ASIO zero-checkbox playback (no channels selected = no sound)
- Fixed WASAPI switching latency (removed JUCE `Thread::sleep(1500)` + double device open, 3-5s → instant)
- Fixed WASAPI default crash + race condition on driver switch
- Fixed plugin params not persisting on exit (`ExitRequested` force-exit skipped persist)
- Fixed ASIO device name corruption when switching to WASAPI (stale `devices` state)
- Fixed mono volume loss from phase cancellation (`(L+R)*0.5` → max-per-sample)
- Fixed JUCE version mismatch breaking CI release builds (8.0.4 → 8.0.12)
- Fixed mock update toast never re-appearing after dismiss (`shownRef` guard)

### ⚡ Performance

- Optimized WASAPI driver switching (3-5s → ~instant)
- Sped up plugin scanner (avoid `knownPluginList.clear()`)
- Vite `manualChunks` optimization + `chunkSizeWarningLimit` increase

### 🎨 UI/UX

- Flexoki-inspired color palette (light + dark)
- Purple theme for mono active state
- Custom titlebar with traffic-light buttons (yellow/green/red)
- Animated select chevron + marquee text overflow on hover
- `cursor-pointer` on all interactive elements (buttons, selects, switches)
- Text selection disabled globally (re-enabled for inputs)
- Sidebar tooltips in collapsed mode
- Alphabetical plugin sorting
- Badge component with purple/green/destructive variants
- Sonner toast themed to app palette

### 🔧 Build & CI

- JUCE auto-download via CMake `FetchContent`
- DLL bundled as NSIS installer resource
- Pre-commit hooks (`cargo fmt`/`clippy`, `eslint`)
- Commitlint + lint-staged
- GitHub Actions: release, version bump, react-doctor
- NSIS installer + uninstaller icon
- Tauri signing keypair (password-protected)
- `[[bin]] name = "ShallowHost"` (cargo binary name aligned with Tauri bundle)
