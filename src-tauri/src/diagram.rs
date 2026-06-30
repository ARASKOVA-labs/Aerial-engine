use aras_dsl::{parser, printer, ast::{Stmt, NodeId}};
use aras_layout::render_svg;
use serde::Serialize;
use std::collections::HashMap;

#[derive(Serialize)]
pub struct RenderResult {
    pub svg: String,
    pub hit_map: HashMap<String, (f64, f64, f64, f64)>,
}

#[tauri::command]
pub async fn render_diagram(code: String) -> Result<RenderResult, String> {
    tokio::task::spawn_blocking(move || {
        let ast = parser::parse(&code)
            .map_err(|e| format!("Failed to parse diagram: {}", e))?;

        let (svg, hit_map) = render_svg(&ast);

        Ok(RenderResult { svg, hit_map })
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn update_diagram_node(code: String, node_id: String, new_label: String) -> Result<String, String> {
    tokio::task::spawn_blocking(move || {
        let mut ast = parser::parse(&code)
            .map_err(|e| format!("Failed to parse diagram: {}", e))?;

        let mut found = false;

        // Search and update NodeDecl
        for stmt in &mut ast.stmts {
            match stmt {
                Stmt::NodeDecl(id, label) if id.0 == node_id => {
                    *label = new_label.clone();
                    found = true;
                }
                Stmt::Group(group) => {
                    for inner_stmt in &mut group.stmts {
                        if let Stmt::NodeDecl(id, label) = inner_stmt {
                            if id.0 == node_id {
                                *label = new_label.clone();
                                found = true;
                            }
                        }
                    }
                }
                _ => {}
            }
        }

        // If node decl wasn't found but node exists, promote Stmt::Node to Stmt::NodeDecl
        if !found {
            for stmt in &mut ast.stmts {
                if let Stmt::Node(id) = stmt {
                    if id.0 == node_id {
                        *stmt = Stmt::NodeDecl(id.clone(), new_label.clone());
                        found = true;
                        break;
                    }
                }
            }
        }

        // If it STILL wasn't found (implicitly defined in an edge), add it at the top
        if !found {
            ast.stmts.insert(0, Stmt::NodeDecl(NodeId(node_id), new_label));
        }

        Ok(printer::print(&ast))
    })
    .await
    .map_err(|e| e.to_string())?
}

