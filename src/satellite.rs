// This is a module that encapsulates the state and the logic to render a satellite using the yew framework.

use crate::math::{self, Vector2D};
use crate::settings::Settings;
use crate::simulation::SIZE;
use rand::prelude::*;
use yew::{html, Html};

use gloo::console::log;
use wasm_bindgen::JsValue;

// Gravitational constant Earth
const GM: f64 = (3.98601877e11) / 36000000.0;

#[derive(Clone, Debug, PartialEq)]
pub struct Satellite {
    anglular_velocity: f64,
    distance: f64,
    pub position: Vector2D,
    energy: f64,
    hue: f64,
    id: usize,
}

impl Satellite {
    pub fn new_random(_settings: &Settings, id: usize) -> Self {
        let mut rng = rand::thread_rng();

        // choose a random number from 1 to 3 to determine orbit
        let orbit = rng.gen_range(1..4);

        // use the orbit to generate a random radious following a gaussian distribution
        let distance = (f64::from(match orbit {
            1 => rng.gen_range(500..1200),
            2 => rng.gen_range(5000..20000),
            3 => 36000,
            _ => panic!("Invalid orbit value"),
        }) / 45000.0)
            * SIZE.y;

        // Generate random starting angle around the earth
        let angle = rng.gen::<f64>() * math::TAU;

        // calculate angular velocity using radius
        let angular_velocity = GM / distance.powi(3);

        Self {
            anglular_velocity: angular_velocity,
            distance: distance,
            position: Vector2D::from_polar(angle, distance),
            energy: 0.0,
            hue: rng.gen::<f64>() * 360.0,
            id: id,
        }
    }

    pub fn update(&mut self, _settings: &Settings) {
        // update position based on angular velocity
        self.position = Vector2D::from_polar(
            self.position.angle() + self.anglular_velocity,
            self.distance,
        );
    }

    pub fn render(&self) -> Html {
        // Render satellite as a circle using svg and internal hue
        let color = format!("hsl({:.3}rad, 100%, 50%)", self.hue);
        let x = format!("{:.3}", self.position.x + SIZE.x / 2.0);
        let y = format!("{:.3}", self.position.y + SIZE.y / 2.0);
        let id = self.id;

        html! {
            <g>
                <circle cx={x} cy={y} r="4" fill={color} onclick={move |_|{ log!(JsValue::from(format!("Hello from {}", id)))}}/>
            </g>
        }
    }
}
