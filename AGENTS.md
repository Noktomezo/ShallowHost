# AGENTS.md

## --- Project Overview --------------------------------------------------------

**ShallowHost** - a graphical shell for scanning, loading and hosting VST2/VST3 plugins (e.g real-time microphone effects).

## Tech Stack

| Layer | Technology |
| --- | --- |
| Desktop shell | Tauri v2 |
| Frontend runtime | Bun |
| Build tool | Vite |
| UI framework | React 19 |
| Language | TypeScript (strict) |
| Styling | TailwindCSS v4 |
| Component library | shadcn/ui |
| Routing | TanStack Router |
| State management | Zustand |
| i18n | i18next + react-i18next |
| Backend | Rust (Tauri commands) |
| Notifications | Sonner |

## Folder Structure

- Frontend: Feature-Sliced Design (FSD)
- Backend: Vertical Slice Design

## Dependency Rules

### Frontend (src)

- **Runtime:** `bun` only. Never `npm`/`pnpm`/`node`.
- Install: `bun add <pkg>` / `bun add -d <pkg>`. Run: `bunx <tool>` / `bun run <script>`.
- Commit `bun.lock` only. Never `package-lock.json` or `pnpm-lock.yaml`.

### Backend (src-tauri)

- Add deps: `cargo add <crate>` (never edit `Cargo.toml` by hand). With features: `--features <f1>,<f2>`. Then `cargo check`.
- **rayon** mandatory-by-default for CPU-heavy or bounded independent IO/status work across many items (tweak statuses, registry scans, file parsing, metadata building).
- Keep `rayon` out of: strict-order, shared mutable state, UI-thread affinity, non-thread-safe COM/Win32, global process settings, service-control sequences, or where parallelism amplifies load/side effects.
- For Tauri commands: wrap blocking work in `tauri::async_runtime::spawn_blocking`, use `rayon` inside only when per-item work is independent.
- Prefer sequential when: tiny collection, already async, or parallelism makes error handling/rollback less predictable.

### Tauri

- Tauri v2 APIs only. Register commands in `lib.rs` via `invoke_handler(generate_handler![...])`. Use `#[tauri::command]` on all handlers.

## Core Priorities

Performance, reliability, predictability under load/failures. When trading off, choose correctness over convenience.

## Maintainability

Extract shared logic to modules. No duplicate logic across files. Change existing code; don't add local shortcuts.

## Codebase Navigation — `@colbymchenry/codegraph`

**MANDATORY:** Use codegraph for all codebase navigation, symbol discovery, relationship analysis. Generic grep discouraged unless searching raw literal strings.

```bash
bunx --bun @colbymchenry/codegraph init # first time
bunx --bun @colbymchenry/codegraph index # re-index after edits
bunx --bun @colbymchenry/codegraph query <symbol> # find definitions/usages
bunx --bun @colbymchenry/codegraph context "<task>" # structured markdown for a feature
bunx --bun @colbymchenry/codegraph status # graph health
```

Workflow: explore with `query`/`context` before reading files → re-index after edits → trace deps with `query` before modifications.

## RTK — Token-Optimized Commands

**Always prefix shell commands with `rtk`** (60-90% context savings, zero behavior change, passthrough if no filter).

- Chain: `rtk git add . && rtk git commit -m "msg"`
- Debugging: raw command without `rtk`
- `rtk proxy <cmd>` — no filtering, tracks usage

## Post-Task Checks

Run after every task. Do not skip.

### Frontend (format → typecheck → dead-code → audit)

```bash
bun run format # eslint --fix (eslint-stylistic replaces Prettier)
bun run typecheck # tsc --noEmit, zero errors
bunx fallow # zero issues
bunx react-doctor # UI health
```

### Backend (fmt → clippy → check)

```bash
cargo fmt
cargo clippy --fix --allow-dirty --allow-staged --all-targets -- -D warnings
cargo check
```

## --- Reference Repos ---------------------------------------------------------

- https://github.com/opencma/LightHost - original idea and behaviour reference
