# ShallowHost patch

This directory vendors `gpui-updater` v0.0.6 (`e8a85768793ac3238b0c1bc1f32d6a1544667ec3`).

The Windows MSI handoff uses a UTF-16 VBScript instead of a CMD file. Windows
Script Host executes the handoff without opening a terminal while preserving
the upstream install, wait, and relaunch behaviour. The handoff also preserves
custom install directories and falls back to the package's default location if
Windows Installer migrates an older installation there.
