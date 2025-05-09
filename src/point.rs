/// A 3D point.
#[derive(Debug, Clone, Copy)]
pub struct Point {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}
use std::ops::{Add, Mul, Sub};

impl Point {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Point { x, y, z }
    }
    pub fn dot(self, other: Point) -> f32 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }

    pub fn magnitude(&self) -> f32 {
        (self.x * self.x + self.y * self.y + self.z * self.z).sqrt()
    }
    pub fn clamp_length(&self, max: f32) -> Point {
        let length = self.magnitude();
        if length > max {
            return self * (max / length);
        }
        *self
    }

    pub fn normalize(self) -> Point {
        let magnitude = (self.x * self.x + self.y * self.y + self.z * self.z).sqrt();
        Point {
            x: self.x / magnitude,
            y: self.y / magnitude,
            z: self.z / magnitude,
        }
    }

    pub fn cross(self, other: Point) -> Point {
        Point {
            x: self.y * other.z - self.z * other.y,
            y: self.z * other.x - self.x * other.z,
            z: self.x * other.y - self.y * other.x,
        }
    }
}

impl Mul<f32> for &Point {
    type Output = Point;
    fn mul(self, scalar: f32) -> Point {
        Point {
            x: self.x * scalar,
            y: self.y * scalar,
            z: self.z * scalar,
        }
    }
}

impl Mul<f32> for Point {
    type Output = Point;
    fn mul(self, scalar: f32) -> Point {
        Point {
            x: self.x * scalar,
            y: self.y * scalar,
            z: self.z * scalar,
        }
    }
}

impl Add for Point {
    type Output = Point;
    fn add(self, other: Point) -> Point {
        Point {
            x: self.x + other.x,
            y: self.y + other.y,
            z: self.z + other.z,
        }
    }
}

impl Sub for Point {
    type Output = Point;
    fn sub(self, other: Point) -> Point {
        Point {
            x: self.x - other.x,
            y: self.y - other.y,
            z: self.z - other.z,
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Point2D {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl PartialEq for Point2D {
    fn eq(&self, other: &Self) -> bool {
        self.x == other.x && self.y == other.y && (self.z - other.z).abs() < f32::EPSILON
    }
}

pub fn normalize(vec: Point) -> Point {
    let magnitude = (vec.x * vec.x + vec.y * vec.y + vec.z * vec.z).sqrt();
    Point {
        x: vec.x / magnitude,
        y: vec.y / magnitude,
        z: vec.z / magnitude,
    }
}
pub fn cross_product(a: Point, b: Point) -> Point {
    Point {
        x: a.y * b.z - a.z * b.y,
        y: a.z * b.x - a.x * b.z,
        z: a.x * b.y - a.y * b.x,
    }
}
pub fn dot_product(a: Point, b: Point) -> f32 {
    a.x * b.x + a.y * b.y + a.z * b.z
}

#[inline(always)]
pub fn snap_to_pixel(point: Point2D) -> Point2D {
    Point2D {
        x: point.x.round(),
        y: point.y.round(),
        z: point.z,
    }
}
