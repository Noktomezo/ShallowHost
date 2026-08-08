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
