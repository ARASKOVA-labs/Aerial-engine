extern crate pest;
#[macro_use]
extern crate pest_derive;

pub mod ast;
pub mod error;
pub mod parser;
pub mod printer;
pub mod transpiler;

pub use ast::*;
pub use error::ArasError;

#[derive(Parser)]
#[grammar = "grammar.pest"]
struct ArasParser;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_and_transpile() {
        let source = r##"
@type: architecture
@theme: midnight

[user] --> [api_gateway]: HTTPS
[api_gateway] --> [auth_service]: validate_token
[api_gateway] --> [inference_node]: forward_request

[inference_node].hardware: Snapdragon X Elite
[inference_node].model: rustama-engine/qwen2.5

group "Edge Cluster" {
  [inference_node]
  [cache]
}

style [inference_node] {
  fill: "#6366f1"
  font-color: "#ffffff"
}
"##;
        let ast = parser::parse(source).expect("Failed to parse");

        let output = transpiler::transpile(&ast).expect("Failed to transpile");

        println!("AST:\n{:#?}", ast);
        println!("D2:\n{}", output.d2_source);

        assert!(output.d2_source.contains("user -> api_gateway: HTTPS"));
        assert!(output.d2_source.contains("edge_cluster: \"Edge Cluster\""));
        assert!(
            output
                .d2_source
                .contains("inference_node.style.fill: \"#6366f1\"")
        );
    }
}
