use std::cell::Cell;
use std::rc::Rc;
use std::sync::Arc;

use gpui::prelude::*;
use gpui::*;
use gpui_component::{ElementExt, StyledExt};

use super::chain_drag::{self, ProjectedRow};
use super::{action_button, card, card_header, icon, icon_button, separator};
use crate::engine::{ChainItem, Engine};
use crate::ui::badge::{BadgeStyle, badge};
use crate::ui::colors;
use crate::ui::i18n;
use crate::ui::routes::{NavigateCallback, Route};

#[derive(Clone)]
pub(super) struct ChainDrag {
    pub node_id: String,
    pub(super) item: ChainItem,
    pub from_index: usize,
    pub(super) source_bounds: Rc<Cell<Option<Bounds<Pixels>>>>,
    pub(super) list_bounds: Rc<Cell<Option<Bounds<Pixels>>>>,
    pub(super) grab_offset: Rc<Cell<Point<Pixels>>>,
}

struct InvisibleDragPreview;

impl Render for InvisibleDragPreview {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div().size(px(1.0)).opacity(0.0)
    }
}

pub(super) fn chain_card(
    engine: Arc<Engine>,
    on_navigate: NavigateCallback,
    chain: Vec<ChainItem>,
    cx: &App,
) -> AnyElement {
    let clear_engine = Arc::clone(&engine);
    let item_engine = Arc::clone(&engine);
    let is_empty = chain.is_empty();
    let active_drag = chain_drag::active(cx);
    let layout_transition = chain_drag::layout_transition(cx);
    let drag_preview = chain_drag::preview_layout(cx);
    let projected_rows = chain_drag::projected_rows(&chain, active_drag);
    let list_bounds = Rc::new(Cell::new(None));
    let measured_list_bounds = Rc::clone(&list_bounds);

    card()
        .child(
            card_header(
                "assets/icons/blocks.svg",
                colors::purple(),
                "home.chain",
                "home.chainDescription",
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .child(
                        action_button("chain-add", "home.goToPlugins", "arrow-right.svg", true)
                            .on_click(move |_, window, cx| {
                                on_navigate(Route::Plugins, window, cx);
                            }),
                    )
                    .child(
                        icon_button("chain-clear", "trash-2.svg", "home.clearChain", true)
                            .on_click(move |_, _, cx| {
                                if let Err(error) = clear_engine.clear_chain() {
                                    eprintln!("failed to clear JUCE chain: {error}");
                                }
                                cx.refresh_windows();
                            }),
                    ),
            ),
        )
        .child(separator())
        .child(
            div().p_4().child(
                div()
                    .relative()
                    .w_full()
                    .flex()
                    .flex_col()
                    .gap_2()
                    .on_prepaint(move |bounds, _, _| {
                        measured_list_bounds.set(Some(bounds));
                    })
                    .when(is_empty, |element| {
                        element.child(
                            div()
                                .w_full()
                                .py_6()
                                .text_center()
                                .text_sm()
                                .text_color(colors::base_500())
                                .child(i18n::t("home.chainEmpty")),
                        )
                    })
                    .children(projected_rows.into_iter().map(move |row| {
                        let row_id = row.id();
                        let element = match row {
                            ProjectedRow::Item { index, item } => chain_item(
                                index,
                                item,
                                Arc::clone(&item_engine),
                                Rc::clone(&list_bounds),
                                active_drag.is_some(),
                            ),
                            ProjectedRow::Placeholder(item) => {
                                chain_drag::placeholder(item, Arc::clone(&item_engine))
                            }
                        };
                        chain_drag::animate_row(element, row_id, layout_transition)
                    }))
                    .when_some(drag_preview, |element, preview| {
                        element.child(render_drag_preview(preview))
                    }),
            ),
        )
        .into_any_element()
}

fn chain_item(
    index: usize,
    item: ChainItem,
    engine: Arc<Engine>,
    list_bounds: Rc<Cell<Option<Bounds<Pixels>>>>,
    is_dragging: bool,
) -> AnyElement {
    let gui_engine = Arc::clone(&engine);
    let bypass_engine = Arc::clone(&engine);
    let gui_id = item.id.clone();
    let bypass_id = item.id.clone();
    let remove_id = item.id.clone();
    let next_bypassed = !item.bypassed;
    let source_bounds = Rc::new(Cell::new(None));
    let drag = ChainDrag {
        node_id: item.id.clone(),
        item: item.clone(),
        from_index: index,
        source_bounds: Rc::clone(&source_bounds),
        list_bounds,
        grab_offset: Rc::new(Cell::new(Point::default())),
    };
    let measured_source_bounds = Rc::clone(&source_bounds);
    let drop_engine = Arc::clone(&engine);

    div()
        .relative()
        .w_full()
        .id(SharedString::from(format!("chain-item-{}", item.id)))
        .on_prepaint(move |bounds, _, _| measured_source_bounds.set(Some(bounds)))
        .child(chain_item_visual(
            &item,
            drag_handle(index, drag),
            div()
                .flex()
                .items_center()
                .gap_1()
                .child(
                    icon_button(
                        format!("gui-{index}"),
                        "external-link.svg",
                        "home.openGui",
                        false,
                    )
                    .on_click(move |_, _, _| {
                        if let Err(error) = gui_engine.open_plugin_gui(&gui_id, "ShallowHost") {
                            eprintln!("failed to open plugin editor: {error}");
                        }
                    }),
                )
                .child(
                    icon_button(
                        format!("bypass-{index}"),
                        "circle-off.svg",
                        if item.bypassed {
                            "home.unbypass"
                        } else {
                            "home.bypass"
                        },
                        false,
                    )
                    .on_click(move |_, _, cx| {
                        if let Err(error) = bypass_engine.bypass_plugin(&bypass_id, next_bypassed) {
                            eprintln!("failed to change plugin bypass: {error}");
                        }
                        cx.refresh_windows();
                    }),
                )
                .child(
                    icon_button(
                        format!("remove-{index}"),
                        "trash-2.svg",
                        "home.removeFromChain",
                        true,
                    )
                    .on_click(move |_, _, cx| {
                        if let Err(error) = engine.remove_from_chain(&remove_id) {
                            eprintln!("failed to remove plugin: {error}");
                        }
                        cx.refresh_windows();
                    }),
                ),
        ))
        .when(is_dragging, |element| {
            element.children(chain_drag::drop_zones(index, drop_engine))
        })
        .into_any_element()
}

fn drag_handle(index: usize, drag: ChainDrag) -> Stateful<Div> {
    let id = ElementId::from(SharedString::from(format!("chain-drag-{index}")));
    let handle = div()
        .id(id.clone())
        .size(px(34.0))
        .flex_none()
        .flex()
        .items_center()
        .justify_center()
        .cursor_default()
        .bg(colors::base_900())
        .border_1()
        .border_color(colors::base_800())
        .rounded_md()
        .hover(|style| {
            style
                .bg(colors::base_850())
                .border_color(colors::base_700())
        })
        .child(icon("grip-vertical.svg", colors::base_500()));
    crate::ui::cursor_tooltip::attach(handle, id, i18n::t("home.dragHandle")).on_drag(
        drag,
        |dragged, _cursor_offset, window, cx| {
            if let Some(source_bounds) = dragged.source_bounds.get() {
                dragged
                    .grab_offset
                    .set(window.mouse_position() - source_bounds.origin);
            }
            chain_drag::begin(dragged, window.mouse_position(), cx);
            cx.refresh_windows();
            cx.new(|_| InvisibleDragPreview)
        },
    )
}

fn render_drag_preview(preview: chain_drag::DragPreviewLayout) -> AnyElement {
    div()
        .absolute()
        .left_0()
        .top(preview.top)
        .w_full()
        .h(preview.height)
        .opacity(0.96)
        .shadow_lg()
        .child(chain_item_visual(
            &preview.item,
            preview_drag_handle(),
            preview_actions(),
        ))
        .into_any_element()
}

pub(super) fn chain_item_visual(
    item: &ChainItem,
    drag_handle: impl IntoElement,
    actions: impl IntoElement,
) -> Div {
    div()
        .w_full()
        .p_3()
        .flex()
        .items_center()
        .justify_between()
        .gap_3()
        .bg(colors::base_900())
        .border_1()
        .border_color(colors::base_800())
        .rounded_md()
        .child(
            div()
                .min_w_0()
                .flex_1()
                .flex()
                .items_center()
                .gap_3()
                .child(drag_handle)
                .child(
                    div()
                        .min_w_0()
                        .flex()
                        .flex_col()
                        .gap(px(1.0))
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap_2()
                                .child(
                                    div()
                                        .truncate()
                                        .text_sm()
                                        .font_semibold()
                                        .text_color(colors::base_200())
                                        .child(item.name.clone()),
                                )
                                .child(badge(item.format.clone(), BadgeStyle::Purple))
                                .child(badge(
                                    i18n::t(if item.bypassed {
                                        "home.bypassed"
                                    } else {
                                        "home.active"
                                    }),
                                    if item.bypassed {
                                        BadgeStyle::Red
                                    } else {
                                        BadgeStyle::Green
                                    },
                                )),
                        )
                        .child(
                            div()
                                .truncate()
                                .text_xs()
                                .text_color(colors::base_500())
                                .child(item.vendor.clone()),
                        ),
                ),
        )
        .child(actions)
}

pub(super) fn preview_drag_handle() -> Div {
    div()
        .size(px(34.0))
        .flex_none()
        .flex()
        .items_center()
        .justify_center()
        .bg(colors::base_900())
        .border_1()
        .border_color(colors::base_800())
        .rounded_md()
        .child(icon("grip-vertical.svg", colors::base_500()))
}

pub(super) fn preview_actions() -> Div {
    div()
        .flex()
        .items_center()
        .gap_1()
        .child(preview_action("external-link.svg"))
        .child(preview_action("circle-off.svg"))
        .child(preview_action("trash-2.svg"))
}

fn preview_action(icon_name: &'static str) -> Div {
    div()
        .size(px(34.0))
        .flex()
        .items_center()
        .justify_center()
        .rounded_md()
        .bg(colors::base_900())
        .border_1()
        .border_color(colors::base_800())
        .child(icon(icon_name, colors::base_200()))
}
