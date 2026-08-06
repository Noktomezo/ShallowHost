use std::fmt;

pub const AUTOSTART_ARGUMENT: &str = "--autostart";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TrayAction {
    Show,
    Quit,
}

pub fn is_autostart_launch() -> bool {
    launched_via_autostart(std::env::args())
}

fn launched_via_autostart(args: impl IntoIterator<Item = String>) -> bool {
    args.into_iter()
        .any(|argument| argument == AUTOSTART_ARGUMENT)
}

#[cfg(windows)]
mod platform {
    use super::{SystemIntegrationError, TrayAction};
    use auto_launch::{AutoLaunch, AutoLaunchBuilder};
    use gpui::Window;
    use raw_window_handle::{HasWindowHandle, RawWindowHandle};
    use tray_icon::menu::{Menu, MenuEvent, MenuItem};
    use tray_icon::{
        Icon, MouseButton, MouseButtonState, TrayIcon, TrayIconBuilder, TrayIconEvent,
    };
    use windows_sys::Win32::Foundation::HWND;
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        SW_HIDE, SW_RESTORE, SetForegroundWindow, ShowWindow,
    };

    const SHOW_MENU_ID: &str = "shallow-host-tray-show";
    const QUIT_MENU_ID: &str = "shallow-host-tray-quit";

    pub struct SystemIntegration {
        autostart: AutoLaunch,
        _tray_icon: TrayIcon,
    }

    impl SystemIntegration {
        pub fn new(show_label: &str, quit_label: &str) -> Result<Self, SystemIntegrationError> {
            let executable =
                std::env::current_exe().map_err(SystemIntegrationError::CurrentExecutable)?;
            let executable = executable
                .to_str()
                .ok_or(SystemIntegrationError::NonUnicodeExecutable)?;
            // auto-launch writes this string as a command, so quote it for paths containing spaces.
            let executable = format!(r#""{executable}""#);
            let autostart = AutoLaunchBuilder::new()
                .set_app_name("ShallowHost")
                .set_app_path(&executable)
                .set_args(&[super::AUTOSTART_ARGUMENT])
                .build()
                .map_err(|error| SystemIntegrationError::Autostart(error.to_string()))?;

            let show = MenuItem::with_id(SHOW_MENU_ID, show_label, true, None);
            let quit = MenuItem::with_id(QUIT_MENU_ID, quit_label, true, None);
            let menu = Menu::with_items(&[&show, &quit])
                .map_err(|error| SystemIntegrationError::Tray(error.to_string()))?;
            let icon = Icon::from_resource(1, Some((32, 32)))
                .map_err(|error| SystemIntegrationError::Tray(error.to_string()))?;
            let tray_icon = TrayIconBuilder::new()
                .with_tooltip(crate::APP_TITLE)
                .with_icon(icon)
                .with_menu(Box::new(menu))
                .with_menu_on_left_click(false)
                .build()
                .map_err(|error| SystemIntegrationError::Tray(error.to_string()))?;

            Ok(Self {
                autostart,
                _tray_icon: tray_icon,
            })
        }

        pub fn sync_autostart(&self, enabled: bool) -> Result<(), SystemIntegrationError> {
            if enabled {
                // Rewrite the command even when the registry entry exists: this is a portable app
                // and the executable may have moved since autostart was first enabled.
                return self.set_autostart(true);
            }
            let current = self
                .autostart
                .is_enabled()
                .map_err(|error| SystemIntegrationError::Autostart(error.to_string()))?;
            if !current {
                return Ok(());
            }
            self.set_autostart(false)
        }

        pub fn set_autostart(&self, enabled: bool) -> Result<(), SystemIntegrationError> {
            let result = if enabled {
                self.autostart.enable()
            } else {
                self.autostart.disable()
            };
            result.map_err(|error| SystemIntegrationError::Autostart(error.to_string()))
        }

        pub fn poll_tray_action(&self) -> Option<TrayAction> {
            let mut action = None;
            while let Ok(event) = MenuEvent::receiver().try_recv() {
                if event.id().as_ref() == QUIT_MENU_ID {
                    return Some(TrayAction::Quit);
                }
                if event.id().as_ref() == SHOW_MENU_ID {
                    action = Some(TrayAction::Show);
                }
            }
            while let Ok(event) = TrayIconEvent::receiver().try_recv() {
                if matches!(
                    event,
                    TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    }
                ) {
                    action = Some(TrayAction::Show);
                }
            }
            action
        }
    }

    fn native_handle(window: &Window) -> Result<HWND, SystemIntegrationError> {
        let handle = HasWindowHandle::window_handle(window)
            .map_err(|error| SystemIntegrationError::WindowHandle(error.to_string()))?;
        match handle.as_raw() {
            RawWindowHandle::Win32(handle) => {
                // HWND is the same pointer-sized Win32 handle represented by NonZeroIsize.
                Ok(handle.hwnd.get() as HWND)
            }
            _ => Err(SystemIntegrationError::UnsupportedWindowHandle),
        }
    }

    pub fn hide_window(window: &Window) -> Result<(), SystemIntegrationError> {
        let handle = native_handle(window)?;
        // SAFETY: `handle` comes from this live GPUI Window and is used synchronously on its UI
        // thread. GPUI exposes no safe Windows API for hiding an individual window.
        unsafe {
            ShowWindow(handle, SW_HIDE);
        }
        Ok(())
    }

    pub fn show_window(window: &Window) -> Result<(), SystemIntegrationError> {
        let handle = native_handle(window)?;
        // SAFETY: `handle` comes from this live GPUI Window and is used synchronously on its UI
        // thread. GPUI exposes activation but no safe API to show a previously hidden window.
        unsafe {
            ShowWindow(handle, SW_RESTORE);
            SetForegroundWindow(handle);
        }
        window.activate_window();
        Ok(())
    }
}

#[cfg(not(windows))]
mod platform {
    use super::{SystemIntegrationError, TrayAction};
    use gpui::Window;

    pub struct SystemIntegration;

    impl SystemIntegration {
        pub fn new(_show_label: &str, _quit_label: &str) -> Result<Self, SystemIntegrationError> {
            Err(SystemIntegrationError::UnsupportedPlatform)
        }

        pub fn sync_autostart(&self, _enabled: bool) -> Result<(), SystemIntegrationError> {
            Err(SystemIntegrationError::UnsupportedPlatform)
        }

        pub fn set_autostart(&self, _enabled: bool) -> Result<(), SystemIntegrationError> {
            Err(SystemIntegrationError::UnsupportedPlatform)
        }

        pub fn poll_tray_action(&self) -> Option<TrayAction> {
            None
        }
    }

    pub fn hide_window(_window: &Window) -> Result<(), SystemIntegrationError> {
        Err(SystemIntegrationError::UnsupportedPlatform)
    }

    pub fn show_window(_window: &Window) -> Result<(), SystemIntegrationError> {
        Err(SystemIntegrationError::UnsupportedPlatform)
    }
}

pub use platform::{SystemIntegration, hide_window, show_window};

#[derive(Debug)]
pub enum SystemIntegrationError {
    CurrentExecutable(std::io::Error),
    NonUnicodeExecutable,
    Autostart(String),
    Tray(String),
    WindowHandle(String),
    UnsupportedWindowHandle,
    #[cfg(not(windows))]
    UnsupportedPlatform,
}

impl fmt::Display for SystemIntegrationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CurrentExecutable(error) => {
                write!(formatter, "cannot locate executable: {error}")
            }
            Self::NonUnicodeExecutable => formatter.write_str("executable path is not valid UTF-8"),
            Self::Autostart(error) => write!(formatter, "autostart integration failed: {error}"),
            Self::Tray(error) => write!(formatter, "tray integration failed: {error}"),
            Self::WindowHandle(error) => write!(formatter, "cannot access window handle: {error}"),
            Self::UnsupportedWindowHandle => formatter.write_str("window is not a Win32 window"),
            #[cfg(not(windows))]
            Self::UnsupportedPlatform => formatter.write_str("system integration is unsupported"),
        }
    }
}

impl std::error::Error for SystemIntegrationError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_only_explicit_autostart_argument() {
        assert!(launched_via_autostart([
            String::from("ShallowHost.exe"),
            String::from(AUTOSTART_ARGUMENT),
        ]));
        assert!(!launched_via_autostart([
            String::from("ShallowHost.exe"),
            String::from("--autostart-to-tray"),
        ]));
    }
}
