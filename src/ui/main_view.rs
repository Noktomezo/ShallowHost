use gpui::prelude::*;
use gpui::*;
use std::rc::Rc;
use std::sync::Arc;
use std::time::{Duration, Instant};

use crate::config::ConfigStore;
use crate::engine::Engine;
use crate::single_instance::SingleInstance;
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
mod sidebar_motion;
mod state_actions;

use sidebar_motion::SidebarMotion;

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
    single_instance: SingleInstance,
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

    sidebar_motion: SidebarMotion,
    _subscriptions: Vec<Subscription>,
    _meter_task: Task<()>,
    _system_task: Task<()>,
    _chain_restore_task: Task<()>,
    _audio_routing_task: Task<()>,
    audio_routing_revision: u64,
    update_check_task: Option<Task<()>>,
}

impl MainView {
    pub fn new(
        engine: Arc<Engine>,
        storage: ConfigStore,
        single_instance: SingleInstance,
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
            single_instance,
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

            sidebar_motion: SidebarMotion::expanded(),
            _subscriptions: Vec::new(),
            _meter_task: Task::ready(()),
            _system_task: Task::ready(()),
            _chain_restore_task: Task::ready(()),
            _audio_routing_task: Task::ready(()),
            audio_routing_revision: 0,
            update_check_task: None,
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
            .push(cx.observe(&updater, |_this, updater, cx| {
                if matches!(
                    updater.read(cx).status(),
                    gpui_updater::UpdateStatus::Staged(_)
                ) && crate::updater::take_restart_after_update()
                {
                    let updater = updater.clone();
                    cx.spawn(async move |_, cx| {
                        cx.background_executor().timer(Duration::from_secs(1)).await;
                        cx.update(|cx| crate::updater::restart(&updater, cx));
                    })
                    .detach();
                }
                cx.notify();
            }));
        if initial_config.system.auto_check_updates {
            crate::updater::start_check(&updater, cx);
            this.start_update_check_task(cx);
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

    fn start_update_check_task(&mut self, cx: &mut Context<Self>) {
        let updater = self.updater.clone();
        self.update_check_task = Some(cx.spawn(async move |_, cx| {
            loop {
                cx.background_executor()
                    .timer(crate::updater::UPDATE_CHECK_INTERVAL)
                    .await;
                cx.update(|cx| crate::updater::start_check(&updater, cx));
            }
        }));
    }

    fn stop_update_check_task(&mut self) {
        self.update_check_task = None;
    }

    fn render_nav_item(
        &self,
        item: &NavigationItem,
        collapsed: bool,
        sidebar_progress: f32,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let route = item.route;
        let is_selected = self.current_route == route;
        let is_hovered = self.hovered_route == Some(route) && !is_selected;
        let is_unhovered = self.unhovered_route == Some(route) && !is_selected;
        let item_label = SharedString::from(item.label());
        let tooltip_source = ElementId::Name(format!("{}-collapsed-tooltip", item.id).into());
        let hover_tooltip_source = tooltip_source.clone();
        let pressed_tooltip_source = tooltip_source.clone();
        let tooltip_label = item_label.clone();

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

        if (selected_alpha > 0.0 && selected_alpha < 1.0)
            || (hover_alpha > 0.0 && hover_alpha < 1.0)
        {
            cx.on_next_frame(window, |_, _, cx| cx.notify());
        }

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
            .when(selected_alpha > 0.001, |this| {
                this.bg(colors::orange().opacity(selected_alpha))
            })
            .when(selected_alpha <= 0.001 && hover_alpha > 0.001, |this| {
                this.bg(colors::orange().opacity(hover_alpha * 0.16))
            })
            .on_hover(cx.listener(move |this, is_hovered, window, cx| {
                this.set_hovered_route(route, *is_hovered, cx);
                if collapsed {
                    crate::ui::cursor_tooltip::set_hovered(
                        hover_tooltip_source.clone(),
                        tooltip_label.clone(),
                        *is_hovered,
                        window,
                        cx,
                    );
                } else {
                    crate::ui::cursor_tooltip::hide_source(&hover_tooltip_source, window, cx);
                }
            }))
            .on_mouse_down(MouseButton::Left, move |_, window, cx| {
                crate::ui::cursor_tooltip::hide_source(&pressed_tooltip_source, window, cx);
            })
            .on_click(cx.listener(move |this, _, _, cx| {
                this.navigate(route, cx);
            }))
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
            .child(
                div()
                    .relative()
                    .flex_1()
                    .text_sm()
                    .text_color(icon_color.opacity(sidebar_progress))
                    .ml(px(-14.0 * (1.0 - sidebar_progress)))
                    .truncate()
                    .overflow_hidden()
                    .child(item_label),
            )
            .into_any_element()
    }
}

impl Render for MainView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let collapsed = self.sidebar_collapsed;
        let (sidebar_progress, sidebar_animating) = self.sidebar_motion.sample();
        if sidebar_animating {
            cx.on_next_frame(window, |_, _, cx| cx.notify());
        }
        let is_maximized = window.is_maximized();
        let sidebar_width = 40.0 + 104.0 * sidebar_progress;

        let root_bg = if self.transparent_shell {
            colors::base_950().opacity(0.5)
        } else {
            colors::base_950()
        };

        let main_items: Vec<_> = MAIN_NAV_ITEMS
            .iter()
            .map(|item| self.render_nav_item(item, collapsed, sidebar_progress, window, cx))
            .collect();

        let footer_item =
            self.render_nav_item(&FOOTER_NAV_ITEM, collapsed, sidebar_progress, window, cx);

        let sidebar_toggle_listener = cx.listener(|this: &mut Self, _: &(), window, cx| {
            this.toggle_sidebar(window, cx);
        });
        let render_ctx = self.render_context();
        let callbacks = Self::dropdown_callbacks(cx);

        let page_content =
            self.current_route
                .render(render_ctx, &callbacks, Arc::clone(&self.engine), window, cx);
        let close_listener = cx.listener(|this: &mut Self, _: &(), window, cx| {
            this.close_or_hide(window, cx);
        });
        let update_listener = cx.listener(|this: &mut Self, _: &(), _window, cx| {
            crate::updater::download_and_install(&this.updater, cx);
        });
        let update_status =
            crate::updater::mock_status().unwrap_or_else(|| self.updater.read(cx).status().clone());
        let titlebar = render_titlebar(
            is_maximized,
            sidebar_progress,
            &update_status,
            Rc::new(move |window, cx| sidebar_toggle_listener(&(), window, cx)),
            Rc::new(move |window, cx| update_listener(&(), window, cx)),
            Rc::new(move |window, cx| close_listener(&(), window, cx)),
            cx,
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
                            .w(px(sidebar_width))
                            .overflow_hidden()
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
