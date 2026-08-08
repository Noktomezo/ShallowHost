use gpui::prelude::*;
use gpui::*;
use gpui_component::StyledExt;
use std::time::Instant;

use crate::domain::preferences::{Language, ThemeMode};
use crate::infrastructure::config::SystemSettings;
use crate::ui::components::card_header::card_heading;
use crate::ui::components::smooth_scroll::SmoothVerticalScroll;
use crate::ui::components::toggle_switch::toggle_switch;
use crate::ui::foundation::colors;
use crate::ui::foundation::i18n;
use crate::ui::foundation::motion::DropdownMotion;
use crate::ui::shell::routes::{
    DropdownCallbacks, LanguageCallback, RenderContext, SystemCallback, SystemSetting,
    ThemeCallback, TransparencyCallback,
};
use gpui_updater::Updater;

mod appearance;
mod appearance_trigger;
mod updates;

impl ThemeMode {
    pub fn label(self) -> String {
        i18n::t(match self {
            Self::System => "settings.themeSystem",
            Self::Light => "settings.themeLight",
            Self::Dark => "settings.themeDark",
        })
    }

    pub const fn icon_path(self) -> &'static str {
        match self {
            Self::System => "assets/icons/monitor.svg",
            Self::Light => "assets/icons/sun.svg",
            Self::Dark => "assets/icons/moon.svg",
        }
    }
}

impl Language {
    pub fn label(self) -> String {
        i18n::t(match self {
            Self::System => "settings.langSystem",
            Self::Russian => "settings.langRu",
            Self::English => "settings.langEn",
        })
    }

    pub const fn icon_path(self) -> &'static str {
        match self {
            Self::System => "assets/icons/globe.svg",
            Self::Russian => "assets/icons/flags/ru.png",
            Self::English => "assets/icons/flags/gb.png",
        }
    }

    pub const fn uses_flag(self) -> bool {
        !matches!(self, Self::System)
    }
}

pub struct SettingsPage {
    selected_theme: ThemeMode,
    selected_language: Language,
    on_change_theme: ThemeCallback,
    on_change_language: LanguageCallback,
    transparent_shell: bool,
    transparency_changed_at: Option<Instant>,
    on_change_transparency: TransparencyCallback,
    system: SystemSettings,
    system_changed_at: [Option<Instant>; 4],
    on_change_system: SystemCallback,
    theme_dropdown_motion: Entity<DropdownMotion>,
    language_dropdown_motion: Entity<DropdownMotion>,
    updater: Entity<Updater>,
}

impl SettingsPage {
    pub fn new(context: RenderContext, callbacks: &DropdownCallbacks) -> Self {
        Self {
            selected_theme: context.selected_theme,
            selected_language: context.selected_language,
            on_change_theme: callbacks.on_change_theme.clone(),
            on_change_language: callbacks.on_change_language.clone(),
            transparent_shell: context.transparent_shell,
            transparency_changed_at: context.transparency_changed_at,
            on_change_transparency: callbacks.on_change_transparency.clone(),
            system: context.system_settings,
            system_changed_at: context.system_changed_at,
            on_change_system: callbacks.on_change_system.clone(),
            theme_dropdown_motion: context.theme_dropdown_motion,
            language_dropdown_motion: context.language_dropdown_motion,
            updater: context.updater,
        }
    }

    pub fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        SmoothVerticalScroll::new(
            "settings-page-scroll",
            div()
                .w_full()
                .p_4()
                .flex()
                .flex_col()
                .gap_3()
                .child(page_header())
                .child(appearance::appearance_card(
                    appearance::AppearanceCardProps {
                        selected_theme: self.selected_theme,
                        selected_language: self.selected_language,
                        transparent_shell: self.transparent_shell,
                        transparency_changed_at: self.transparency_changed_at,
                        on_change_theme: self.on_change_theme,
                        on_change_language: self.on_change_language,
                        on_change_transparency: self.on_change_transparency,
                        theme_motion: self.theme_dropdown_motion,
                        language_motion: self.language_dropdown_motion,
                    },
                    cx,
                ))
                .child(system_card(
                    &self.system,
                    &self.system_changed_at,
                    self.on_change_system.clone(),
                ))
                .child(updates::updates_card(
                    &self.system,
                    self.system_changed_at[SystemSetting::AutoCheckUpdates.motion_index()],
                    self.on_change_system,
                    self.updater,
                    cx,
                )),
        )
    }
}

fn page_header() -> AnyElement {
    div()
        .flex()
        .flex_col()
        .gap(px(2.0))
        .child(
            div()
                .text_xl()
                .font_semibold()
                .text_color(colors::base_200())
                .child(i18n::t("settings.title")),
        )
        .child(
            div()
                .text_sm()
                .text_color(colors::base_500())
                .child(i18n::t("settings.description")),
        )
        .into_any_element()
}

fn system_card(
    settings: &SystemSettings,
    changed_at: &[Option<Instant>; 4],
    callback: SystemCallback,
) -> AnyElement {
    card()
        .child(div().p_4().child(card_heading(
            "assets/icons/cog.svg",
            colors::cyan(),
            "settings.system",
            "settings.systemDescription",
        )))
        .child(separator())
        .child(
            div()
                .p_4()
                .flex()
                .flex_col()
                .gap_4()
                .child(toggle_row(ToggleRowProps {
                    id: "system-autostart",
                    title: "settings.autostart",
                    description: "settings.autostartDescription",
                    checked: settings.autostart,
                    enabled: true,
                    changed_at: changed_at[SystemSetting::Autostart.motion_index()],
                    setting: SystemSetting::Autostart,
                    callback: callback.clone(),
                }))
                .child(toggle_row(ToggleRowProps {
                    id: "system-autostart-tray",
                    title: "settings.autostartToTray",
                    description: "settings.autostartToTrayDescription",
                    checked: settings.autostart_to_tray,
                    enabled: settings.autostart,
                    changed_at: changed_at[SystemSetting::AutostartToTray.motion_index()],
                    setting: SystemSetting::AutostartToTray,
                    callback: callback.clone(),
                }))
                .child(toggle_row(ToggleRowProps {
                    id: "system-minimize-tray",
                    title: "settings.minimizeToTray",
                    description: "settings.minimizeToTrayDescription",
                    checked: settings.minimize_to_tray,
                    enabled: true,
                    changed_at: changed_at[SystemSetting::MinimizeToTray.motion_index()],
                    setting: SystemSetting::MinimizeToTray,
                    callback,
                })),
        )
        .into_any_element()
}

struct ToggleRowProps {
    id: &'static str,
    title: &'static str,
    description: &'static str,
    checked: bool,
    enabled: bool,
    changed_at: Option<Instant>,
    setting: SystemSetting,
    callback: SystemCallback,
}

fn toggle_row(props: ToggleRowProps) -> AnyElement {
    let ToggleRowProps {
        id,
        title,
        description,
        checked,
        enabled,
        changed_at,
        setting,
        callback,
    } = props;
    row()
        .id(id)
        .px_0()
        .py_0()
        .when(enabled, |element| {
            element.cursor_pointer().on_click(move |_, window, cx| {
                callback(setting, !checked, window, cx);
            })
        })
        .when(!enabled, |element| element.cursor_default().opacity(0.5))
        .child(setting_copy(title, description))
        .child(toggle_switch(id, checked, enabled, changed_at))
        .into_any_element()
}

fn card() -> Div {
    div()
        .w_full()
        .bg(colors::base_950())
        .border_1()
        .border_color(colors::base_800())
        .rounded_lg()
        .flex()
        .flex_col()
}

fn row() -> Div {
    div()
        .w_full()
        .p_4()
        .flex()
        .items_center()
        .justify_between()
        .gap_4()
}

fn setting_copy(title: &'static str, description: &'static str) -> AnyElement {
    div()
        .flex_1()
        .flex()
        .flex_col()
        .gap(px(1.0))
        .child(
            div()
                .text_sm()
                .font_medium()
                .text_color(colors::base_200())
                .child(i18n::t(title)),
        )
        .child(
            div()
                .text_xs()
                .text_color(colors::base_500())
                .child(i18n::t(description)),
        )
        .into_any_element()
}

fn separator() -> Div {
    div().h(px(1.0)).w_full().bg(colors::base_800())
}

fn resolve_path(relative: &'static str) -> String {
    crate::ui::resolve_asset_path(relative)
}
