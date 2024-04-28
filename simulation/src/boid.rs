use std::fmt::Write;

use rand::Rng;
use yew::{html, Html};

use crate::math::{self, Mean, Vector2D, WeightedMean};
use crate::quadtree::box2d::Box2d;
use crate::quadtree::{quadtree::QuadTree, types::*};
use crate::settings::Settings;
use crate::simulation::SIZE;


#[derive(Clone, Debug, PartialEq)]
pub struct Boid {
    position: Vector2D,
    velocity: Vector2D,
    radius: f32,
    hue: f32,
}

impl Boid {
    pub fn new_random(settings: &Settings) -> Self {
        let mut rng = rand::thread_rng();

        let max_radius = settings.min_distance / 2.0;
        let min_radius = max_radius / 6.0;
        // by using the third power large boids become rarer
        let radius = min_radius + rng.gen::<f32>().powi(3) * (max_radius - min_radius);

        Self {
            position: Vector2D::new(rng.gen::<f32>() * SIZE.x, rng.gen::<f32>() * SIZE.y),
            velocity: Vector2D::from_polar(rng.gen::<f32>() * math::TAU, settings.max_speed),
            radius,
            hue: rng.gen::<f32>() * math::TAU,
        }
    }

    fn coherence(&self, boids: &Vec<&Boid>, factor: f32) -> Vector2D {
        Vector2D::weighted_mean(
            boids
                .iter()
                .map(|other| (other.position, other.radius * other.radius)),
        )
        .map(|mean| (mean - self.position) * factor)
        .unwrap_or_default()
    }

    fn separation(&self, boids: &Vec<&Boid>, settings: &Settings) -> Vector2D {
        let accel = boids
            .iter()
            .filter_map(|other| {
                let offset = other.position - self.position;
                if offset.magnitude() > settings.min_distance {
                    None
                } else {
                    Some(-offset)
                }
            })
            .sum::<Vector2D>();
        accel * settings.separation_factor
    }

    fn alignment(&self, boids: &Vec<&Boid>, factor: f32) -> Vector2D {
        Vector2D::mean(boids.iter().map(|other| other.velocity))
            .map(|mean| (mean - self.velocity) * factor)
            .unwrap_or_default()
    }

    fn adapt_color(&self, boids: &Vec<&Boid>, factor: f32) -> f32 {
        let mean = f32::mean(boids.iter().filter_map(|other| {
            if other.radius > self.radius {
                Some(math::smallest_angle_between(self.hue, other.hue))
            } else {
                None
            }
        }));
        if let Some(avg_hue_offset) = mean {
            return self.hue + (avg_hue_offset * factor);
        } else {
            return self.hue;
        }
    }

    fn update_velocity(&self, settings: &Settings, boids: &Vec<&Boid>) -> Vector2D {
        // Calculate new velocity from internal forces
        let v = self.velocity
            + self.coherence(boids, settings.cohesion_factor)
            + self.separation(boids, settings)
            + self.alignment(boids, settings.alignment_factor);

        // Cap velocity
        v.clamp_magnitude(settings.max_speed)
    }

    fn update(&self, settings: &Settings, boids: Vec<&Boid>) -> Self {
        let mut ret = self.clone();
        ret.hue = self.adapt_color(&boids, settings.color_adapt_factor);

        // Update velocity and make sure boid is within bounds
        ret.velocity = self.update_velocity(settings, &boids);
        ret.velocity = keep_in_bounds(settings, ret.velocity, ret.position);

        ret.position += ret.velocity;

        ret
    }

    pub fn update_all(settings: &Settings, boids: &mut Vec<Boid>) -> (Vec<Self>, QuadTree<usize>) {
        let mut ret = Vec::with_capacity(boids.len());

        // Create quadtree
        let mut qtree = QuadTree::new(
            Box2d::new(
                Point::new(0.0, SIZE.y),
                Point::new(SIZE.x, 0.0)
            ),
            16,
        );

        // Build quadtree for efficient Boid search
        for (id, boid) in boids.iter().enumerate() {
            qtree.insert(Point::new(boid.position.x, boid.position.y), id);
        }

        let visible_range = settings.visible_range;

        for (curr_id, boid) in boids.iter().cloned().enumerate() {
            let neighbors = qtree
                .query_range(Box2d::new(
                    Point::new(boid.position.x - visible_range, boid.position.y + visible_range),
                    Point::new(boid.position.x + visible_range, boid.position.y - visible_range)
                ))
                .iter()
                .filter_map(|e| {
                    if curr_id != *e.value {
                        boids.get(*e.value)
                    } else {
                        None
                    }
                })
                .collect::<Vec<&Boid>>();

            ret.push(boid.update(settings, neighbors));
        }

        (ret, qtree)
    }

    pub fn render(&self) -> Html {
        let color = format!("hsl({:.3}rad, 100%, 50%)", self.hue);

        let mut points = String::new();
        for offset in iter_shape_points(self.radius, self.velocity.angle()) {
            let Vector2D { x, y } = self.position + offset;

            // Write to string will never fail.
            let _ = write!(points, "{x:.2},{y:.2} ");
        }

        /*
        log!(JsValue::from(&points));
        */

        html! { <polygon {points} fill={color} /> }
    }
}

fn iter_shape_points(radius: f32, rotation: f32) -> impl Iterator<Item = Vector2D> {
    // This is angle and radius pairs, points are:
    //  (2,0), (-sqrt(2)/2, sqrt(2)/2), (-sqrt(2)/2, -sqrt(2)/2)
    const SHAPE: [(f32, f32); 3] = [
        (0. * math::FRAC_TAU_3, 2.0),
        (1. * math::FRAC_TAU_3, 1.0),
        (2. * math::FRAC_TAU_3, 1.0),
    ];
    SHAPE
        .iter()
        .copied()
        .map(move |(angle, radius_mul)| Vector2D::from_polar(angle + rotation, radius_mul * radius))
}

fn keep_in_bounds(settings: &Settings, velocity: Vector2D, position: Vector2D) -> Vector2D{
    let min = SIZE * settings.border_margin;
    let max = SIZE - min;

    let mut v = Vector2D::default();

    let turn_speed = velocity.magnitude() * settings.turn_speed_ratio;
    let pos = position;
    if pos.x < min.x {
        v.x += turn_speed;
    }
    if pos.x > max.x {
        v.x -= turn_speed
    }

    if pos.y < min.y {
        v.y += turn_speed;
    }
    if pos.y > max.y {
        v.y -= turn_speed;
    }

    velocity + v
}
