use std::time::{Duration, Instant};

use gpui::prelude::*;
use gpui::*;
use gpui_updater::{UpdateStatus, Updater};

use super::{ToggleRowProps, card, resolve_path, row, separator, toggle_row};
use crate::config::SystemSettings;
use crate::ui::badge::{BadgeStyle, badge, loading_badge, progress_badge};
use crate::ui::card_header::card_heading_with_suffix;
use crate::ui::colors;
use crate::ui::control_style::ControlTypography;
use crate::ui::i18n;
use crate::ui::routes::{SystemCallback, SystemSetting};

pub(super) fn updates_card(
    settings: &SystemSettings,
    changed_at: Option<Instant>,
    callback: SystemCallback,
    updater: Entity<Updater>,
    cx: &mut App,
) -> AnyElement {
    let mocked_status = crate::updater::mock_status();
    let status = mocked_status.unwrap_or_else(|| updater.read(cx).status().clone());
    let error_line = error_line(&status);
    let header_badges = update_badges(&status);
    let primary_action = primary_action(&status, updater.clone());
    let check_action = check_action(&status, updater);

    card()
        .child(
            row()
                .child(card_heading_with_suffix(
                    "assets/icons/cloud-download.svg",
                    colors::green(),
                    "settings.updates",
                    "settings.updatesDescription",
                    Some(header_badges),
                ))
                .child(
                    div()
                        .flex_none()
                        .flex()
                        .items_center()
                        .gap_2()
                        .children(primary_action)
                        .child(check_action),
                ),
        )
        .child(separator())
        .child(
            div()
                .p_4()
                .flex()
                .flex_col()
                .gap_3()
                .when_some(error_line, |element, text| {
                    element.child(div().text_xs().text_color(colors::red()).child(text))
                })
                .child(toggle_row(ToggleRowProps {
                    id: "system-auto-updates",
                    title: "settings.autoCheck",
                    description: "settings.autoCheckDescription",
                    checked: settings.auto_check_updates,
                    enabled: true,
                    changed_at,
                    setting: SystemSetting::AutoCheckUpdates,
                    callback,
                })),
        )
        .into_any_element()
}

fn primary_action(status: &UpdateStatus, updater: Entity<Updater>) -> Option<AnyElement> {
    let (label, icon_path) = match status {
        UpdateStatus::Available(_) => ("update.install", "assets/icons/cloud-download.svg"),
        _ => return None,
    };

    Some(
        div()
            .id("update-primary-action")
            .h(px(34.0))
            .px_3()
            .flex()
            .items_center()
            .gap_2()
            .rounded_md()
            .flex_none()
            .cursor_pointer()
            .bg(colors::orange())
            .border_1()
            .border_color(colors::orange())
            .control_text()
            .text_color(colors::accent_foreground())
            .hover(|style| style.border_color(colors::accent_foreground().opacity(0.45)))
            .on_click(move |_, _, cx| {
                crate::updater::download_and_install(&updater, cx);
            })
            .child(
                svg()
                    .external_path(resolve_path(icon_path))
                    .size_4()
                    .text_color(colors::accent_foreground()),
            )
            .child(i18n::t(label))
            .into_any_element(),
    )
}

fn check_action(status: &UpdateStatus, updater: Entity<Updater>) -> AnyElement {
    let busy = check_is_disabled(status);
    let button = div()
        .id("update-check-action")
        .size(px(34.0))
        .flex()
        .items_center()
        .justify_center()
        .rounded_md()
        .flex_none()
        .bg(colors::base_900())
        .border_1()
        .border_color(colors::base_800())
        .text_color(colors::base_200())
        .when(busy, |element| element.cursor_default().opacity(0.6))
        .when(!busy, |element| {
            element
                .cursor_pointer()
                .hover(|style| style.bg(colors::base_850()))
                .on_click(move |_, _, cx| {
                    crate::updater::start_check(&updater, cx);
                })
        })
        .child(update_icon(matches!(status, UpdateStatus::Checking)));

    crate::ui::cursor_tooltip::attach(
        button,
        ElementId::Name("update-check-tooltip".into()),
        i18n::t("update.check"),
    )
    .into_any_element()
}

fn check_is_disabled(status: &UpdateStatus) -> bool {
    status.is_busy() || matches!(status, UpdateStatus::Staged(_))
}

fn update_icon(spinning: bool) -> AnyElement {
    let icon = svg()
        .external_path(resolve_path("assets/icons/refresh-cw.svg"))
        .size_4()
        .text_color(colors::base_200());
    if !spinning {
        return icon.into_any_element();
    }
    icon.with_animation(
        "app-update-spinner",
        Animation::new(Duration::from_millis(850)).repeat(),
        |icon, delta| {
            icon.with_transformation(Transformation::rotate(Radians(
                -std::f32::consts::TAU * delta,
            )))
        },
    )
    .into_any_element()
}

fn update_badges(status: &UpdateStatus) -> AnyElement {
    let badges = div().flex_none().flex().items_center().gap_2().child(badge(
        format!("v{}", env!("CARGO_PKG_VERSION")),
        BadgeStyle::Neutral,
    ));

    match status {
        UpdateStatus::Idle | UpdateStatus::Errored(_) => badges,
        UpdateStatus::Checking => badges.child(loading_badge(i18n::t("update.checking"))),
        UpdateStatus::UpToDate => badges.child(badge(i18n::t("update.latest"), BadgeStyle::Green)),
        UpdateStatus::Available(version) => badges.child(badge(
            message("update.available", "version", &version.to_string()),
            BadgeStyle::Orange,
        )),
        UpdateStatus::Downloading { downloaded, total } => match total.filter(|total| *total > 0) {
            Some(total) => {
                let percent = downloaded.saturating_mul(100) / total;
                let detail = format!("{}%", percent);
                let progress = u8::try_from(percent.min(100)).unwrap_or(100);
                badges.child(progress_badge(
                    message("update.downloading", "progress", &detail),
                    f32::from(progress) / 100.0,
                ))
            }
            None => badges.child(loading_badge(message(
                "update.downloading",
                "progress",
                &format!("{} MB", downloaded / 1_048_576),
            ))),
        },
        UpdateStatus::Installing => badges.child(loading_badge(i18n::t("update.installing"))),
        UpdateStatus::Staged(_) => badges.child(loading_badge(i18n::t("update.restarting"))),
    }
    .into_any_element()
}

fn error_line(status: &UpdateStatus) -> Option<String> {
    match status {
        UpdateStatus::Errored(error) => Some(message("update.failed", "error", error)),
        _ => None,
    }
}

fn message(key: &str, placeholder: &str, value: &str) -> String {
    i18n::t(key).replace(&format!("%{{{placeholder}}}"), value)
}

#[cfg(test)]
mod tests {
    use gpui_updater::{UpdateStatus, Version};

    use super::check_is_disabled;

    #[test]
    fn update_check_stays_disabled_through_restart_but_recovers_after_errors() {
        assert!(check_is_disabled(&UpdateStatus::Staged(Version::new(
            1, 2, 3
        ))));
        assert!(!check_is_disabled(&UpdateStatus::Errored(
            "network error".to_owned()
        )));
    }
}
