use gpui::*;
use std::rc::Rc;
use std::sync::Arc;
use std::time::Instant;

pub use crate::domain::preferences::{Language, ThemeMode};
use crate::infrastructure::config::{PluginSettings, SystemSettings};
use crate::infrastructure::engine::Engine;
use gpui_updater::Updater;

use crate::ui::components::text_input::TextInputState;
use crate::ui::foundation::motion::DropdownMotion;
use crate::ui::pages::plugins::{PluginLibraryState, PluginScanState};
use crate::ui::pages::{HomePage, PluginsPage, SettingsPage};
use crate::ui::state::audio_controls::{AudioControls, ChannelDirection};
use crate::ui::state::chain_operations::ChainOperationState;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Route {
    Home,
    Plugins,
    Settings,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SystemSetting {
    Autostart,
    AutostartToTray,
    MinimizeToTray,
    AutoCheckUpdates,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PluginPathUpdate {
    Add { kind: PluginPathKind, path: String },
    Remove { kind: PluginPathKind, path: String },
    Reset,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PluginPathKind {
    Vst2,
    Vst3,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct AudioMeterState {
    pub input_level: f32,
    pub output_level: f32,
    pub input_peak: bool,
    pub output_peak: bool,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct ScanPathsDialogState {
    pub open: bool,
    pub closing: bool,
    pub revision: u64,
}

impl ScanPathsDialogState {
    pub const fn visible(self) -> bool {
        self.open || self.closing
    }
}

pub struct RenderContext {
    pub selected_theme: ThemeMode,
    pub selected_language: Language,
    pub transparent_shell: bool,
    pub transparency_changed_at: Option<Instant>,
    pub scan_paths: ScanPathsDialogState,
    pub audio_controls: AudioControls,
    pub is_mono: bool,
    pub mono_changed_at: Option<Instant>,
    pub plugin_settings: PluginSettings,
    pub system_settings: SystemSettings,
    pub system_changed_at: [Option<Instant>; 4],
    pub meter: AudioMeterState,
    pub theme_dropdown_motion: Entity<DropdownMotion>,
    pub language_dropdown_motion: Entity<DropdownMotion>,
    pub plugin_scan_state: Entity<PluginScanState>,
    pub plugin_search: Entity<TextInputState>,
    pub plugin_library_state: Entity<PluginLibraryState>,
    pub chain_operation_state: Entity<ChainOperationState>,
    pub updater: Entity<Updater>,
}

pub type ThemeCallback = Rc<dyn Fn(ThemeMode, &mut Window, &mut App)>;
pub type LanguageCallback = Rc<dyn Fn(Language, &mut Window, &mut App)>;
pub type TransparencyCallback = Rc<dyn Fn(bool, &mut Window, &mut App)>;
pub type NavigateCallback = Rc<dyn Fn(Route, &mut Window, &mut App)>;
pub type MonoCallback = Rc<dyn Fn(bool, &mut Window, &mut App)>;
pub type SystemCallback = Rc<dyn Fn(SystemSetting, bool, &mut Window, &mut App)>;
pub type ScanPathsVisibilityCallback = Rc<dyn Fn(bool, &mut Window, &mut App)>;
pub type PluginPathCallback = Rc<dyn Fn(PluginPathUpdate, &mut Window, &mut App)>;
pub type PluginPathPickerCallback = Rc<dyn Fn(PluginPathKind, &mut Window, &mut App)>;
pub type AudioRoutingCallback =
    Rc<dyn Fn(ChannelDirection, Vec<usize>, bool, &mut Window, &mut App)>;

#[derive(Clone)]
pub struct DropdownCallbacks {
    pub on_change_theme: ThemeCallback,
    pub on_change_language: LanguageCallback,
    pub on_change_transparency: TransparencyCallback,
    pub on_navigate: NavigateCallback,
    pub on_set_mono: MonoCallback,
    pub on_change_system: SystemCallback,
    pub on_set_scan_paths_open: ScanPathsVisibilityCallback,
    pub on_update_plugin_path: PluginPathCallback,
    pub on_pick_plugin_path: PluginPathPickerCallback,
    pub on_change_audio_routing: AudioRoutingCallback,
}

impl Route {
    pub fn render(
        &self,
        ctx: RenderContext,
        callbacks: &DropdownCallbacks,
        engine: Arc<Engine>,
        window: &mut Window,
        cx: &mut App,
    ) -> AnyElement {
        match self {
            Self::Home => HomePage::new(
                callbacks,
                engine,
                ctx.audio_controls,
                ctx.is_mono,
                ctx.mono_changed_at,
                ctx.meter,
                ctx.chain_operation_state,
            )
            .render(window, cx)
            .into_any_element(),
            Self::Plugins => PluginsPage::new(
                callbacks,
                engine,
                ctx.plugin_settings,
                ctx.scan_paths,
                (ctx.plugin_scan_state, ctx.plugin_library_state),
                ctx.plugin_search,
                ctx.chain_operation_state,
            )
            .render(window, cx)
            .into_any_element(),
            Self::Settings => SettingsPage::new(ctx, callbacks)
                .render(window, cx)
                .into_any_element(),
        }
    }
}

impl SystemSetting {
    pub const fn motion_index(self) -> usize {
        match self {
            Self::Autostart => 0,
            Self::AutostartToTray => 1,
            Self::MinimizeToTray => 2,
            Self::AutoCheckUpdates => 3,
        }
    }
}
