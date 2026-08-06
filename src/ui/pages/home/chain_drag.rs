use std::sync::Arc;
use std::time::Duration;

use gpui::prelude::*;
use gpui::*;

use super::chain_panel::{ChainDrag, chain_item_visual, preview_actions, preview_drag_handle};
use crate::engine::{ChainItem, Engine};
use crate::ui::colors;

#[derive(Clone, Copy)]
pub(super) struct ActiveChainDrag {
    pub from_index: usize,
    pub placeholder_index: usize,
}

#[derive(Default)]
struct ChainDragUiState {
    active: Option<ActiveChainDrag>,
    dragged: Option<ChainDrag>,
    mouse_position: Point<Pixels>,
    transition: Option<ChainLayoutTransition>,
    transition_revision: u64,
    row_stride: Pixels,
}

impl Global for ChainDragUiState {}

pub(super) enum ProjectedRow {
    Item { index: usize, item: ChainItem },
    Placeholder(ChainItem),
}

#[derive(Clone, Copy)]
pub(super) enum ProjectedRowId {
    Item(usize),
    Placeholder,
}

#[derive(Clone, Copy)]
pub(super) struct ChainLayoutTransition {
    from_index: usize,
    previous_placeholder_index: usize,
    placeholder_index: usize,
    revision: u64,
    row_stride: Pixels,
}

pub(super) struct DragPreviewLayout {
    pub item: ChainItem,
    pub top: Pixels,
    pub height: Pixels,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum DropEdge {
    Before,
    After,
}

pub(super) fn init(cx: &mut App) {
    cx.set_global(ChainDragUiState::default());
}

pub(super) fn active(cx: &App) -> Option<ActiveChainDrag> {
    cx.has_active_drag()
        .then(|| cx.global::<ChainDragUiState>().active)
        .flatten()
}

pub(super) fn begin(dragged: &ChainDrag, mouse_position: Point<Pixels>, cx: &mut App) {
    let state = cx.global_mut::<ChainDragUiState>();
    state.active = Some(ActiveChainDrag {
        from_index: dragged.from_index,
        placeholder_index: dragged.from_index,
    });
    state.dragged = Some(dragged.clone());
    state.mouse_position = mouse_position;
    state.transition = None;
    state.row_stride = dragged
        .source_bounds
        .get()
        .map_or(px(68.0), |bounds| bounds.size.height + px(8.0));
}

pub(super) fn update_mouse_position(position: Point<Pixels>, cx: &mut App) -> bool {
    if !cx.has_active_drag() {
        return false;
    }
    let state = cx.global_mut::<ChainDragUiState>();
    if state.active.is_none() || state.mouse_position == position {
        return false;
    }
    state.mouse_position = position;
    true
}

pub(super) fn preview_layout(cx: &App) -> Option<DragPreviewLayout> {
    if !cx.has_active_drag() {
        return None;
    }
    let state = cx.global::<ChainDragUiState>();
    let dragged = state.dragged.as_ref()?;
    let source_bounds = dragged.source_bounds.get()?;
    let list_bounds = dragged.list_bounds.get()?;
    let card_y = (state.mouse_position.y - dragged.grab_offset.get().y).clamp(
        list_bounds.top(),
        (list_bounds.bottom() - source_bounds.size.height).max(list_bounds.top()),
    );

    Some(DragPreviewLayout {
        item: dragged.item.clone(),
        top: card_y - list_bounds.top(),
        height: source_bounds.size.height,
    })
}

pub(super) fn layout_transition(cx: &App) -> Option<ChainLayoutTransition> {
    cx.has_active_drag()
        .then(|| cx.global::<ChainDragUiState>().transition)
        .flatten()
}

impl ProjectedRow {
    pub(super) fn id(&self) -> ProjectedRowId {
        match self {
            Self::Item { index, .. } => ProjectedRowId::Item(*index),
            Self::Placeholder(_) => ProjectedRowId::Placeholder,
        }
    }
}

pub(super) fn animate_row(
    row: AnyElement,
    row_id: ProjectedRowId,
    transition: Option<ChainLayoutTransition>,
) -> AnyElement {
    let Some(transition) = transition else {
        return row;
    };
    let initial_offset = row_initial_offset(transition, row_id);
    if initial_offset == px(0.0) {
        return row;
    }

    let row_name = match row_id {
        ProjectedRowId::Item(index) => format!("item-{index}"),
        ProjectedRowId::Placeholder => "placeholder".to_owned(),
    };
    let animation_id = ElementId::NamedInteger(
        SharedString::from(format!("chain-layout-{row_name}")),
        transition.revision,
    );
    div()
        .relative()
        .w_full()
        .with_animation(
            animation_id,
            Animation::new(Duration::from_millis(160)).with_easing(ease_in_out),
            move |element, delta| element.top(initial_offset * (1.0 - delta)),
        )
        .child(row)
        .into_any_element()
}

pub(super) fn projected_rows(
    chain: &[ChainItem],
    drag: Option<ActiveChainDrag>,
) -> Vec<ProjectedRow> {
    let Some(drag) = drag.filter(|drag| drag.from_index < chain.len()) else {
        return chain
            .iter()
            .cloned()
            .enumerate()
            .map(|(index, item)| ProjectedRow::Item { index, item })
            .collect();
    };

    let dragged_item = chain[drag.from_index].clone();
    projected_indices(chain.len(), drag)
        .into_iter()
        .map(|index| match index {
            Some(index) => ProjectedRow::Item {
                index,
                item: chain[index].clone(),
            },
            None => ProjectedRow::Placeholder(dragged_item.clone()),
        })
        .collect()
}

pub(super) fn placeholder(item: ChainItem, engine: Arc<Engine>) -> AnyElement {
    div()
        .id(SharedString::from(format!("chain-placeholder-{}", item.id)))
        .relative()
        .w_full()
        .child(chain_item_visual(&item, preview_drag_handle(), preview_actions()).invisible())
        .child(
            div()
                .absolute()
                .top_0()
                .left_0()
                .size_full()
                .rounded_md()
                .bg(colors::orange().opacity(0.025))
                .child(
                    canvas(
                        |_, _, _| {},
                        |bounds, _, window, _| {
                            let inset = px(1.0);
                            let radius = px(6.0);
                            let left = bounds.origin.x + inset;
                            let top = bounds.origin.y + inset;
                            let right = bounds.origin.x + bounds.size.width - inset;
                            let bottom = bounds.origin.y + bounds.size.height - inset;
                            let mut outline =
                                PathBuilder::stroke(px(2.0)).dash_array(&[px(16.0), px(8.0)]);

                            outline.move_to(point(left + radius, top));
                            outline.line_to(point(right - radius, top));
                            outline.arc_to(
                                point(radius, radius),
                                px(0.0),
                                false,
                                true,
                                point(right, top + radius),
                            );
                            outline.line_to(point(right, bottom - radius));
                            outline.arc_to(
                                point(radius, radius),
                                px(0.0),
                                false,
                                true,
                                point(right - radius, bottom),
                            );
                            outline.line_to(point(left + radius, bottom));
                            outline.arc_to(
                                point(radius, radius),
                                px(0.0),
                                false,
                                true,
                                point(left, bottom - radius),
                            );
                            outline.line_to(point(left, top + radius));
                            outline.arc_to(
                                point(radius, radius),
                                px(0.0),
                                false,
                                true,
                                point(left + radius, top),
                            );
                            outline.close();

                            if let Ok(outline) = outline.build() {
                                window.paint_path(outline, colors::orange().opacity(0.7));
                            }
                        },
                    )
                    .absolute()
                    .top_0()
                    .left_0()
                    .size_full(),
                ),
        )
        .can_drop(|value, _, cx| {
            let Some(dragged) = value.downcast_ref::<ChainDrag>() else {
                return false;
            };
            cx.global::<ChainDragUiState>()
                .active
                .is_some_and(|active| active.placeholder_index != dragged.from_index)
        })
        .on_drop(move |dragged: &ChainDrag, _, cx| {
            let target = cx
                .global::<ChainDragUiState>()
                .active
                .map(|active| active.placeholder_index);
            if let Some(to_index) = target
                && to_index != dragged.from_index
                && let Err(error) = engine.reorder_chain(&dragged.node_id, to_index)
            {
                eprintln!("failed to reorder plugin chain: {error}");
            }
            finish(cx);
        })
        .into_any_element()
}

pub(super) fn drop_zones(index: usize, engine: Arc<Engine>) -> [AnyElement; 2] {
    [
        drop_zone(index, DropEdge::Before, Arc::clone(&engine)).into_any_element(),
        drop_zone(index, DropEdge::After, engine).into_any_element(),
    ]
}

fn drop_zone(index: usize, edge: DropEdge, engine: Arc<Engine>) -> Stateful<Div> {
    let edge_name = match edge {
        DropEdge::Before => "before",
        DropEdge::After => "after",
    };

    div()
        .id(SharedString::from(format!(
            "chain-drop-{edge_name}-{index}"
        )))
        .absolute()
        .left_0()
        .right_0()
        .h(relative(0.5))
        .when(edge == DropEdge::Before, |zone| zone.top_0())
        .when(edge == DropEdge::After, |zone| zone.bottom_0())
        .on_drag_move::<ChainDrag>(move |event, window, cx| {
            let dragged = event.drag(cx);
            if event.bounds.contains(&event.event.position) {
                let placeholder_index = drop_index(dragged.from_index, index, edge);
                if move_placeholder(dragged.from_index, placeholder_index, cx) {
                    window.refresh();
                }
            }
        })
        .can_drop(move |value, _, _| {
            value
                .downcast_ref::<ChainDrag>()
                .and_then(|dragged| reorder_index(dragged.from_index, index, edge))
                .is_some()
        })
        .on_drop(move |dragged: &ChainDrag, _, cx| {
            let Some(to_index) = reorder_index(dragged.from_index, index, edge) else {
                return;
            };
            if let Err(error) = engine.reorder_chain(&dragged.node_id, to_index) {
                eprintln!("failed to reorder plugin chain: {error}");
            }
            finish(cx);
        })
}

fn finish(cx: &mut App) {
    let state = cx.global_mut::<ChainDragUiState>();
    state.active = None;
    state.dragged = None;
    state.transition = None;
    cx.refresh_windows();
}

fn move_placeholder(from_index: usize, placeholder_index: usize, cx: &mut App) -> bool {
    let state = cx.global_mut::<ChainDragUiState>();
    let Some(active) = state.active else {
        return false;
    };
    if active.from_index != from_index || active.placeholder_index == placeholder_index {
        return false;
    }

    state.transition_revision = state.transition_revision.wrapping_add(1);
    state.transition = Some(ChainLayoutTransition {
        from_index,
        previous_placeholder_index: active.placeholder_index,
        placeholder_index,
        revision: state.transition_revision,
        row_stride: state.row_stride,
    });
    state.active = Some(ActiveChainDrag {
        from_index,
        placeholder_index,
    });
    true
}

fn row_initial_offset(transition: ChainLayoutTransition, row_id: ProjectedRowId) -> Pixels {
    let previous = projected_indices(
        transition_length(transition),
        ActiveChainDrag {
            from_index: transition.from_index,
            placeholder_index: transition.previous_placeholder_index,
        },
    );
    let current = projected_indices(
        transition_length(transition),
        ActiveChainDrag {
            from_index: transition.from_index,
            placeholder_index: transition.placeholder_index,
        },
    );
    let needle = match row_id {
        ProjectedRowId::Item(index) => Some(index),
        ProjectedRowId::Placeholder => None,
    };
    let previous_position = previous.iter().position(|entry| *entry == needle);
    let current_position = current.iter().position(|entry| *entry == needle);

    match (previous_position, current_position) {
        (Some(previous), Some(current)) if previous > current => {
            (current..previous).fold(px(0.0), |offset, _| offset + transition.row_stride)
        }
        (Some(previous), Some(current)) if current > previous => {
            (previous..current).fold(px(0.0), |offset, _| offset - transition.row_stride)
        }
        _ => px(0.0),
    }
}

fn transition_length(transition: ChainLayoutTransition) -> usize {
    transition
        .from_index
        .max(transition.previous_placeholder_index)
        .max(transition.placeholder_index)
        + 1
}

fn projected_indices(len: usize, drag: ActiveChainDrag) -> Vec<Option<usize>> {
    if len == 0 || drag.from_index >= len {
        return Vec::new();
    }
    let mut indices = (0..len)
        .filter(|index| *index != drag.from_index)
        .map(Some)
        .collect::<Vec<_>>();
    indices.insert(drag.placeholder_index.min(len - 1), None);
    indices
}

fn reorder_index(from_index: usize, hovered_index: usize, edge: DropEdge) -> Option<usize> {
    let to_index = drop_index(from_index, hovered_index, edge);
    (to_index != from_index).then_some(to_index)
}

fn drop_index(from_index: usize, hovered_index: usize, edge: DropEdge) -> usize {
    let insertion_slot = hovered_index + usize::from(edge == DropEdge::After);
    insertion_slot.saturating_sub(usize::from(insertion_slot > from_index))
}

#[cfg(test)]
mod tests {
    use super::{
        ActiveChainDrag, ChainLayoutTransition, DropEdge, ProjectedRowId, drop_index,
        projected_indices, reorder_index, row_initial_offset,
    };
    use gpui::px;

    #[test]
    fn translates_drop_edges_to_post_removal_indices() {
        assert_eq!(reorder_index(0, 2, DropEdge::Before), Some(1));
        assert_eq!(reorder_index(0, 2, DropEdge::After), Some(2));
        assert_eq!(reorder_index(2, 0, DropEdge::Before), Some(0));
        assert_eq!(reorder_index(2, 0, DropEdge::After), Some(1));
        assert_eq!(reorder_index(1, 1, DropEdge::Before), None);
        assert_eq!(reorder_index(1, 1, DropEdge::After), None);
        assert_eq!(drop_index(0, 1, DropEdge::Before), 0);
    }

    #[test]
    fn moves_the_placeholder_without_duplicating_the_dragged_row() {
        assert_eq!(
            projected_indices(
                4,
                ActiveChainDrag {
                    from_index: 0,
                    placeholder_index: 2,
                }
            ),
            [Some(1), Some(2), None, Some(3)]
        );
        assert_eq!(
            projected_indices(
                4,
                ActiveChainDrag {
                    from_index: 3,
                    placeholder_index: 1,
                }
            ),
            [Some(0), None, Some(1), Some(2)]
        );
    }

    #[test]
    fn offsets_placeholder_and_displaced_rows_from_their_previous_positions() {
        let transition = ChainLayoutTransition {
            from_index: 0,
            previous_placeholder_index: 0,
            placeholder_index: 2,
            revision: 1,
            row_stride: px(68.0),
        };

        assert_eq!(
            row_initial_offset(transition, ProjectedRowId::Placeholder),
            px(-136.0)
        );
        assert_eq!(
            row_initial_offset(transition, ProjectedRowId::Item(1)),
            px(68.0)
        );
        assert_eq!(
            row_initial_offset(transition, ProjectedRowId::Item(2)),
            px(68.0)
        );
        assert_eq!(
            row_initial_offset(transition, ProjectedRowId::Item(3)),
            px(0.0)
        );
    }
}
