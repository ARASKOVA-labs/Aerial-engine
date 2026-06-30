use pest::Parser;
use pest::iterators::Pair;

use crate::ast::*;
use crate::error::ArasError;
use crate::ArasParser;
use crate::Rule;

/// Parse a raw `.aras` source string into a [`Diagram`].
pub fn parse(source: &str) -> Result<Diagram, ArasError> {
    let pairs = ArasParser::parse(Rule::diagram, source)
        .map_err(|e| ArasError::Parse(e.to_string()))?;

    let mut meta: Vec<MetaEntry> = Vec::new();
    let mut stmts: Vec<Stmt> = Vec::new();

    for pair in pairs {
        match pair.as_rule() {
            Rule::diagram => {
                for inner in pair.into_inner() {
                    if inner.as_rule() == Rule::stmt {
                        if let Some(stmt_inner) = inner.into_inner().next() {
                            if stmt_inner.as_rule() == Rule::meta_stmt {
                                meta.push(parse_meta(stmt_inner));
                            } else {
                                if let Some(stmt) = parse_stmt(stmt_inner)? {
                                    stmts.push(stmt);
                                }
                            }
                        }
                    }
                }
            }
            _ => {}
        }
    }

    Ok(Diagram { meta, stmts })
}

// ── Internal helpers ──────────────────────────────────────────────────────────

fn parse_meta(pair: Pair<Rule>) -> MetaEntry {
    let mut inner = pair.into_inner();
    let key = inner.next().unwrap().as_str().trim().to_string();
    let value = parse_value(inner.next().unwrap());
    MetaEntry { key, value }
}

fn parse_value(pair: Pair<Rule>) -> String {
    match pair.as_rule() {
        Rule::value => {
            let inner = pair.into_inner().next().unwrap();
            parse_value(inner)
        }
        Rule::quoted => {
            // value is inside quotes
            let s = pair.as_str();
            s[1..s.len() - 1].to_string()
        }
        Rule::bare_value => pair.as_str().trim().to_string(),
        _ => pair.as_str().trim().to_string(),
    }
}

fn parse_node_id(pair: Pair<Rule>) -> NodeId {
    let ident = pair.into_inner().next().unwrap();
    NodeId(ident.as_str().to_string())
}

fn parse_stmt(pair: Pair<Rule>) -> Result<Option<Stmt>, ArasError> {
    if pair.as_rule() == Rule::stmt {
        if let Some(inner) = pair.into_inner().next() {
            if inner.as_rule() == Rule::meta_stmt {
                return Ok(None);
            }
            return parse_stmt(inner);
        } else {
            return Ok(None);
        }
    }

    match pair.as_rule() {
        Rule::node_ref => Ok(Some(Stmt::Node(parse_node_id(pair)))),

        Rule::node_decl => {
            let mut inner = pair.into_inner();
            let id = parse_node_id(inner.next().unwrap());
            let label = inner
                .next()
                .map(|p| parse_value(p))
                .unwrap_or_default();
            Ok(Some(Stmt::NodeDecl(id, label)))
        }

        Rule::node_attr => {
            let mut inner = pair.into_inner();
            let node = parse_node_id(inner.next().unwrap());
            let key = inner.next().unwrap().as_str().trim().to_string();
            let value = inner
                .next()
                .map(|p| p.as_str().trim().to_string())
                .unwrap_or_default();
            Ok(Some(Stmt::NodeAttr(NodeAttr { node, key, value })))
        }

        Rule::connection => {
            let mut inner = pair.into_inner();
            let from = parse_node_id(inner.next().unwrap());

            let arrow_pair = inner.next().unwrap();
            let arrow = parse_arrow(arrow_pair);

            let to = parse_node_id(inner.next().unwrap());

            let label = inner.next().map(|p| {
                p.into_inner()
                    .next()
                    .map(|t| t.as_str().trim().to_string())
                    .unwrap_or_default()
            });

            Ok(Some(Stmt::Connection(Connection { from, to, arrow, label })))
        }

        Rule::group_stmt => {
            let mut inner = pair.into_inner();
            let label = parse_value(inner.next().unwrap());

            let group_inner = inner.next().unwrap(); // Rule::group_inner
            let mut group_stmts = Vec::new();
            for child in group_inner.into_inner() {
                if child.as_rule() != Rule::EOI {
                    if let Some(s) = parse_stmt(child)? {
                        group_stmts.push(s);
                    }
                }
            }
            Ok(Some(Stmt::Group(Group { label, stmts: group_stmts })))
        }

        Rule::style_block => {
            let mut inner = pair.into_inner();
            let node = parse_node_id(inner.next().unwrap());
            let mut props = Vec::new();
            for prop_pair in inner {
                if prop_pair.as_rule() == Rule::style_prop {
                    let mut pp = prop_pair.into_inner();
                    let key = pp.next().unwrap().as_str().trim().to_string();
                    let value = pp
                        .next()
                        .map(|p| p.as_str().trim().to_string())
                        .unwrap_or_default();
                    props.push(StyleProp { key, value });
                }
            }
            Ok(Some(Stmt::Style(StyleBlock { node, props })))
        }

        Rule::EOI => Ok(None),

        r => Err(ArasError::Parse(format!("Unexpected rule: {:?}", r))),
    }
}

fn parse_arrow(pair: Pair<Rule>) -> Arrow {
    let inner = pair.into_inner().next().unwrap();
    match inner.as_rule() {
        Rule::arrow_directed   => Arrow::Directed,
        Rule::arrow_reverse    => Arrow::Reverse,
        Rule::arrow_bidir      => Arrow::Bidirectional,
        Rule::arrow_undirected => Arrow::Undirected,
        Rule::arrow_async      => Arrow::Async,
        _ => Arrow::Directed,
    }
}
