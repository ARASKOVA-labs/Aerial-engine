use crate::ast::*;

pub fn print(diagram: &Diagram) -> String {
    let mut out = String::new();

    // 1. Meta
    for meta in &diagram.meta {
        out.push_str(&format!("@{}: {}\n", meta.key, meta.value));
    }
    if !diagram.meta.is_empty() {
        out.push('\n');
    }

    // 2. Statements
    for stmt in &diagram.stmts {
        out.push_str(&print_stmt(stmt, 0));
        out.push('\n');
    }

    out
}

fn print_stmt(stmt: &Stmt, indent: usize) -> String {
    let ind = "  ".repeat(indent);
    match stmt {
        Stmt::Node(id) => format!("{}[{}]", ind, id.0),
        Stmt::NodeDecl(id, label) => format!("{}[{}]: \"{}\"", ind, id.0, label),
        Stmt::NodeAttr(attr) => format!("{}[{}].{}: {}", ind, attr.node.0, attr.key, attr.value),
        Stmt::Connection(conn) => {
            let arrow_str = match conn.arrow {
                Arrow::Directed => "-->",
                Arrow::Reverse => "<--",
                Arrow::Bidirectional => "<->",
                Arrow::Undirected => "--",
                Arrow::Async => "-..->",
            };
            if let Some(label) = &conn.label {
                format!(
                    "{}[{}] {} [{}]: {}",
                    ind, conn.from.0, arrow_str, conn.to.0, label
                )
            } else {
                format!("{}[{}] {} [{}]", ind, conn.from.0, arrow_str, conn.to.0)
            }
        }
        Stmt::Group(group) => {
            let mut g = format!("{}group \"{}\" {{\n", ind, group.label);
            for s in &group.stmts {
                g.push_str(&print_stmt(s, indent + 1));
                g.push('\n');
            }
            g.push_str(&format!("{}}}", ind));
            g
        }
        Stmt::Style(style) => {
            let mut s = format!("{}style [{}] {{\n", ind, style.node.0);
            for prop in &style.props {
                s.push_str(&format!("{}  {}: {}\n", ind, prop.key, prop.value));
            }
            s.push_str(&format!("{}}}", ind));
            s
        }
    }
}
