use std::f32::consts::{FRAC_PI_3, PI};
use std::iter::Sum;
use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign};

// at the time of writing the TAU constant is still unstable
pub const TAU: f32 = 2.0 * PI;
pub const FRAC_TAU_3: f32 = 2.0 * FRAC_PI_3;

/// Get the smaller signed angle from `source` to `target`.
/// The result is in the range `[-PI, PI)`.
pub fn smallest_angle_between(source: f32, target: f32) -> f32 {
    let d = target - source;
    (d + PI).rem_euclid(TAU) - PI
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Vector2D {
    pub x: f32,
    pub y: f32,
}
impl Vector2D {
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    pub fn from_polar(angle: f32, radius: f32) -> Self {
        let (sin, cos) = angle.sin_cos();
        Self::new(radius * cos, radius * sin)
    }

    pub fn magnitude_squared(self) -> f32 {
        self.x * self.x + self.y * self.y
    }

    pub fn magnitude(self) -> f32 {
        self.magnitude_squared().sqrt()
    }

    pub fn clamp_magnitude(self, max: f32) -> Self {
        let mag = self.magnitude();
        if mag > max {
            self / mag * max
        } else {
            self
        }
    }

    /// Positive angles measured counter-clockwise from positive x axis.
    pub fn angle(self) -> f32 {
        self.y.atan2(self.x)
    }
}

impl Neg for Vector2D {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self::new(-self.x, -self.y)
    }
}

impl AddAssign for Vector2D {
    fn add_assign(&mut self, other: Self) {
        self.x += other.x;
        self.y += other.y;
    }
}
impl Add for Vector2D {
    type Output = Self;

    fn add(mut self, rhs: Self) -> Self::Output {
        self += rhs;
        self
    }
}

impl SubAssign for Vector2D {
    fn sub_assign(&mut self, other: Self) {
        self.x -= other.x;
        self.y -= other.y;
    }
}
impl Sub for Vector2D {
    type Output = Self;

    fn sub(mut self, rhs: Self) -> Self::Output {
        self -= rhs;
        self
    }
}

impl MulAssign<f32> for Vector2D {
    fn mul_assign(&mut self, scalar: f32) {
        self.x *= scalar;
        self.y *= scalar;
    }
}
impl Mul<f32> for Vector2D {
    type Output = Self;

    fn mul(mut self, rhs: f32) -> Self::Output {
        self *= rhs;
        self
    }
}

impl DivAssign<f32> for Vector2D {
    fn div_assign(&mut self, scalar: f32) {
        self.x /= scalar;
        self.y /= scalar;
    }
}
impl Div<f32> for Vector2D {
    type Output = Self;

    fn div(mut self, rhs: f32) -> Self::Output {
        self /= rhs;
        self
    }
}

impl Sum for Vector2D {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(Self::default(), |sum, v| sum + v)
    }
}

pub trait WeightedMean<T = Self>: Sized {
    fn weighted_mean(it: impl Iterator<Item = (T, f32)>) -> Option<Self>;
}

impl<T> WeightedMean for T
where
    T: AddAssign + Mul<f32, Output = T> + Div<f32, Output = T> + Copy + Default,
{
    fn weighted_mean(it: impl Iterator<Item = (T, f32)>) -> Option<T> {
        let (sum, total_weight) = it.fold(
            (T::default(), 0.0),
            |(mut sum, total_weight), (value, weight)| {
                sum += value * weight;
                (sum, total_weight + weight)
            },
        );
        if total_weight.is_normal() {
            Some(sum / total_weight)
        } else {
            None
        }
    }
}

pub trait Mean<T = Self>: Sized {
    fn mean(it: impl Iterator<Item = T>) -> Option<Self>;
}

impl<T> Mean for T
where
    T: AddAssign + Sub<Output = T> + Div<f32, Output = T> + Copy + Default,
{
    fn mean(it: impl Iterator<Item = T>) -> Option<T> {
        let (avg, count) = it.fold((T::default(), 0.0), |(mut avg, mut count), value| {
            count += 1.0;
            avg += (value - avg) / count;
            (avg, count)
        });
        if count.is_normal() {
            Some(avg)
        } else {
            None
        }
    }
}
