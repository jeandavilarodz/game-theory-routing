// This is a module that encapsulates the state and the logic to render a satellite using the yew framework.

use std::time;

use crate::math::{self, Vector2D};
use crate::packet::{Packet, PacketSource};
use crate::settings::Settings;
use crate::simulation::SIZE;
use rand::prelude::*;
use yew::{html, Callback, Html};

// Gravitational constant Earth
const STD_GRAV_PARAM: f64 = 3.98601877e11;

const MAX_DISTANCE: f64 = 40000.0;


pub struct SatelliteProperties {
    id: usize,
    angular_velocity: f64,
    distance: f64,
    selected:bool,
    hue: f64,
}

#[derive(Clone, PartialEq)]
pub struct SatellitePosition {
    position: Vector2D,
    angle: f64,
}

pub struct SatelliteComms {
    packets: Vec<Packet>,
    source: PacketSource,
}

pub struct SatelliteEnergy {
    energy: u32,
}

impl SatelliteProperties {
    pub fn new_random(id: usize) -> Self {
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
        let angular_velocity = (STD_GRAV_PARAM / distance.powi(3)).sqrt();

        let hue = rng.gen::<f64>() * 360.0;

        Self {
            id,
            angular_velocity,
            distance,
            selected: false,
            hue,
        }
    }

    pub fn id(&self) -> usize {
        self.id
    }

    pub fn set_selected(&mut self, selected: bool) {
        self.selected = selected;
    }
}

impl SatellitePosition {
    pub fn new_random(sat: &SatelliteProperties) -> Self {
        let mut rng = rand::thread_rng();

        // Generate starting angle
        let angle = rng.gen::<f64>() * math::TAU;

        let position = Vector2D::from_polar(
            angle,
            (sat.distance / MAX_DISTANCE) * (SIZE.y / 2.0),
        );

        Self {
            position,
            angle,
        }
    }

    pub fn update(&mut self, sat: &SatelliteProperties, settings: &Settings) {
        // Calculate new position based on angular velocity
        self.angle += sat.angular_velocity * (settings.tick_interval_ms as f64 / 1000.0);
        let radius = (sat.distance / MAX_DISTANCE) * (SIZE.y / 2.0);
        self.position = Vector2D::from_polar(self.angle, radius);

        // Offset screen position to orbit center of screen
        self.position.x += SIZE.x / 2.0;
        self.position.y += SIZE.y / 2.0;
    }

    pub fn screen_position(&self) -> Vector2D {
        self.position
    }
}

impl SatelliteComms {
    pub fn new(id: usize) -> Self {
        Self {
            packets: Vec::new(),
            source: PacketSource::new(id),
        }
    }

    pub fn id(&self) -> usize {
        self.source.id()
    }

    pub fn packets(&self) -> &Vec<Packet> {
        &self.packets
    }

    pub fn add_packet(&mut self, packet: Packet) {
        self.packets.push(packet);
    }

    pub fn send_packet(&mut self,
                       sat: &SatellitePosition,
                       neighbors: &mut Vec<(SatelliteComms, SatellitePosition)>,
                       _settings: &Settings)
    {
        let mut rng = rand::thread_rng();

        // Create a new packet
        let mut packet = self.source.next();

        // Add the current satellite to the path
        let timestamp = time::SystemTime::now().elapsed().unwrap().as_millis() as u64;
        packet.add_hop(self.source.id(), timestamp);

        let comms_distance = f64::MAX;

        for (neighbor_comms, neighbor_pos) in neighbors.iter_mut() {
            if self.source.id() == neighbor_comms.id() {
                continue;
            }

            let distance = (sat.position - neighbor_pos.position).magnitude();

            if distance < comms_distance {
                neighbor_comms.add_packet(packet.clone());

                for packet in self.packets.iter().cloned() {
                    neighbor_comms.add_packet(packet);
                }
            }
        }
    }
}

impl SatelliteEnergy {
    pub fn new_random() -> Self {
        let mut rng = rand::thread_rng();
        let energy = (rng.gen::<f64>() * 100.0) as u32;

        Self {
            energy,
        }
    }

    pub fn update(&mut self, _settings: &Settings) {
        todo!()
    }

    pub fn energy(&self) -> u32 {
        self.energy
    }
}

pub fn render(sat: &SatelliteProperties, position: &SatellitePosition, onclick_cb: Callback<usize>) -> Html {
    let color = format!("hsl({:.3}rad, 100%, 50%)", sat.hue);
    let x = format!("{:.3}", position.position.x);
    let y = format!("{:.3}", position.position.y);
    let callback = onclick_cb.clone();
    let id = sat.id;

    html! {
        // Create a circle when clicked it will cause a callback to update self.selected
        <circle cx={x} cy={y} r="5" fill={color} onclick={move |_|{callback.emit(id)}}>
        if sat.selected {
            <animate attributeName="r" values="5; 15; 5" dur="1s" repeatCount="indefinite" />
        }
        </circle>
    }
}
