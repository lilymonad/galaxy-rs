#[derive(Debug, Clone, Copy)]
pub struct Point {
    pub x:f64,
    pub y:f64,
}

impl Point {
    pub fn new(x:f64, y:f64) -> Self {
        Point {
            x,
            y,
        }
    }
    pub fn polar(r:f64, theta:f64) -> Self {
        Point::new(theta.cos(), theta.sin()) * r
    }

    /// dot product with another point
    pub fn dot(self, rhs:Point) -> f64 {
        self.x * rhs.x + self.y * rhs.y
    }

    /// Make the vector keep its direction, but have a distance of 1.0
    pub fn normalize(self) -> Self {
        self / self.length()
    }

    pub fn length(self) -> f64 {
        (self * self).sqrt()
    }

    /// Compute a vector normal to self using this rule:
    /// 
    /// let v = (a, b)  // our vector
    /// let u = (-b, a) // its normal
    ///
    /// These vectors verify the equality : v.u = 0
    /// And every two vector v and u verifying this are perpendicular
    pub fn minusb_a(self) -> Self {
        Point { x: -self.y, y: self.x }
    }

    pub fn with_data<T>(self, data:T) -> DataPoint<T> {
        DataPoint::new(self.x, self.y, data)
    }
}


impl std::ops::Mul<f64> for Point {
    type Output = Point;
    fn mul(self, rhs:f64) -> Self::Output {
        Point {
            x: self.x * rhs,
            y: self.y * rhs,
        }
    }
}

impl std::ops::Div<f64> for Point {
    type Output = Point;
    fn div(self, rhs:f64) -> Self::Output {
        Point {
            x: self.x / rhs,
            y: self.y / rhs,
        }
    }
}

impl std::ops::Add<Point> for Point {
    type Output = Point;

    fn add(self, rhs:Point) -> Self::Output {
        Point {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl std::ops::Sub<Point> for Point {
    type Output = Point;
    fn sub(self, rhs:Point) -> Self::Output {
        Point {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl std::ops::Mul<Point> for Point {
    type Output = f64;
    fn mul(self, rhs:Point) -> Self::Output {
        self.dot(rhs)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct DataPoint<T> {
    pub point: Point,
    pub data: T,
}

impl<T> DataPoint<T> {
    pub fn new(x:f64, y:f64, data:T) -> Self {
        Self::from_point(Point::new(x, y), data)
    }

    pub fn polar(r:f64, theta:f64, data:T) -> Self {
        Self::from_point(Point::polar(r, theta), data)
    }

    pub fn from_point(point:Point, data:T) -> Self {
        Self {
            point,
            data,
        }
    }

    pub fn map<F, U>(self, f:F) -> DataPoint<U>
        where F : FnOnce(T) -> U,
    {
        DataPoint {
            point: self.point,
            data: f(self.data),
        }
    }
}


impl<T> std::ops::Deref for DataPoint<T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.data
    }
}

impl<T> std::ops::DerefMut for DataPoint<T> {
    fn deref_mut(&mut self) -> &mut T {
        &mut self.data
    }
}
