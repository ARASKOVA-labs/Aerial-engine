use crate::graph::LayoutGraph;

use std::collections::HashMap;

/// Escape text for safe inclusion in SVG XML.
fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

fn get_icon_svg(name: &str) -> &'static str {
    match name {
        "s3" | "database" | "bucket" | "storage" => {
            r#"<svg xmlns="http://www.w3.org/2000/svg" width="32" height="32" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><ellipse cx="12" cy="5" rx="9" ry="3"/><path d="M3 5V19A9 3 0 0 0 21 19V5"/><path d="M3 12A9 3 0 0 0 21 12"/></svg>"#
        }
        "lambda" | "function" | "code" | "compute" => {
            r#"<svg xmlns="http://www.w3.org/2000/svg" width="32" height="32" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="16 18 22 12 16 6"/><polyline points="8 6 2 12 8 18"/></svg>"#
        }
        "cloud" | "cloudfront" | "cdn" | "network" => {
            r#"<svg xmlns="http://www.w3.org/2000/svg" width="32" height="32" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M17.5 19A5.5 5.5 0 0 0 18 8h-1.26a8 8 0 1 0-11.62 9"/></svg>"#
        }
        "server" | "ec2" | "instance" => {
            r#"<svg xmlns="http://www.w3.org/2000/svg" width="32" height="32" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect width="20" height="8" x="2" y="2" rx="2" ry="2"/><rect width="20" height="8" x="2" y="14" rx="2" ry="2"/><line x1="6" x2="6.01" y1="6" y2="6"/><line x1="6" x2="6.01" y1="18" y2="18"/></svg>"#
        }
        "user" | "client" | "person" => {
            r#"<svg xmlns="http://www.w3.org/2000/svg" width="32" height="32" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M19 21v-2a4 4 0 0 0-4-4H9a4 4 0 0 0-4 4v2"/><circle cx="12" cy="7" r="4"/></svg>"#
        }
        "model" | "brain" | "ai" | "llm" => {
            r#"<svg xmlns="http://www.w3.org/2000/svg" width="32" height="32" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M12 5a3 3 0 1 0-5.997.125 4 4 0 0 0-2.526 5.77 4 4 0 0 0 .556 6.588A4 4 0 1 0 12 18Z"/><path d="M12 5a3 3 0 1 1 5.997.125 4 4 0 0 1 2.526 5.77 4 4 0 0 1-.556 6.588A4 4 0 1 1 12 18Z"/><path d="M15 13a4.5 4.5 0 0 1-3-4 4.5 4.5 0 0 1-3 4"/><path d="M17.599 6.5a3 3 0 0 0 .399-1.375"/></svg>"#
        }
        "message" | "queue" | "sns" | "sqs" | "event" => {
            r#"<svg xmlns="http://www.w3.org/2000/svg" width="32" height="32" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z"/></svg>"#
        }
        _ => {
            r#"<svg xmlns="http://www.w3.org/2000/svg" width="32" height="32" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect width="18" height="18" x="3" y="3" rx="2" ry="2"/></svg>"#
        }
    }
}

pub fn generate_svg(graph: &LayoutGraph) -> (String, HashMap<String, (f64, f64, f64, f64)>) {
    let mut max_x = 0.0_f64;
    let mut max_y = 0.0_f64;
    let mut min_x_global = f64::MAX;

    for node in graph.node_weights() {
        if node.x + node.width > max_x {
            max_x = node.x + node.width;
        }
        if node.y + node.height > max_y {
            max_y = node.y + node.height;
        }
        if node.x < min_x_global {
            min_x_global = node.x;
        }
    }

    let width = max_x + 100.0;
    let height = max_y + 100.0;

    // Offset viewbox if min_x is less than 50
    let vx = (min_x_global - 50.0).min(0.0);

    let mut svg = format!(
        r##"<svg width="{}" height="{}" viewBox="{} 0 {} {}" xmlns="http://www.w3.org/2000/svg">"##,
        width,
        height,
        vx,
        width - vx,
        height
    );

    // Font definitions
    svg.push_str(
        r##"
    <defs>
        <style>
            text {
                font-family: 'Space Grotesk', ui-sans-serif, system-ui, sans-serif;
            }
        </style>
        <marker id="arrowhead" markerWidth="10" markerHeight="7" refX="9" refY="3.5" orient="auto">
            <polygon points="0 0, 10 3.5, 0 7" fill="#71717a" />
        </marker>
    </defs>
    "##,
    );

    // Compute and draw groups
    let mut groups: HashMap<String, (f64, f64, f64, f64)> = HashMap::new();
    for node in graph.node_weights() {
        if let Some(g) = &node.group {
            let entry = groups
                .entry(g.clone())
                .or_insert((f64::MAX, f64::MAX, f64::MIN, f64::MIN));
            entry.0 = entry.0.min(node.x);
            entry.1 = entry.1.min(node.y);
            entry.2 = entry.2.max(node.x + node.width);
            entry.3 = entry.3.max(node.y + node.height);
        }
    }

    for (group_name, (min_x, min_y, max_x, max_y)) in &groups {
        let pad = 40.0;
        let gx = min_x - pad;
        let gy = min_y - pad - 24.0;
        let gw = (max_x - min_x) + pad * 2.0;
        let gh = (max_y - min_y) + pad * 2.0 + 24.0;

        svg.push_str(&format!(
            r##"<rect x="{}" y="{}" width="{}" height="{}" rx="16" fill="rgba(39, 39, 42, 0.4)" stroke="#3f3f46" stroke-width="1.5" stroke-dasharray="6 6" />"##,
            gx, gy, gw, gh
        ));

        svg.push_str(&format!(
            r##"<text x="{}" y="{}" fill="#a1a1aa" font-size="13" font-weight="600" letter-spacing="1">📁 {}</text>"##,
            gx + 16.0, gy + 24.0, xml_escape(&group_name.to_uppercase())
        ));
    }

    // Draw edges
    for edge in graph.edge_weights() {
        let pts = &edge.points;
        if pts.len() >= 2 {
            let sx = pts[0].0;
            let sy = pts[0].1;
            let tx = pts[1].0;
            let ty = pts[1].1;

            let tension = (ty - sy).abs().min(80.0);
            let cy1 = sy + tension;
            let cy2 = ty - tension;

            let path = format!(
                "M {} {} C {} {}, {} {}, {} {}",
                sx, sy, sx, cy1, tx, cy2, tx, ty
            );

            svg.push_str(&format!(
                r##"<path d="{}" stroke="#52525b" stroke-width="2" fill="none" marker-end="url(#arrowhead)"/>"##,
                path
            ));

            if let Some(label) = &edge.label {
                let mx = (sx + tx) / 2.0;
                let my_label = (sy + ty) / 2.0 - 10.0;

                let pill = format!(
                    r#"<foreignObject x="{}" y="{}" width="160" height="30">
                        <div xmlns="http://www.w3.org/1999/xhtml" style="
                            display: flex; align-items: center; justify-content: center;
                            width: 100%; height: 100%;
                        ">
                            <span style="
                                background-color: #18181b;
                                color: #a1a1aa;
                                font-family: 'Space Grotesk', sans-serif;
                                font-size: 11px;
                                font-weight: 500;
                                padding: 2px 8px;
                                border-radius: 12px;
                                border: 1px solid #3f3f46;
                            ">{}</span>
                        </div>
                    </foreignObject>"#,
                    mx - 80.0,
                    my_label - 15.0,
                    xml_escape(label)
                );
                svg.push_str(&pill);
            }
        }
    }

    // Draw nodes
    for node in graph.node_weights() {
        let icon_html = match &node.icon {
            Some(i) => format!(
                "<div style='margin-bottom: 8px; color: {};'>{}</div>",
                node.font_color,
                get_icon_svg(i)
            ),
            None => "".to_string(),
        };

        let html = format!(
            r#"<foreignObject x="{}" y="{}" width="{}" height="{}">
                <div xmlns="http://www.w3.org/1999/xhtml" style="
                    display: flex; 
                    flex-direction: column;
                    align-items: center; 
                    justify-content: center; 
                    width: 100%; 
                    height: 100%; 
                    box-sizing: border-box; 
                    padding: 12px; 
                    background-color: {}; 
                    border: 1.5px solid {}; 
                    border-radius: 12px; 
                    color: {}; 
                    font-family: 'Space Grotesk', ui-sans-serif, system-ui, sans-serif; 
                    font-size: 14px; 
                    font-weight: 600; 
                    text-align: center; 
                    line-height: 1.4;
                    box-shadow: 0 4px 6px -1px rgba(0, 0, 0, 0.3), 0 2px 4px -2px rgba(0, 0, 0, 0.3);
                ">
                    {}
                    <div>{}</div>
                </div>
            </foreignObject>"#,
            node.x,
            node.y,
            node.width,
            node.height,
            node.fill,
            node.stroke,
            node.font_color,
            icon_html,
            xml_escape(&node.label)
        );
        svg.push_str(&html);
    }

    let mut hit_map = HashMap::new();
    for node in graph.node_weights() {
        hit_map.insert(node.id.clone(), (node.x, node.y, node.width, node.height));
    }

    svg.push_str("</svg>");
    (svg, hit_map)
}
