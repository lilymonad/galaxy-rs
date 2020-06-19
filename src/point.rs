use las::{Color, Point as LPoint};

#[derive(Debug, Clone, Copy)]
pub struct Point {
    pub x:f64,
    pub y:f64,
}

impl Point {
    pub fn polar(module:f64, radian:f64) -> Self {
        (Point {
            x: radian.cos(),
            y: radian.sin(),
        }) * module
    }

    /// dot product with another point
    pub fn dot(self, rhs:Point) -> f64 {
        self.x * rhs.x + self.y * rhs.y
    }

    /// Make the vector keep its direction, but have a distance of 1.0
    pub fn normalize(self) -> Self {
        self / self.dot(self).sqrt()
    }

    /// Compute a vector normal to self using this rule:
    /// 
    /// let v = (a, b)  // our vector
    /// let u = (-b, a) // its normal
    ///
    /// These vectors verify the equality : v.u = 0
    /// And every two vector v and u verifying this are perpendicular
    pub fn minusb_a(self) -> Self {
        Point { x: -self.y, y: self.x, ..self }
    }

    pub fn with_color(self, color:Color) -> LPoint {
        LPoint {
            color: Some(color),
            ..self.into()
        }
    }
}

impl Into<LPoint> for Point {
    fn into(self) -> LPoint {
        LPoint {
            x: self.x,
            y: self.y,
            z: 0f64,
            ..Default::default()
        }
    }
}

impl From<LPoint> for Point {
    fn from(rhs:LPoint) -> Point {
        Point {
            x: rhs.x,
            y: rhs.y,
        }
    }
}

impl std::ops::Mul<f64> for Point {
    type Output = Point;
    fn mul(self, rhs:f64) -> Point {
        Point {
            x: self.x * rhs,
            y: self.y * rhs,
        }
    }
}

impl std::ops::Div<f64> for Point {
    type Output = Point;
    fn div(self, rhs:f64) -> Point {
        Point {
            x: self.x / rhs,
            y: self.y / rhs,
        }
    }
}

impl std::ops::Add<Point> for Point {
    type Output = Point;

    fn add(self, rhs:Point) -> Point {
        Point {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl std::ops::Sub<Point> for Point {
    type Output = Point;
    fn sub(self, rhs:Point) -> Point {
        Point {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}
