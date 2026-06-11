use serde::{Deserialize, Serialize};
use std::ops::{Add, Div, Mul, Sub};

#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Vec3Km {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Vec3KmPerSec {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Vec3Km {
    pub const ZERO: Self = Self::new(0.0, 0.0, 0.0);

    pub const fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }

    pub fn is_finite(self) -> bool {
        self.x.is_finite() && self.y.is_finite() && self.z.is_finite()
    }

    pub fn dot(self, other: Self) -> f64 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }

    pub fn magnitude(self) -> f64 {
        self.dot(self).sqrt()
    }

    pub fn distance(self, other: Self) -> f64 {
        (self - other).magnitude()
    }
}

impl Vec3KmPerSec {
    pub const ZERO: Self = Self::new(0.0, 0.0, 0.0);

    pub const fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }

    pub fn is_finite(self) -> bool {
        self.x.is_finite() && self.y.is_finite() && self.z.is_finite()
    }

    pub fn dot_position(self, other: Vec3Km) -> f64 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }
}

macro_rules! impl_vec_ops {
    ($type:ty) => {
        impl Add for $type {
            type Output = Self;

            fn add(self, rhs: Self) -> Self::Output {
                Self::new(self.x + rhs.x, self.y + rhs.y, self.z + rhs.z)
            }
        }

        impl Sub for $type {
            type Output = Self;

            fn sub(self, rhs: Self) -> Self::Output {
                Self::new(self.x - rhs.x, self.y - rhs.y, self.z - rhs.z)
            }
        }

        impl Mul<f64> for $type {
            type Output = Self;

            fn mul(self, rhs: f64) -> Self::Output {
                Self::new(self.x * rhs, self.y * rhs, self.z * rhs)
            }
        }

        impl Div<f64> for $type {
            type Output = Self;

            fn div(self, rhs: f64) -> Self::Output {
                Self::new(self.x / rhs, self.y / rhs, self.z / rhs)
            }
        }
    };
}

impl_vec_ops!(Vec3Km);
impl_vec_ops!(Vec3KmPerSec);
