use gpui::prelude::*;
use gpui::*;
use std::rc::Rc;
use std::sync::Arc;
use std::time::{Duration, Instant};

use crate::config::ConfigStore;
use crate::engine::Engine;
use crate::system_integration::SystemIntegration;

use super::audio_controls::AudioControls;
use super::audio_dropdown::AudioDropdownEvent;
use super::chain_operations::ChainOperationState;
use super::colors;
use super::i18n;
use super::motion::{DropdownMotion, mix_color};
use super::navigation::{FOOTER_NAV_ITEM, MAIN_NAV_ITEMS, NavigationItem};
use super::pages::home::update_chain_drag_mouse;
use super::pages::plugins::PluginScanState;
use super::routes::{Language, Route, ThemeMode};
use super::titlebar::render_titlebar;
use gpui_updater::Updater;

mod render_support;
mod state_actions;

pub struct MainView {
    engine: Arc<Engine>,
    storage: ConfigStore,
    sidebar_collapsed: bool,
    current_route: Route,
    deselected_route: Option<Route>,
    selected_at: Instant,
    deselected_at: Option<Instant>,

    selected_theme: ThemeMode,
    selected_language: Language,
    transparent_shell: bool,
    transparency_changed_at: Option<Instant>,
    scan_paths_open: bool,
    audio_controls: AudioControls,
    is_mono: bool,
    mono_changed_at: Option<Instant>,
    theme_dropdown_motion: Entity<DropdownMotion>,
    language_dropdown_motion: Entity<DropdownMotion>,
    plugin_scan_state: Entity<PluginScanState>,
    chain_operation_state: Entity<ChainOperationState>,
    updater: Entity<Updater>,
    system_integration: Option<SystemIntegration>,
    system_changed_at: [Option<Instant>; 4],
    input_level: f32,
    output_level: f32,
    input_peak_until: Option<Instant>,
    output_peak_until: Option<Instant>,
    hovered_route: Option<Route>,
    unhovered_route: Option<Route>,
    hovered_at: Option<Instant>,
    unhovered_at: Option<Instant>,

    anim_key: usize,
    _subscriptions: Vec<Subscription>,
    _meter_task: Task<()>,
    _system_task: Task<()>,
    _chain_restore_task: Task<()>,
}

impl MainView {
    pub fn new(
        engine: Arc<Engine>,
        storage: ConfigStore,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let initial_config = storage.config().clone();
        let default_theme = initial_config.theme;
        let default_lang = initial_config.language;

        colors::set_active_theme(default_theme);
        state_actions::sync_component_theme(default_theme, window, cx);
        i18n::set_language(default_lang);
        window.set_background_appearance(if initial_config.transparent_shell {
            WindowBackgroundAppearance::Blurred
        } else {
            WindowBackgroundAppearance::Opaque
        });

        let devices = engine
            .audio_devices(&initial_config.audio.driver, "")
            .unwrap_or_else(|error| {
                eprintln!("failed to enumerate audio devices: {error}");
                Default::default()
            });
        let audio_controls = AudioControls::new(&devices, &initial_config.audio, cx);
        let updater = crate::updater::new_entity(cx);
        let system_integration =
            match SystemIntegration::new(&i18n::t("tray.show"), &i18n::t("tray.quit")) {
                Ok(integration) => {
                    if let Err(error) = integration.sync_autostart(initial_config.system.autostart)
                    {
                        eprintln!("failed to synchronize autostart: {error}");
                    }
                    Some(integration)
                }
                Err(error) => {
                    eprintln!("failed to initialize system integration: {error}");
                    None
                }
            };

        let mut this = Self {
            engine,
            storage,
            sidebar_collapsed: false,
            current_route: Route::Home,
            deselected_route: None,
            selected_at: Instant::now() - Duration::from_secs(10),
            deselected_at: None,

            selected_theme: default_theme,
            selected_language: default_lang,
            transparent_shell: initial_config.transparent_shell,
            transparency_changed_at: None,
            scan_paths_open: false,
            audio_controls: audio_controls.clone(),
            is_mono: initial_config.audio.is_mono,
            mono_changed_at: None,
            theme_dropdown_motion: cx.new(|_| DropdownMotion::default()),
            language_dropdown_motion: cx.new(|_| DropdownMotion::default()),
            plugin_scan_state: cx.new(|_| PluginScanState::default()),
            chain_operation_state: cx.new(|_| ChainOperationState::default()),
            updater: updater.clone(),
            system_integration,
            system_changed_at: [None; 4],
            input_level: 0.0,
            output_level: 0.0,
            input_peak_until: None,
            output_peak_until: None,
            hovered_route: None,
            unhovered_route: None,
            hovered_at: None,
            unhovered_at: None,

            anim_key: 0,
            _subscriptions: Vec::new(),
            _meter_task: Task::ready(()),
            _system_task: Task::ready(()),
            _chain_restore_task: Task::ready(()),
        };

        this._subscriptions.push(cx.subscribe(
            &audio_controls.driver,
            |this, _, _: &AudioDropdownEvent, cx| {
                this.audio_controls.refresh_devices(&this.engine, cx);
                if this.audio_controls.is_asio(cx) {
                    this.audio_controls.refresh_asio_channels(&this.engine, cx);
                }
                this.audio_controls.remember_device_selection(cx);
                this.apply_and_persist_audio(cx);
            },
        ));

        this._subscriptions.push(cx.subscribe(
            &audio_controls.output,
            |this, _, _: &AudioDropdownEvent, cx| {
                if this.audio_controls.is_asio(cx) {
                    this.audio_controls.refresh_asio_channels(&this.engine, cx);
                }
                this.audio_controls.remember_device_selection(cx);
                this.apply_and_persist_audio(cx);
            },
        ));

        this._subscriptions.push(cx.subscribe(
            &audio_controls.sample_rate,
            |this, _, _: &AudioDropdownEvent, cx| {
                this.audio_controls.refresh_buffer_latency(cx);
                this.apply_and_persist_audio(cx);
            },
        ));

        this._subscriptions.push(cx.subscribe(
            &audio_controls.input,
            |this, _, _: &AudioDropdownEvent, cx| {
                this.audio_controls.remember_device_selection(cx);
                this.apply_and_persist_audio(cx);
            },
        ));

        this._subscriptions.push(cx.subscribe(
            &audio_controls.buffer_size,
            |this, _, _: &AudioDropdownEvent, cx| {
                this.apply_and_persist_audio(cx);
            },
        ));

        this._subscriptions
            .push(cx.observe(&updater, |_, _, cx| cx.notify()));
        if initial_config.system.auto_check_updates {
            crate::updater::start_check(&updater, cx);
        }

        if this.audio_controls.is_asio(cx) {
            this.audio_controls.refresh_asio_channels(&this.engine, cx);
        }
        this.audio_controls.remember_device_selection(cx);
        this.audio_controls.apply(&this.engine, cx, this.is_mono);
        this.start_chain_restore_task(cx);
        this.start_meter_task(cx);
        this.start_system_task(cx);
        this.install_close_handler(window, cx);
        if crate::system_integration::is_autostart_launch()
            && initial_config.system.autostart_to_tray
            && let Err(error) = crate::system_integration::hide_window(window)
        {
            eprintln!("failed to hide autostart window: {error}");
        }
        this
    }

    fn render_nav_item(
        &self,
        item: &NavigationItem,
        collapsed: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let route = item.route;
        let is_selected = self.current_route == route;
        let is_hovered = self.hovered_route == Some(route) && !is_selected;
        let is_unhovered = self.unhovered_route == Some(route) && !is_selected;
        let item_label = item.label();

        // Smooth Bidirectional Selection Factor (150ms fade-in AND fade-out)
        let selected_alpha = if is_selected {
            let elapsed = self.selected_at.elapsed().as_secs_f32();
            (elapsed / 0.15).clamp(0.0, 1.0)
        } else if self.deselected_route == Some(route) {
            if let Some(at) = self.deselected_at {
                (1.0 - at.elapsed().as_secs_f32() / 0.15).clamp(0.0, 1.0)
            } else {
                0.0
            }
        } else {
            0.0
        };

        // Smooth Bidirectional Hover Factor (150ms fade-in AND fade-out)
        let hover_alpha = if is_hovered {
            if let Some(at) = self.hovered_at {
                (at.elapsed().as_secs_f32() / 0.15).clamp(0.0, 1.0)
            } else {
                1.0
            }
        } else if is_unhovered {
            if let Some(at) = self.unhovered_at {
                (1.0 - at.elapsed().as_secs_f32() / 0.15).clamp(0.0, 1.0)
            } else {
                0.0
            }
        } else {
            0.0
        };

        // Synchronized Foreground RGB Interpolation (Icon and Text in 100% Lockstep)
        let active_t = if selected_alpha > 0.001 {
            selected_alpha
        } else {
            hover_alpha
        };

        let target_foreground = if selected_alpha > 0.001 {
            colors::accent_foreground()
        } else {
            colors::orange()
        };
        let icon_color = mix_color(colors::base_200(), target_foreground, active_t);

        // Request frame while selection or hover transitions are active
        if (selected_alpha > 0.0 && selected_alpha < 1.0)
            || (hover_alpha > 0.0 && hover_alpha < 1.0)
        {
            cx.on_next_frame(window, |_, _, cx| cx.notify());
        }

        let anim_key = self.anim_key;
        let text_anim_id =
            ElementId::NamedInteger(format!("nav-text-{}", item.id).into(), anim_key as u64);

        div()
            .id(item.id)
            .relative()
            .h(px(32.0))
            .w_full()
            .flex()
            .flex_row()
            .items_center()
            .px(px(8.0))
            .gap(px(8.0))
            .rounded_md()
            .cursor_pointer()
            // Smoothly Interpolated Background & Text Colors
            .when(selected_alpha > 0.001, |this| {
                this.bg(colors::orange().opacity(selected_alpha))
            })
            .when(selected_alpha <= 0.001 && hover_alpha > 0.001, |this| {
                this.bg(colors::orange().opacity(hover_alpha * 0.16))
            })
            .on_hover(cx.listener(move |this, is_hovered, _, cx| {
                this.set_hovered_route(route, *is_hovered, cx);
            }))
            .on_click(cx.listener(move |this, _, _, cx| {
                this.navigate(route, cx);
            }))
            // Icon Container
            .child(
                div()
                    .relative()
                    .flex_shrink_0()
                    .flex()
                    .items_center()
                    .justify_center()
                    .child(
                        svg()
                            .external_path(crate::ui::resolve_asset_path(item.icon_path))
                            .size_4()
                            .text_color(icon_color),
                    ),
            )
            // Sliding & fading text label (200ms animation on sidebar toggle)
            .child({
                let text_color_static = if collapsed {
                    icon_color.opacity(0.0)
                } else {
                    icon_color
                };

                div()
                    .relative()
                    .flex_1()
                    .text_sm()
                    .text_color(text_color_static)
                    .truncate()
                    .overflow_hidden()
                    .with_animation(
                        text_anim_id,
                        Animation::new(Duration::from_millis(200)),
                        move |s, delta| {
                            let t = delta.clamp(0.0, 1.0);
                            let eased = if t < 0.5 {
                                4.0 * t * t * t
                            } else {
                                1.0 - (-2.0 * t + 2.0).powi(3) / 2.0
                            };

                            let (alpha_factor, offset) = if collapsed {
                                (1.0 - eased, -14.0 * eased)
                            } else {
                                (eased, -14.0 * (1.0 - eased))
                            };

                            s.text_color(icon_color.opacity(alpha_factor))
                                .ml(px(offset))
                        },
                    )
                    .child(item_label)
            })
            .into_any_element()
    }
}

impl Render for MainView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let collapsed = self.sidebar_collapsed;
        let anim_id = ElementId::NamedInteger("sidebar-anim".into(), self.anim_key as u64);
        let is_maximized = window.is_maximized();

        let start_width = if collapsed { 144.0 } else { 40.0 };
        let target_width = if collapsed { 40.0 } else { 144.0 };

        let root_bg = if self.transparent_shell {
            colors::base_950().opacity(0.5)
        } else {
            colors::base_950()
        };

        let main_items: Vec<_> = MAIN_NAV_ITEMS
            .iter()
            .map(|item| self.render_nav_item(item, collapsed, window, cx))
            .collect();

        let footer_item = self.render_nav_item(&FOOTER_NAV_ITEM, collapsed, window, cx);

        let sidebar_toggle_listener = cx.listener(|this: &mut Self, _: &(), _window, cx| {
            this.toggle_sidebar(cx);
        });
        let render_ctx = self.render_context();
        let callbacks = Self::dropdown_callbacks(cx);

        let page_content =
            self.current_route
                .render(render_ctx, &callbacks, Arc::clone(&self.engine), window, cx);
        let close_listener = cx.listener(|this: &mut Self, _: &(), window, cx| {
            this.close_or_hide(window, cx);
        });
        let titlebar = render_titlebar(
            is_maximized,
            Rc::new(move |window, cx| sidebar_toggle_listener(&(), window, cx)),
            Rc::new(move |window, cx| close_listener(&(), window, cx)),
        );

        div()
            .size_full()
            .on_mouse_move(cx.listener(|_, event: &MouseMoveEvent, window, cx| {
                let tooltip_moved = crate::ui::cursor_tooltip::update_position(event.position, cx);
                if update_chain_drag_mouse(event.position, cx) || tooltip_moved {
                    cx.notify();
                    window.refresh();
                }
            }))
            .bg(root_bg)
            .flex()
            .flex_col()
            .child(titlebar)
            .child(
                div()
                    .flex_1()
                    .w_full()
                    .flex()
                    .flex_row()
                    .overflow_hidden()
                    .child(
                        div()
                            .id("sidebar-container")
                            .flex()
                            .flex_col()
                            .justify_between()
                            .h_full()
                            .overflow_hidden()
                            .with_animation(
                                anim_id,
                                Animation::new(Duration::from_millis(200)),
                                move |this, delta| {
                                    let t = delta.clamp(0.0, 1.0);
                                    let eased = if t < 0.5 {
                                        4.0 * t * t * t
                                    } else {
                                        1.0 - (-2.0 * t + 2.0).powi(3) / 2.0
                                    };
                                    let width = start_width + (target_width - start_width) * eased;
                                    this.w(px(width))
                                },
                            )
                            .child(
                                div()
                                    .flex_1()
                                    .flex()
                                    .flex_col()
                                    .gap(px(4.0))
                                    .px(px(4.0))
                                    .py(px(4.0))
                                    .children(main_items),
                            )
                            .child(
                                div()
                                    .flex_shrink_0()
                                    .flex()
                                    .flex_col()
                                    .px(px(4.0))
                                    .pb(px(4.0))
                                    .child(footer_item),
                            ),
                    )
                    .child(
                        div()
                            .flex_1()
                            .h_full()
                            .bg(colors::black())
                            .border_t_1()
                            .border_l_1()
                            .border_color(colors::base_800())
                            .rounded_tl(px(8.0))
                            .overflow_hidden()
                            .child(page_content),
                    ),
            )
            .child(crate::ui::cursor_tooltip::overlay(cx))
    }
}
