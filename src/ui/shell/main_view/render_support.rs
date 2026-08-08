use gpui::Context;
use std::rc::Rc;
use std::time::Instant;

use super::MainView;
use crate::ui::shell::routes::{
    AudioMeterState, DropdownCallbacks, Language, PluginPathUpdate, RenderContext, Route,
    SystemSetting, ThemeMode,
};
use crate::ui::state::audio_controls::ChannelDirection;

impl MainView {
    pub(super) fn render_context(&self) -> RenderContext {
        RenderContext {
            selected_theme: self.selected_theme,
            selected_language: self.selected_language,
            transparent_shell: self.transparent_shell,
            transparency_changed_at: self.transparency_changed_at,
            scan_paths_open: self.scan_paths_open,
            audio_controls: self.audio_controls.clone(),
            is_mono: self.is_mono,
            mono_changed_at: self.mono_changed_at,
            plugin_settings: self.storage.config().plugins.clone(),
            system_settings: self.storage.config().system.clone(),
            system_changed_at: self.system_changed_at,
            meter: AudioMeterState {
                input_level: self.input_level,
                output_level: self.output_level,
                input_peak: self
                    .input_peak_until
                    .is_some_and(|until| until > Instant::now()),
                output_peak: self
                    .output_peak_until
                    .is_some_and(|until| until > Instant::now()),
            },
            theme_dropdown_motion: self.theme_dropdown_motion.clone(),
            language_dropdown_motion: self.language_dropdown_motion.clone(),
            plugin_scan_state: self.plugin_scan_state.clone(),
            chain_operation_state: self.chain_operation_state.clone(),
            updater: self.updater.clone(),
        }
    }

    pub(super) fn dropdown_callbacks(cx: &mut Context<Self>) -> DropdownCallbacks {
        let theme = cx.listener(|this: &mut Self, value: &ThemeMode, window, cx| {
            this.set_theme(*value, window, cx);
        });
        let language = cx.listener(|this: &mut Self, value: &Language, _window, cx| {
            this.set_language(*value, cx);
        });
        let transparency = cx.listener(|this: &mut Self, value: &bool, window, cx| {
            this.set_transparent_shell(*value, window, cx);
        });
        let navigate = cx.listener(|this: &mut Self, value: &Route, _window, cx| {
            this.navigate(*value, cx);
        });
        let mono = cx.listener(|this: &mut Self, value: &bool, _window, cx| {
            this.set_mono(*value, cx);
        });
        let audio_routing = cx.listener(
            |this: &mut Self, value: &(ChannelDirection, Vec<usize>, bool), _window, cx| {
                this.toggle_audio_channels(value.0, &value.1, value.2, cx);
            },
        );
        let system = cx.listener(
            |this: &mut Self, value: &(SystemSetting, bool), _window, cx| {
                this.set_system_setting(value.0, value.1, cx);
            },
        );
        let scan_paths_visibility = cx.listener(|this: &mut Self, value: &bool, _window, cx| {
            this.set_scan_paths_open(*value, cx);
        });
        let plugin_path = cx.listener(|this: &mut Self, value: &PluginPathUpdate, _window, cx| {
            this.update_plugin_path(value.clone(), cx);
        });
        let plugin_path_picker = cx.listener(|this: &mut Self, _value: &(), window, cx| {
            this.pick_plugin_path(window, cx);
        });

        DropdownCallbacks {
            on_change_theme: Rc::new(move |value, window, cx| theme(&value, window, cx)),
            on_change_language: Rc::new(move |value, window, cx| language(&value, window, cx)),
            on_change_transparency: Rc::new(move |value, window, cx| {
                transparency(&value, window, cx);
            }),
            on_navigate: Rc::new(move |value, window, cx| navigate(&value, window, cx)),
            on_set_mono: Rc::new(move |value, window, cx| mono(&value, window, cx)),
            on_change_system: Rc::new(move |setting, enabled, window, cx| {
                system(&(setting, enabled), window, cx);
            }),
            on_set_scan_paths_open: Rc::new(move |value, window, cx| {
                scan_paths_visibility(&value, window, cx);
            }),
            on_update_plugin_path: Rc::new(move |value, window, cx| {
                plugin_path(&value, window, cx);
            }),
            on_pick_plugin_path: Rc::new(move |window, cx| {
                plugin_path_picker(&(), window, cx);
            }),
            on_change_audio_routing: Rc::new(move |direction, indices, enabled, window, cx| {
                audio_routing(&(direction, indices, enabled), window, cx);
            }),
        }
    }
}
