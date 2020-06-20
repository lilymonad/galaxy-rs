#![cfg(feature = "lidar")]
use las_lib::{Color, Point as LPoint};
use crate::point::*;
use crate::galaxy::NodeType;

impl Into<LPoint> for DataPoint<Color> {
    fn into(self) -> LPoint {
        LPoint {
            x: self.point.x,
            y: self.point.y,
            z: 0f64,
            color: Some(self.data),
            ..Default::default()
        }
    }
}

impl From<LPoint> for DataPoint<Color> {
    fn from(rhs:LPoint) -> Self {
        Self::new(rhs.x, rhs.y, rhs.color.unwrap_or_default())
    }
}

impl DataPoint<Color> {
    pub fn to_lidar_with_z(self, z:f64) -> LPoint {
        LPoint {
            z,
            ..self.into()
        }
    }
}

impl Into<Color> for NodeType {
    fn into(self) -> Color {
        match self {
            NodeType::Root   => Color::new(u16::MAX, 0, 0),
            NodeType::Arm    => Color::new(u16::MAX, 0, 0),
            NodeType::Ext    => Color::new(0, u16::MAX, 0),
            NodeType::Loner  => Color::new(0, 0, u16::MAX),
            NodeType::System => Color::new(u16::MAX, 0, u16::MAX),
        }
    }
}
