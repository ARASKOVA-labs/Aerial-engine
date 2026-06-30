use serde::{Deserialize, Serialize};

/// The root of every parsed `.aras` file.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Diagram {
    /// @type, @theme, @direction, @title, etc.
    pub meta: Vec<MetaEntry>,
    /// Top-level statements (nodes, edges, groups, style blocks).
    pub stmts: Vec<Stmt>,
}

/// A @key: value metadata entry.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MetaEntry {
    pub key: String,
    pub value: String,
}

/// Every construct that can appear at diagram scope or inside a group.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Stmt {
    /// `[node_id]`  — bare node mention
    Node(NodeId),

    /// `[node_id]: "Display Label"`
    NodeDecl(NodeId, String),

    /// `[node].key: value`
    NodeAttr(NodeAttr),

    /// `[a] --> [b]: label`
    Connection(Connection),

    /// `group "Name" { ... }`
    Group(Group),

    /// `style [node] { fill: "#fff" }`
    Style(StyleBlock),
}

/// A node identifier — the raw string inside `[...]`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct NodeId(pub String);

/// `[node].key: value`
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct NodeAttr {
    pub node: NodeId,
    pub key: String,
    pub value: String,
}

/// An edge between two nodes.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Connection {
    pub from: NodeId,
    pub to: NodeId,
    pub arrow: Arrow,
    /// Optional edge label.
    pub label: Option<String>,
}

/// Arrow direction / style variants.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Arrow {
    /// `-->`
    Directed,
    /// `<--`
    Reverse,
    /// `<->`
    Bidirectional,
    /// `--`
    Undirected,
    /// `-..->`  async / dashed
    Async,
}

/// `group "Name" { stmts }`
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Group {
    pub label: String,
    pub stmts: Vec<Stmt>,
}

/// `style [node] { prop: value }`
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StyleBlock {
    pub node: NodeId,
    pub props: Vec<StyleProp>,
}

/// One `key: value` inside a style block.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StyleProp {
    pub key: String,
    pub value: String,
}
