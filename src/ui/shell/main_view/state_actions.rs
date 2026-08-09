use gpui::{App, AppContext, Context, Window};
use gpui_component::theme::{Theme, ThemeMode as ComponentThemeMode};
use std::sync::Arc;
use std::time::Duration;
use std::time::Instant;

use super::MainView;
use crate::infrastructure::system::{TrayAction, hide_window, show_window};
use crate::ui::foundation::colors;
use crate::ui::foundation::i18n;
use crate::ui::foundation::motion::CONTROL_MOTION;
use crate::ui::shell::routes::{
    Language, PluginPathKind, PluginPathUpdate, Route, SystemSetting, ThemeMode,
};
use crate::ui::state::audio_controls::ChannelDirection;
use crate::ui::state::chain_operations::PendingPlugin;

impl MainView {
    pub(super) fn start_chain_restore_task(&mut self, cx: &mut Context<Self>) {
        let placeholder_engine = Arc::clone(&self.engine);
        let restore_engine = Arc::clone(&self.engine);
        let operations = self.chain_operation_state.clone();
        self._chain_restore_task = cx.spawn(async move |_, cx| {
            let placeholders = cx
                .background_spawn(async move { placeholder_engine.saved_chain_placeholders() })
                .await;
            let placeholders = match placeholders {
                Ok(placeholders) => placeholders,
                Err(error) => {
                    eprintln!("failed to read saved plugin chain: {error}");
                    return;
                }
            };
            let pending = placeholders
                .into_iter()
                .filter_map(PendingPlugin::from_chain_item)
                .collect::<Vec<_>>();
            if pending.is_empty() {
                return;
            }
            let started = operations.update(&mut *cx, |state, cx| {
                let started = state.begin_restore(pending);
                if started {
                    cx.notify();
                }
                started
            });
            if !started {
                return;
            }
            cx.refresh();

            let result = cx
                .background_spawn(async move { restore_engine.restore_chain_state() })
                .await;
            if let Err(error) = result {
                eprintln!("failed to restore plugin chain: {error}");
            }
            operations.update(&mut *cx, |state, cx| {
                state.finish_restore();
                cx.notify();
            });
            cx.refresh();
        });
    }

    pub(super) fn start_system_task(&mut self, cx: &mut Context<Self>) {
        self._system_task = cx.spawn(async move |view, cx| {
            loop {
                cx.background_executor()
                    .timer(Duration::from_millis(50))
                    .await;
                if view
                    .update_in(&mut *cx, |view, window, cx| {
                        let activation_requested = match view.single_instance.activation_requested()
                        {
                            Ok(requested) => requested,
                            Err(error) => {
                                eprintln!("failed to poll the single-instance signal: {error}");
                                false
                            }
                        };
                        let action = view
                            .system_integration
                            .as_ref()
                            .and_then(|integration| integration.poll_tray_action());
                        let should_show =
                            activation_requested || matches!(action, Some(TrayAction::Show));
                        match action {
                            Some(TrayAction::Quit) => cx.quit(),
                            _ if should_show => {
                                if let Err(error) = show_window(window) {
                                    eprintln!("failed to restore the existing window: {error}");
                                } else {
                                    cx.activate(true);
                                }
                            }
                            _ => {}
                        }
                    })
                    .is_err()
                {
                    break;
                }
            }
        });
    }

    pub(super) fn install_close_handler(&self, window: &Window, cx: &Context<Self>) {
        let view = cx.entity().downgrade();
        window.on_window_should_close(cx, move |window, cx| {
            let close_to_tray = view
                .upgrade()
                .is_some_and(|view| view.read(cx).storage.config().system.minimize_to_tray);
            if !close_to_tray {
                return true;
            }
            match hide_window(window) {
                Ok(()) => false,
                Err(error) => {
                    eprintln!("failed to hide window in tray: {error}");
                    true
                }
            }
        });
    }

    pub(super) fn close_or_hide(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        crate::ui::foundation::hover_motion::clear_all_hovers(window, cx);
        crate::ui::components::cursor_tooltip::hide(window, cx);
        if self.storage.config().system.minimize_to_tray {
            match hide_window(window) {
                Ok(()) => return,
                Err(error) => eprintln!("failed to hide window in tray: {error}"),
            }
        }
        window.remove_window();
    }

    pub(super) fn start_meter_task(&mut self, cx: &mut Context<Self>) {
        let meter_engine = Arc::clone(&self.engine);
        self._meter_task = cx.spawn(async move |view, cx| {
            loop {
                cx.background_executor()
                    .timer(Duration::from_millis(30))
                    .await;
                let engine = Arc::clone(&meter_engine);
                let levels = cx
                    .background_spawn(async move { engine.audio_levels() })
                    .await;
                if view
                    .update(&mut *cx, |view, cx| {
                        if view.current_route != Route::Home {
                            return;
                        }
                        if let Ok((input, output)) = levels {
                            view.update_meter_levels(input, output);
                            view.reset_inactive_meter_levels(cx);
                            cx.notify();
                        }
                    })
                    .is_err()
                {
                    break;
                }
            }
        });
    }

    pub(super) fn apply_and_persist_audio(&mut self, cx: &mut Context<Self>) {
        self.reset_inactive_meter_levels(cx);
        self.audio_controls.apply(&self.engine, cx, self.is_mono);
        self.persist_audio(cx);
        cx.notify();
    }

    pub(super) fn toggle_audio_channels(
        &mut self,
        direction: ChannelDirection,
        indices: &[usize],
        enabled: bool,
        cx: &mut Context<Self>,
    ) {
        self.audio_controls
            .toggle_channels(direction, indices, enabled, cx);
        self.audio_controls.remember_device_selection(cx);
        self.persist_audio(cx);
        self.audio_routing_revision = self.audio_routing_revision.wrapping_add(1);
        let revision = self.audio_routing_revision;
        self._audio_routing_task = cx.spawn(async move |this, cx| {
            cx.background_executor().timer(CONTROL_MOTION).await;
            let _intentionally_ignored = this.update(&mut *cx, |this, cx| {
                if this.audio_routing_revision != revision {
                    return;
                }
                this.reset_inactive_meter_levels(cx);
                this.audio_controls.apply(&this.engine, cx, this.is_mono);
                cx.notify();
            });
        });
        cx.notify();
    }

    pub(super) fn update_meter_levels(&mut self, input: f32, output: f32) {
        let now = Instant::now();
        let input = scale_level(input);
        let output = scale_level(output);
        self.input_level = smooth_level(self.input_level, input);
        self.output_level = smooth_level(self.output_level, output);
        update_peak_hold(&mut self.input_peak_until, input, now);
        update_peak_hold(&mut self.output_peak_until, output, now);
    }

    pub(super) fn reset_inactive_meter_levels(&mut self, cx: &App) {
        if !self.audio_controls.has_input_device(cx) {
            self.input_level = 0.0;
            self.input_peak_until = None;
        }
        if !self.audio_controls.has_output_device(cx) {
            self.output_level = 0.0;
            self.output_peak_until = None;
        }
    }

    pub(super) fn persist_audio(&mut self, cx: &App) {
        self.storage.config_mut().audio = self.audio_controls.settings(cx, self.is_mono);
        self.save_config();
    }

    fn save_config(&self) {
        if let Err(error) = self.storage.save() {
            eprintln!("failed to save portable config: {error}");
        }
    }

    pub(super) fn set_mono(&mut self, mono: bool, cx: &mut Context<Self>) {
        if self.is_mono == mono {
            return;
        }
        if let Err(error) = self.engine.set_mono_mode(mono) {
            eprintln!("failed to change mono mode: {error}");
            return;
        }
        self.is_mono = mono;
        self.mono_changed_at = Some(Instant::now());
        self.storage.config_mut().audio.is_mono = mono;
        self.save_config();
        cx.notify();
    }

    pub(super) fn toggle_sidebar(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        crate::ui::components::cursor_tooltip::hide(window, cx);
        self.sidebar_collapsed = !self.sidebar_collapsed;
        self.sidebar_motion.set_collapsed(self.sidebar_collapsed);
        cx.notify();
    }

    pub(super) fn navigate(&mut self, route: Route, cx: &mut Context<Self>) {
        if self.current_route != route {
            match self.current_route {
                Route::Home => self.audio_controls.reset_dropdown_interactions(cx),
                Route::Settings => {
                    crate::ui::foundation::motion::reset_dropdown_interaction(
                        &self.theme_dropdown_motion,
                        cx,
                    );
                    crate::ui::foundation::motion::reset_dropdown_interaction(
                        &self.language_dropdown_motion,
                        cx,
                    );
                }
                Route::Plugins => {}
            }
            let now = Instant::now();
            self.deselected_route = Some(self.current_route);
            self.deselected_at = Some(now);
            self.current_route = route;
            self.selected_at = now;
            cx.notify();
        }
    }

    pub(super) fn set_hovered_route(
        &mut self,
        route: Route,
        is_hovered: bool,
        cx: &mut Context<Self>,
    ) {
        let now = Instant::now();
        if is_hovered {
            if self.hovered_route != Some(route) {
                if let Some(old) = self.hovered_route {
                    self.unhovered_route = Some(old);
                    self.unhovered_at = Some(now);
                }
                self.hovered_route = Some(route);
                self.hovered_at = Some(now);
                cx.notify();
            }
        } else if self.hovered_route == Some(route) {
            self.unhovered_route = Some(route);
            self.unhovered_at = Some(now);
            self.hovered_route = None;
            self.hovered_at = None;
            cx.notify();
        }
    }

    pub(super) fn set_theme(
        &mut self,
        theme: ThemeMode,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.selected_theme = theme;
        self.storage.config_mut().theme = theme;
        colors::set_active_theme(theme);
        sync_component_theme(theme, window, cx);
        self.save_config();
        cx.notify();
    }

    pub(super) fn set_language(&mut self, language: Language, cx: &mut Context<Self>) {
        self.selected_language = language;
        self.storage.config_mut().language = language;
        i18n::set_language(language);
        self.save_config();
        cx.notify();
    }

    pub(super) fn set_transparent_shell(
        &mut self,
        enabled: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.transparent_shell = enabled;
        self.transparency_changed_at = Some(Instant::now());
        self.storage.config_mut().transparent_shell = enabled;
        window.set_background_appearance(if enabled {
            gpui::WindowBackgroundAppearance::Blurred
        } else {
            gpui::WindowBackgroundAppearance::Opaque
        });
        self.save_config();
        cx.notify();
    }

    pub(super) fn set_system_setting(
        &mut self,
        setting: SystemSetting,
        enabled: bool,
        cx: &mut Context<Self>,
    ) {
        if setting == SystemSetting::Autostart {
            let Some(integration) = &self.system_integration else {
                eprintln!("cannot change autostart: system integration is unavailable");
                return;
            };
            if let Err(error) = integration.set_autostart(enabled) {
                eprintln!("failed to change autostart: {error}");
                return;
            }
        }
        let system = &mut self.storage.config_mut().system;
        match setting {
            SystemSetting::Autostart => system.autostart = enabled,
            SystemSetting::AutostartToTray => system.autostart_to_tray = enabled,
            SystemSetting::MinimizeToTray => system.minimize_to_tray = enabled,
            SystemSetting::AutoCheckUpdates => system.auto_check_updates = enabled,
        }
        self.system_changed_at[setting.motion_index()] = Some(Instant::now());
        self.save_config();
        if setting == SystemSetting::AutoCheckUpdates {
            if enabled {
                crate::infrastructure::updater::start_check(&self.updater, cx);
                self.start_update_check_task(cx);
            } else {
                self.stop_update_check_task();
            }
        }
        cx.notify();
    }

    pub(super) fn update_plugin_path(&mut self, update: PluginPathUpdate, cx: &mut Context<Self>) {
        let settings = &mut self.storage.config_mut().plugins;
        match update {
            PluginPathUpdate::Add { kind, path } => {
                let paths = match kind {
                    PluginPathKind::Vst2 => &mut settings.vst2_paths,
                    PluginPathKind::Vst3 => &mut settings.vst3_paths,
                };
                if !paths.contains(&path) {
                    paths.push(path);
                }
            }
            PluginPathUpdate::Remove { kind, path } => {
                let paths = match kind {
                    PluginPathKind::Vst2 => &mut settings.vst2_paths,
                    PluginPathKind::Vst3 => &mut settings.vst3_paths,
                };
                paths.retain(|existing| existing != &path);
            }
            PluginPathUpdate::Reset => {
                *settings = crate::infrastructure::config::PluginSettings::default()
            }
        }
        self.save_config();
        cx.notify();
    }

    pub(super) fn pick_plugin_path(
        &mut self,
        kind: PluginPathKind,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let receiver = cx.prompt_for_paths(gpui::PathPromptOptions {
            files: false,
            directories: true,
            multiple: false,
            prompt: None,
        });
        cx.spawn_in(window, async move |view, cx| {
            let Ok(Ok(Some(paths))) = receiver.await else {
                return;
            };
            let Some(path) = paths.into_iter().next() else {
                return;
            };
            if view
                .update_in(cx, |view, window, cx| {
                    view.update_plugin_path(
                        PluginPathUpdate::Add {
                            kind,
                            path: path.to_string_lossy().into_owned(),
                        },
                        cx,
                    );
                    // The native directory picker may return without producing another input
                    // frame, so explicitly repaint the still-open paths dialog.
                    window.refresh();
                })
                .is_err()
            {
                eprintln!("scan path selection finished after the main view was closed");
            }
        })
        .detach();
    }
}

pub(super) fn sync_component_theme(theme: ThemeMode, window: &mut Window, cx: &mut App) {
    let component_theme = match theme {
        ThemeMode::Dark => ComponentThemeMode::Dark,
        ThemeMode::Light => ComponentThemeMode::Light,
        ThemeMode::System if colors::is_dark() => ComponentThemeMode::Dark,
        ThemeMode::System => ComponentThemeMode::Light,
    };
    Theme::change(component_theme, Some(window), cx);
}

fn scale_level(level: f32) -> f32 {
    if level <= 0.005 {
        0.0
    } else {
        level.clamp(0.0, 1.0).powf(0.35)
    }
}

fn smooth_level(current: f32, target: f32) -> f32 {
    if target >= current {
        target
    } else {
        let decayed = current * 0.65;
        if decayed < 0.05 { 0.0 } else { decayed }
    }
}

fn update_peak_hold(peak_until: &mut Option<Instant>, level: f32, now: Instant) {
    if level >= 0.92 {
        *peak_until = Some(now + std::time::Duration::from_secs(2));
    } else if peak_until.is_some_and(|until| until <= now) {
        *peak_until = None;
    }
}
