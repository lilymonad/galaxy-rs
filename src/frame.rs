use las::Color;

#[derive(Clone, Copy)]
pub enum NodeType {
    Root,
    Arm,
    Ext,
    Loner,
    System,
}

impl Into<Color> for NodeType {
    fn into(self) -> Color {
        match self {
            NodeType::Root   => Color::new(u16::MAX, 0, 0),
            NodeType::Arm    => Color::new(u16::MAX, 0, 0),
            NodeType::Ext    => Color::new(u16::MAX, u16::MAX, 0),
            NodeType::Loner  => Color::new(0, 0, u16::MAX),
            NodeType::System => Color::new(u16::MAX, 0, u16::MAX),
        }
    }
}
