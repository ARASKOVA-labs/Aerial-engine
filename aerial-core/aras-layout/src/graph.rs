use petgraph::graph::{DiGraph, NodeIndex};
use std::collections::HashMap;
use aras_dsl::ast::{Diagram, Stmt, Arrow};

#[derive(Debug, Clone)]
pub struct LayoutNode {
    pub id: String,
    pub label: String,
    pub group: Option<String>,
    pub icon: Option<String>,
    pub width: f64,
    pub height: f64,
    
    // Computed layout
    pub x: f64,
    pub y: f64,
    pub layer: usize,
    
    // Styling
    pub fill: String,
    pub stroke: String,
    pub font_color: String,
}

impl Default for LayoutNode {
    fn default() -> Self {
        Self {
            id: String::new(),
            label: String::new(),
            group: None,
            icon: None,
            width: 260.0,
            height: 90.0,
            x: 0.0,
            y: 0.0,
            layer: 0,
            fill: "#18181b".to_string(),
            stroke: "#3f3f46".to_string(), // Softer border by default
            font_color: "#f4f4f5".to_string(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct LayoutEdge {
    pub source: NodeIndex,
    pub target: NodeIndex,
    pub label: Option<String>,
    pub kind: Arrow,
    
    // Computed layout
    pub points: Vec<(f64, f64)>,
}

pub type LayoutGraph = DiGraph<LayoutNode, LayoutEdge>;

pub fn build_graph(diagram: &Diagram) -> (LayoutGraph, HashMap<String, NodeIndex>) {
    let mut graph = DiGraph::new();
    let mut node_indices = HashMap::new();

    // 1. Gather all nodes (explicit and implicit)
    let mut process_stmt = |stmt: &Stmt, group_label: Option<&str>| {
        match stmt {
            Stmt::Node(id) => {
                if !node_indices.contains_key(&id.0) {
                    let mut node = LayoutNode::default();
                    node.id = id.0.clone();
                    node.label = id.0.clone();
                    node.group = group_label.map(|s| s.to_string());
                    let idx = graph.add_node(node);
                    node_indices.insert(id.0.clone(), idx);
                } else if let Some(g) = group_label {
                    let idx = node_indices[&id.0];
                    graph[idx].group = Some(g.to_string());
                }
            }
            Stmt::NodeDecl(id, label) => {
                if !node_indices.contains_key(&id.0) {
                    let mut node = LayoutNode::default();
                    node.id = id.0.clone();
                    node.label = label.clone();
                    node.group = group_label.map(|s| s.to_string());
                    let idx = graph.add_node(node);
                    node_indices.insert(id.0.clone(), idx);
                } else {
                    let idx = node_indices[&id.0];
                    graph[idx].label = label.clone();
                    if let Some(g) = group_label {
                        graph[idx].group = Some(g.to_string());
                    }
                }
            }
            Stmt::Connection(conn) => {
                for id in [&conn.from, &conn.to] {
                    if !node_indices.contains_key(&id.0) {
                        let mut node = LayoutNode::default();
                        node.id = id.0.clone();
                        node.label = id.0.clone();
                        node.group = group_label.map(|s| s.to_string());
                        let idx = graph.add_node(node);
                        node_indices.insert(id.0.clone(), idx);
                    } else if let Some(g) = group_label {
                        let idx = node_indices[&id.0];
                        graph[idx].group = Some(g.to_string());
                    }
                }
            }
            Stmt::NodeAttr(attr) => {
                if let Some(idx) = node_indices.get(&attr.node.0) {
                    let val = attr.value.trim_matches('"').to_string();
                    if attr.key == "icon" {
                        graph[*idx].icon = Some(val);
                    }
                }
            }
            Stmt::Style(style) => {
                if let Some(idx) = node_indices.get(&style.node.0) {
                    for prop in &style.props {
                        let val = prop.value.trim_matches('"').to_string();
                        match prop.key.as_str() {
                            "fill" => graph[*idx].fill = val,
                            "stroke" => graph[*idx].stroke = val,
                            "font-color" => graph[*idx].font_color = val,
                            "icon" => graph[*idx].icon = Some(val),
                            _ => {}
                        }
                    }
                }
            }
            _ => {}
        }
    };

    // First pass: normal statements
    for stmt in &diagram.stmts {
        if let Stmt::Group(group) = stmt {
            for inner in &group.stmts {
                process_stmt(inner, Some(&group.label));
            }
        } else {
            process_stmt(stmt, None);
        }
    }

    // 2. Add edges
    for stmt in &diagram.stmts {
        if let Stmt::Connection(conn) = stmt {
            let source_idx = node_indices[&conn.from.0];
            let target_idx = node_indices[&conn.to.0];
            
            graph.add_edge(source_idx, target_idx, LayoutEdge {
                source: source_idx,
                target: target_idx,
                label: conn.label.clone(),
                kind: conn.arrow.clone(),
                points: Vec::new(),
            });
        }
    }

    (graph, node_indices)
}
