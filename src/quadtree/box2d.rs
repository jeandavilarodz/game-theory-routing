use super::{types::Point};

use yew::{html, Html};




#[derive(Clone, Debug)]
pub struct Box2d
{
    pub(crate) top_left: Point<f64>,
    pub(crate) btm_right: Point<f64>,
}

impl Box2d
{
    pub fn new(top_left: Point<f64>, btm_right: Point<f64>) -> Self {
        Box2d {
            top_left,
            btm_right,
        }
    }

    pub fn contains(&self, point: &Point<f64>) -> bool {
        let top_left = &self.top_left;
        let btm_right = &self.btm_right;

        (point.x >= top_left.x && point.x <= btm_right.x)
            && (point.y >= btm_right.y && point.y <= top_left.y)
    }

    pub fn intersects_box(&self, other: &Box2d) -> bool {
        let top_left = &self.top_left;
        let btm_right = &self.btm_right;

        (btm_right.x >= other.top_left.x)
            && (top_left.x <= other.btm_right.x)
            && (btm_right.y <= other.top_left.y)
            && (top_left.y >= other.btm_right.y)
    }

    /// Subdivide the current box into 4 new boxes north-west, north-east, south-east, south-west
    pub(crate) fn subdivide(&self) -> Vec<Box2d> {
        let top_left = &self.top_left;
        let btm_right = &self.btm_right;
        let x_midpoint = (btm_right.x + top_left.x) / 2.0;
        let y_midpoint = (btm_right.y + top_left.y) / 2.0;
        vec![
            // North-west
            Box2d::new(top_left.clone(), Point::new(x_midpoint, y_midpoint)),
            // North-east
            Box2d::new(
                Point::new(x_midpoint, top_left.y),
                Point::new(btm_right.x, y_midpoint),
            ),
            // South-east
            Box2d::new(Point::new(x_midpoint, y_midpoint), btm_right.clone()),
            // South-west
            Box2d::new(
                Point::new(top_left.x, y_midpoint),
                Point::new(x_midpoint, btm_right.y),
            ),
        ]
    }

    pub fn render(&self) -> Html {
        let height = (self.top_left.y - self.btm_right.y).abs().to_string();
        let width = (self.btm_right.x - self.top_left.x).abs().to_string();
        let x = self.top_left.x.to_string();
        let y = self.btm_right.y.to_string();

        html! { <rect x={x} y={y} width={width} height={height} fill="none" stroke="gray" stroke-width="1" /> }
    }
}
