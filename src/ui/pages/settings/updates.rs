use std::time::{Duration, Instant};

use gpui::prelude::*;
use gpui::*;
use gpui_updater::{UpdateStatus, Updater};

use super::{ToggleRowProps, card, resolve_path, row, separator, toggle_row};
use crate::config::SystemSettings;
use crate::ui::badge::{BadgeStyle, badge, loading_badge};
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
    let status = updater.read(cx).status().clone();
    let busy = status.is_busy();
    let action_label = action_label(&status);
    let error_line = error_line(&status);
    let header_badges = update_badges(&status);
    let action_updater = updater.clone();

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
                        .id("update-primary-action")
                        .h(px(34.0))
                        .px_3()
                        .flex()
                        .items_center()
                        .gap_2()
                        .rounded_md()
                        .flex_none()
                        .bg(colors::base_900())
                        .border_1()
                        .border_color(colors::base_800())
                        .control_text()
                        .text_color(colors::base_200())
                        .when(busy, |element| element.cursor_default().opacity(0.6))
                        .when(!busy, |element| {
                            element
                                .cursor_pointer()
                                .hover(|style| style.bg(colors::base_850()))
                                .on_click(move |_, _, cx| {
                                    crate::updater::run_primary_action(&action_updater, cx);
                                })
                        })
                        .child(update_icon(busy))
                        .child(action_label),
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
                std::f32::consts::TAU * delta,
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
        UpdateStatus::Downloading { downloaded, total } => {
            let detail = match total.filter(|total| *total > 0) {
                Some(total) => format!("{}%", downloaded.saturating_mul(100) / total),
                None => format!("{} MB", downloaded / 1_048_576),
            };
            badges.child(loading_badge(message(
                "update.downloading",
                "progress",
                &detail,
            )))
        }
        UpdateStatus::Installing => badges.child(loading_badge(i18n::t("update.installing"))),
        UpdateStatus::Staged(version) => badges.child(badge(
            message("update.ready", "version", &version.to_string()),
            BadgeStyle::Green,
        )),
    }
    .into_any_element()
}

fn action_label(status: &UpdateStatus) -> String {
    i18n::t(match status {
        UpdateStatus::Available(_) => "update.install",
        UpdateStatus::Staged(_) => "update.restart",
        _ => "update.check",
    })
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
