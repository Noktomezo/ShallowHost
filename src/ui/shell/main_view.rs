use gpui::prelude::*;
use gpui::*;
use std::rc::Rc;
use std::sync::Arc;
use std::time::{Duration, Instant};

use crate::infrastructure::config::ConfigStore;
use crate::infrastructure::engine::Engine;
use crate::infrastructure::single_instance::SingleInstance;
use crate::infrastructure::system::SystemIntegration;

use super::navigation::{FOOTER_NAV_ITEM, MAIN_NAV_ITEMS};
use super::routes::{Language, Route, ThemeMode};
use super::titlebar::render_titlebar;
use crate::ui::components::audio_dropdown::AudioDropdownEvent;
use crate::ui::components::text_input::{TextInputEvent, TextInputState};
use crate::ui::foundation::motion::DropdownMotion;
use crate::ui::foundation::{colors, i18n};
use crate::ui::pages::home::update_chain_drag_mouse;
use crate::ui::pages::plugins::{PluginLibraryState, PluginScanState, SEARCH_FOCUS_KEY};
use crate::ui::state::audio_controls::AudioControls;
use crate::ui::state::chain_operations::ChainOperationState;
use gpui_updater::Updater;

mod chain_state_task;
mod navigation_item;
mod page_transition;
mod render_support;
mod scan_paths_motion;
mod sidebar_motion;
mod state_actions;

use page_transition::PageTransition;
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
    scan_paths_closing: bool,
    scan_paths_revision: u64,
    audio_controls: AudioControls,
    is_mono: bool,
    mono_changed_at: Option<Instant>,
    theme_dropdown_motion: Entity<DropdownMotion>,
    language_dropdown_motion: Entity<DropdownMotion>,
    plugin_scan_state: Entity<PluginScanState>,
    plugin_search: Entity<TextInputState>,
    plugin_library_state: Entity<PluginLibraryState>,
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
    page_transition: PageTransition,
    _subscriptions: Vec<Subscription>,
    _meter_task: Task<()>,
    _system_task: Task<()>,
    _chain_restore_task: Task<()>,
    _chain_state_task: Task<()>,
    _audio_routing_task: Task<()>,
    _scan_paths_motion_task: Task<()>,
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
        let plugin_search = cx.new(|cx| {
            TextInputState::new(window, cx)
                .placeholder(i18n::t("plugins.search"))
                .clean_on_escape()
        });
        let updater = crate::infrastructure::updater::new_entity(cx);
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
            scan_paths_closing: false,
            scan_paths_revision: 0,
            audio_controls: audio_controls.clone(),
            is_mono: initial_config.audio.is_mono,
            mono_changed_at: None,
            theme_dropdown_motion: cx.new(|_| DropdownMotion::default()),
            language_dropdown_motion: cx.new(|_| DropdownMotion::default()),
            plugin_scan_state: cx.new(|_| PluginScanState::default()),
            plugin_search: plugin_search.clone(),
            plugin_library_state: cx
                .new(|_| PluginLibraryState::new(initial_config.plugins.group_by_author)),
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
            page_transition: PageTransition::new(),
            _subscriptions: Vec::new(),
            _meter_task: Task::ready(()),
            _system_task: Task::ready(()),
            _chain_restore_task: Task::ready(()),
            _chain_state_task: Task::ready(()),
            _audio_routing_task: Task::ready(()),
            _scan_paths_motion_task: Task::ready(()),
            audio_routing_revision: 0,
            update_check_task: None,
        };

        this._subscriptions.push(cx.subscribe_in(
            &plugin_search,
            window,
            |_, _, event: &TextInputEvent, window, cx| match event {
                TextInputEvent::Change => cx.notify(),
                TextInputEvent::Focus => crate::ui::foundation::hover_motion::set_active(
                    SharedString::from(SEARCH_FOCUS_KEY),
                    true,
                    window,
                    cx,
                ),
                TextInputEvent::Blur => crate::ui::foundation::hover_motion::set_active(
                    SharedString::from(SEARCH_FOCUS_KEY),
                    false,
                    window,
                    cx,
                ),
                TextInputEvent::PressEnter => {}
            },
        ));

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
                ) && crate::infrastructure::updater::take_restart_after_update()
                {
                    let updater = updater.clone();
                    cx.spawn(async move |_, cx| {
                        cx.background_executor().timer(Duration::from_secs(1)).await;
                        cx.update(|cx| crate::infrastructure::updater::restart(&updater, cx));
                    })
                    .detach();
                }
                cx.notify();
            }));
        if initial_config.system.auto_check_updates {
            crate::infrastructure::updater::start_check(&updater, cx);
            this.start_update_check_task(cx);
        }

        if this.audio_controls.is_asio(cx) {
            this.audio_controls.refresh_asio_channels(&this.engine, cx);
        }
        this.audio_controls.remember_device_selection(cx);
        this.audio_controls.apply(&this.engine, cx, this.is_mono);
        this.start_chain_restore_task(cx);
        this.start_chain_state_task(cx);
        this.start_meter_task(cx);
        this.start_system_task(cx);
        this.install_close_handler(window, cx);
        if crate::infrastructure::system::is_autostart_launch()
            && initial_config.system.autostart_to_tray
            && let Err(error) = crate::infrastructure::system::hide_window(window)
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
                    .timer(crate::infrastructure::updater::UPDATE_CHECK_INTERVAL)
                    .await;
                cx.update(|cx| crate::infrastructure::updater::start_check(&updater, cx));
            }
        }));
    }

    fn stop_update_check_task(&mut self) {
        self.update_check_task = None;
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
        let page_content = self.page_transition.wrap(page_content);
        let close_listener = cx.listener(|this: &mut Self, _: &(), window, cx| {
            this.close_or_hide(window, cx);
        });
        let update_listener = cx.listener(|this: &mut Self, _: &(), _window, cx| {
            crate::infrastructure::updater::download_and_install(&this.updater, cx);
        });
        let update_status = crate::infrastructure::updater::mock_status()
            .unwrap_or_else(|| self.updater.read(cx).status().clone());
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
            .font_family(crate::ui::foundation::control_style::CONTROL_FONT_FAMILY)
            .on_mouse_move(cx.listener(|_, event: &MouseMoveEvent, window, cx| {
                let tooltip_moved =
                    crate::ui::components::cursor_tooltip::update_position(event.position, cx);
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
                            .border_t(px(1.0))
                            .border_l(px(1.0))
                            .border_color(colors::base_800())
                            .rounded_tl(px(8.0))
                            .overflow_hidden()
                            .child(page_content),
                    ),
            )
            .child(crate::ui::components::cursor_tooltip::overlay(cx))
    }
}
