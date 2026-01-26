//! Table component for data display

use dioxus::prelude::*;

/// Table column definition
#[derive(Clone, PartialEq)]
pub struct TableColumn {
    /// Column header text
    pub header: String,
    /// Column key/identifier
    pub key: String,
}

/// Table component
/// 
/// A simple table component for displaying tabular data.
/// Supports custom columns and row data.
#[component]
pub fn Table(
    /// Table columns
    columns: Vec<TableColumn>,
    /// Table rows data (each row is a Vec of cell values matching column order)
    rows: Vec<Vec<String>>,
    /// Additional CSS classes
    class: Option<String>,
    /// Whether to show striped rows
    #[props(default = true)]
    striped: bool,
) -> Element {
    let base_classes = "min-w-full divide-y divide-gray-200";
    let custom_classes = class.unwrap_or_default();
    let classes = format!("{} {}", base_classes, custom_classes);

    rsx! {
        div {
            class: "overflow-x-auto",
            table {
                class: "{classes}",
                thead {
                    class: "bg-gray-50",
                    tr {
                        for column in &columns {
                            th {
                                class: "px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider",
                                {column.header.clone()}
                            }
                        }
                    }
                }
                tbody {
                    class: if striped { "bg-white divide-y divide-gray-200" } else { "bg-white" },
                    for (row_idx, row) in rows.iter().enumerate() {
                        tr {
                            class: if striped && row_idx % 2 == 1 {
                                "bg-gray-50"
                            } else {
                                ""
                            },
                            for cell in row {
                                td {
                                    class: "px-6 py-4 whitespace-nowrap text-sm text-gray-900",
                                    {cell.clone()}
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dioxus::core::Mutations;

    #[test]
    fn table_renders_columns_and_rows() {
        fn test_app() -> Element {
            let columns = vec![
                TableColumn { header: "Name".to_string(), key: "name".to_string() },
                TableColumn { header: "Status".to_string(), key: "status".to_string() },
            ];
            let rows = vec![
                vec!["Item 1".to_string(), "Active".to_string()],
                vec!["Item 2".to_string(), "Inactive".to_string()],
            ];
            rsx! {
                Table {
                    columns: columns,
                    rows: rows,
                }
            }
        }

        let mut vdom = VirtualDom::new(test_app);
        let mut mutations = Mutations::default();
        vdom.rebuild(&mut mutations);
        let html = dioxus_ssr::render(&vdom);
        assert!(html.contains("Name"));
        assert!(html.contains("Status"));
        assert!(html.contains("Item 1"));
        assert!(html.contains("Active"));
    }

    #[test]
    fn table_renders_empty_rows() {
        fn test_app() -> Element {
            let columns = vec![
                TableColumn { header: "Name".to_string(), key: "name".to_string() },
            ];
            let rows: Vec<Vec<String>> = vec![];
            rsx! {
                Table {
                    columns: columns,
                    rows: rows,
                }
            }
        }

        let mut vdom = VirtualDom::new(test_app);
        let mut mutations = Mutations::default();
        vdom.rebuild(&mut mutations);
        let html = dioxus_ssr::render(&vdom);
        assert!(html.contains("Name"));
    }
}
