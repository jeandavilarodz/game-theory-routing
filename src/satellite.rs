// This is a module that encapsulates the state and the logic to render a satellite using the yew framework.

use crate::math::{self, Vector2D};
use crate::packet::Packet;
use crate::settings::Settings;
use crate::simulation::SIZE;
use rand::prelude::*;
use yew::{html, Callback, Component, Context, Html, Properties};

// Gravitational constant Earth
const standard_gravitational_parameter: f64 = 3.98601877e11;

#[derive(Clone, Debug, PartialEq)]
pub struct Satellite {
    angular_velocity: f64,
    distance: f64,
    angle: f64,
    pub screen_position: Vector2D,
    pub energy: u32,
    hue: f64,
    pub id: usize,
    pub packets: Vec<Packet>,
}

impl Satellite {
    pub fn new_random(settings: &Settings, id: usize) -> Self {
        let mut rng = rand::thread_rng();

        // choose a random number from 1 to 3 to determine orbit
        let orbit = rng.gen_range(2..4);

        // use the orbit to generate a random radious following a gaussian distribution
        let distance = f64::from(match orbit {
            1 => rng.gen_range(500..1200),
            2 => rng.gen_range(5000..20000),
            3 => 36000,
            _ => panic!("Invalid orbit value"),
        });

        // calculate angular velocity using radius
        let angular_velocity = (standard_gravitational_parameter / distance.powi(3)).sqrt()
            * (settings.tick_interval_ms as f64)
            / 1000.0;

        // Generate random starting angle
        let angle = rng.gen::<f64>() * math::TAU;

        // Generate starting position based on angle and distance
        let mut screen_position = Vector2D::from_polar(
            angle,
            (distance / 36000.0) * SIZE.y / 2.0,
        );
        screen_position.x += SIZE.x / 2.0;
        screen_position.y += SIZE.y / 2.0;

        // Initial energy
        let energy = (rng.gen::<f64>() * 100.0) as u32;

        // Statellite color
        let hue = rng.gen::<f64>() * 360.0;
        

        Self {
            angular_velocity,
            distance,
            angle,
            screen_position,
            energy,
            hue,
            id,
            packets: Vec::new(),
        }
    }

    pub fn update(&mut self, _settings: &Settings) {
        self.angle += self.angular_velocity;
        // update position based on angular velocity
        self.screen_position =
            Vector2D::from_polar(self.angle, (self.distance / 36000.0) * SIZE.y / 2.0);
        self.screen_position.x += SIZE.x / 2.0;
        self.screen_position.y += SIZE.y / 2.0;
    }

    // TODO: Implement game theoretic approach to calculate energy
}

#[derive(Debug, PartialEq, Properties)]
pub struct SatelliteProps {
    pub info: Satellite,
    pub on_clicked: Callback<usize>,
    #[prop_or(false)]
    pub selected: bool,
}

pub struct SatelliteComponent {
    selected: bool,
    on_clicked: Callback<usize>,
    sat: Satellite,
}

impl Component for SatelliteComponent {
    type Message = ();
    type Properties = SatelliteProps;

    fn create(ctx: &Context<Self>) -> Self {
        Self {
            selected: ctx.props().selected,
            sat: ctx.props().info.clone(),
            on_clicked: ctx.props().on_clicked.clone(),
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, _msg: Self::Message) -> bool {
        unimplemented!()
    }

    fn changed(&mut self, ctx: &Context<Self>, _old_props: &Self::Properties) -> bool {
        self.selected = ctx.props().selected;
        self.sat = ctx.props().info.clone();
        true
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        let color = format!("hsl({:.3}rad, 100%, 50%)", self.sat.hue);
        let x = format!("{:.3}", self.sat.screen_position.x);
        let y = format!("{:.3}", self.sat.screen_position.y);
        let callback = self.on_clicked.clone();
        let id = self.sat.id;

        html! {
            // Create a circle when clicked it will cause a callback to update self.selected
            <circle cx={x} cy={y} r="5" fill={color} onclick={move |_|{callback.emit(id)}}>
            if self.selected {
                <animate attributeName="r" values="5; 15; 5" dur="1s" repeatCount="indefinite" />
            }
            </circle>
        }
    }
}
