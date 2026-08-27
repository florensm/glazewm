/// Represents an x-y coordinate.
#[derive(Debug, Clone)]
pub struct Point {
  pub x: i32,
  pub y: i32,
}

impl Point {
  /// Calculates the Euclidean distance between this point and another
  /// point.
  #[must_use]
  pub fn distance_between(&self, other: &Point) -> f32 {
    #[allow(clippy::cast_precision_loss)]
    f32::hypot((self.x - other.x) as f32, (self.y - other.y) as f32)
  }
}
