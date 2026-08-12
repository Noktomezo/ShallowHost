use super::{LibraryRow, PluginLibraryState, build_rows};
use crate::ui::pages::plugins::PluginItem;

fn plugin_with_format(id: &str, author: &str, format: &str) -> PluginItem {
    PluginItem {
        id: id.into(),
        name: id.into(),
        vendor: author.into(),
        format: format.into(),
        path: String::new(),
        in_chain: false,
        initializing: false,
    }
}

fn plugin(id: &str, author: &str) -> PluginItem {
    plugin_with_format(id, author, "VST3")
}

#[test]
fn collapsed_authors_only_emit_header_rows() {
    let mut state = PluginLibraryState {
        grouped_by_author: true,
        ..PluginLibraryState::default()
    };
    let plugins = vec![
        plugin("a", "Acme"),
        plugin("b", "Acme"),
        plugin("c", "Waves"),
    ];
    assert_eq!(build_rows(&plugins, &state).len(), 2);

    state.open_authors.insert(String::from("Acme"));
    let rows = build_rows(&plugins, &state);
    assert_eq!(rows.len(), 4);
    assert!(matches!(rows[1], LibraryRow::AuthorPlugin { .. }));
}

#[test]
fn author_header_lists_each_supplied_format_once() {
    let plugins = vec![
        plugin_with_format("a", "Acme", "VST"),
        plugin_with_format("b", "Acme", "VST2"),
        plugin_with_format("c", "Acme", "VST3"),
    ];

    let rows = build_rows(&plugins, &PluginLibraryState::new(true));
    let LibraryRow::AuthorHeader(header) = &rows[0] else {
        panic!("first row must be the author header");
    };

    assert_eq!(header.formats, ["VST2", "VST3"]);
}
