//! Document packs components

use dioxus::prelude::*;
use ui_design_system::{Table, TableColumn};

/// Document pack data structure (placeholder - will connect to GraphQL later)
#[derive(Debug, Clone)]
pub struct DocPack {
    pub id: String,
    pub pack_id: String,
    pub name: String,
    pub jurisdiction: String,
    pub status: String,
    pub created_at: String,
    pub document_count: u32,
}

/// Document packs view component
/// 
/// Displays a table of compliance document packs with their details.
#[component]
pub fn DocPacksView() -> Element {
    // Placeholder data - will be replaced with GraphQL query
    let doc_packs = vec![
        DocPack {
            id: "1".to_string(),
            pack_id: "PACK-001".to_string(),
            name: "Export Authorization Pack".to_string(),
            jurisdiction: "US".to_string(),
            status: "Complete".to_string(),
            created_at: "2026-01-20 09:00:00".to_string(),
            document_count: 5,
        },
        DocPack {
            id: "2".to_string(),
            pack_id: "PACK-002".to_string(),
            name: "Transfer Documentation".to_string(),
            jurisdiction: "CA".to_string(),
            status: "In Progress".to_string(),
            created_at: "2026-01-25 11:00:00".to_string(),
            document_count: 3,
        },
    ];

    rsx! {
        div {
            class: "bg-white rounded-lg shadow p-8",
            h2 {
                class: "text-xl font-semibold text-gray-800 mb-4",
                "Document Packs"
            }
            Table {
                columns: vec![
                    TableColumn { header: "Pack ID".to_string(), key: "pack_id".to_string() },
                    TableColumn { header: "Name".to_string(), key: "name".to_string() },
                    TableColumn { header: "Jurisdiction".to_string(), key: "jurisdiction".to_string() },
                    TableColumn { header: "Status".to_string(), key: "status".to_string() },
                    TableColumn { header: "Created".to_string(), key: "created".to_string() },
                    TableColumn { header: "Documents".to_string(), key: "documents".to_string() },
                ],
                rows: doc_packs.iter().map(|pack| {
                    vec![
                        pack.pack_id.clone(),
                        pack.name.clone(),
                        pack.jurisdiction.clone(),
                        pack.status.clone(),
                        pack.created_at.clone(),
                        pack.document_count.to_string(),
                    ]
                }).collect(),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dioxus::core::Mutations;

    #[test]
    fn doc_packs_view_renders() {
        let mut vdom = VirtualDom::new(DocPacksView);
        let mut mutations = Mutations::default();
        vdom.rebuild(&mut mutations);
        let html = dioxus_ssr::render(&vdom);
        assert!(html.contains("Document Packs"));
        assert!(html.contains("Pack ID"));
    }
}
