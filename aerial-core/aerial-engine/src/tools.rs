/// aerial-engine/src/tools.rs
/// Active tool state enum — determines what happens on mouse events.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActiveTool {
    Select,
    Hand,
    FreeDraw,
    FountainPen,
    Rectangle,
    Ellipse,
    Line,
    Arrow,
    Highlighter,
    Text,
    Eraser,
    MagicPen,
    LaserPen,
}

impl Default for ActiveTool {
    fn default() -> Self {
        Self::Select
    }
}
