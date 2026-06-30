pub mod graph;
pub mod sugiyama;
pub mod svg;

use aras_dsl::ast::Diagram;

pub fn render_svg(
    diagram: &Diagram,
) -> (
    String,
    std::collections::HashMap<String, (f64, f64, f64, f64)>,
) {
    let (mut g, _) = graph::build_graph(diagram);
    sugiyama::layout(&mut g);
    svg::generate_svg(&g)
}

#[cfg(test)]
mod tests {
    use super::*;
    use aras_dsl::parser;

    #[test]
    fn test_layout_and_render() {
        let source = r##"
@type: architecture

[user] --> [api_gateway]: HTTPS
[api_gateway] --> [auth_service]: validate
[api_gateway] --> [inference_node]

style [inference_node] {
  fill: "#6366f1"
  font-color: "#ffffff"
}
"##;
        let ast = parser::parse(source).unwrap();
        let (svg_out, _) = render_svg(&ast);
        assert!(svg_out.contains("<svg"));
        assert!(svg_out.contains("inference_node"));
    }
}
