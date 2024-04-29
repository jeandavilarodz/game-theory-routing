// This is a module that encapsulates the state and the logic to render a satellite using the yew framework.

use crate::cluster::Cluster;
use crate::math::{self, Vector2D};
use crate::packet::{Packet, PacketSource};
use crate::settings::Settings;
use crate::simulation::SIZE;
use rand::prelude::*;
use yew::{html, Callback, Html};


use gloo::console::log;
use wasm_bindgen::JsValue;

// Gravitational constant Earth
const STD_GRAV_PARAM: f32 = 3.98601877e11;

pub const MAX_DISTANCE: f32 = 40000.0;


pub struct SatelliteProperties {
    id: usize,
    angular_velocity: f32,
    distance: f32,
    selected:bool,
    hue: f32,
}

#[derive(Clone, PartialEq)]
pub struct SatellitePosition {
    position: Vector2D,
    angle: f32,
}

#[derive(Clone, Debug)]
pub struct SatelliteComms {
    packets: Vec<Packet>,
    source: PacketSource,
}

pub struct SatelliteEnergy {
    id: usize,
    in_game: bool,
    cost: f32,
    gain: f32,
    energy: f32,
    max_energy: f32,
    prob_entering: f32,
}

impl SatelliteProperties {
    pub fn new_random(id: usize) -> Self {
        let mut rng = rand::thread_rng();

        // choose a random number from 1 to 3 to determine orbit
        let orbit = rng.gen_range(2..4);

        // use the orbit to generate a random radious following a gaussian distribution
        let distance = match orbit {
            1 => rng.gen_range(500..1200) as f32,
            2 => rng.gen_range(5000..20000) as f32,
            3 => 36000.0f32,
            _ => panic!("Invalid orbit value"),
        };

        // calculate angular velocity using radius
        let angular_velocity = (STD_GRAV_PARAM / distance.powi(3)).sqrt();

        let hue = rng.gen::<f32>() * 360.0;

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

    pub fn distance(&self) -> f32 {
        self.distance
    } 

    pub fn set_selected(&mut self, selected: bool) {
        self.selected = selected;
    }

    pub fn color(&self) -> f32 {
        self.hue
    }

    pub fn set_color(&mut self, hue: f32) {
        self.hue = hue;
    }
}

impl SatellitePosition {
    pub fn new_random(sat: &SatelliteProperties) -> Self {
        let mut rng = rand::thread_rng();

        // Generate starting angle
        let angle = rng.gen::<f32>() * math::TAU;

        let mut position = Vector2D::from_polar(
            angle,
            (sat.distance / MAX_DISTANCE) * (SIZE.y / 2.0),
        );
        
        position.x += SIZE.x / 2.0;
        position.y += SIZE.y / 2.0;

        Self {
            position,
            angle,
        }
    }

    pub fn update(&mut self, sat: &SatelliteProperties, settings: &Settings) {
        // Calculate new position based on angular velocity
        self.angle += sat.angular_velocity * (settings.tick_interval_ms as f32 / 1000.0);
        let radius = (sat.distance / MAX_DISTANCE) * (SIZE.y / 2.0);
        self.position = Vector2D::from_polar(self.angle, radius);

        // Offset screen position to orbit center of screen
        self.position.x += SIZE.x / 2.0;
        self.position.y += SIZE.y / 2.0;
    }

    pub fn screen_position(&self) -> Vector2D {
        self.position
    }

    pub fn distance_from_earth(&self) -> f32 {
        let x = self.position.x - SIZE.x / 2.0;
        let y = self.position.y - SIZE.y / 2.0;

        (x * x + y * y).sqrt()
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
        if packet.in_path(self.source.id()) {
            return;
        }

        if packet.ttl() == 0 {
            return;
        }

        let mut packet = packet;
        packet.decrement_ttl();
        let timestamp = chrono::Local::now().timestamp() as u64;
        packet.add_hop(self.source.id(), timestamp);

        let debug = format!("Added on {}: {:?}", self.id(), packet);
        log!(JsValue::from(&debug));

        self.packets.push(packet);
    }

    pub fn update(&mut self,
                       sat: &SatelliteProperties,
                       pos: &SatellitePosition,
                       neigh_pos: Vec<&SatellitePosition>,
                       neigh_comms: Vec<&mut SatelliteComms>,
                       _settings: &Settings)
    {
        let mut rng = rand::thread_rng();

        // Create a new packet
        let mut packet = if rng.gen_bool(0.1) {
            Some(self.source.next())
        } else {
            None
        };

        // Make the assumption that satellites can communicate with a max distance from Earth to the vehicle
        let comms_distance = (sat.distance / MAX_DISTANCE) * (SIZE.y / 2.0);
        let mut neigh_comms = neigh_comms;

        // Iterate through all neighbors and send packets if they are within comms distance
        for (neighbor_comms, neighbor_pos) in neigh_comms.iter_mut().zip(neigh_pos.iter()) {

            // Do not send packet to ourselves
            if self.source.id() == neighbor_comms.id() {
                continue;
            }

            let distance = (pos.position - neighbor_pos.position).magnitude();

            if distance < comms_distance {
                if let Some(ref mut packet) = packet {
                    neighbor_comms.add_packet(packet.clone());
                }

                for packet in self.packets.iter().cloned() {
                    neighbor_comms.add_packet(packet);
                }
            }
        }

        // Sent all packets to neighbors
        self.packets.clear();
    }
}

impl SatelliteEnergy {
    pub fn new_random(id: usize, settings: &Settings) -> Self {
        let mut rng = rand::thread_rng();
        let energy = rng.gen::<f32>() * 100.0;

        Self {
            id,
            in_game: false,
            cost: settings.comms_cost,
            gain: settings.energy_gain,
            energy,
            max_energy: settings.max_energy,
            prob_entering: 100.0,
        }
    }

    pub fn prob_entering(&self) -> f32 {
        self.prob_entering
    }

    pub fn update_game(&mut self, cluster: &Cluster) {
        if self.energy < self.cost || self.energy < 0.0 {
            self.in_game = false;
            return;
        }

        if cluster.size() < 2 {
            if self.energy > self.cost {
                self.in_game = true;
            } else {
                self.in_game = false;
            }
            return;
        }

        // Calculate Nash equilibrium probability
        let mut rng = rand::thread_rng();
        let num_neighbors = (cluster.size() - 1) as f32;
        let prob_entering = 1.0 - (1.0 - ((self.energy - self.cost) / (self.energy + self.gain))).powf(1.0/num_neighbors);

        if prob_entering < 0.0 || prob_entering > 1.0 || prob_entering.is_nan() {
            self.in_game = false;
            return;
        }

        // Store probability of entering game
        self.prob_entering = prob_entering;

        // Determine if satellite enters game
        self.in_game = rng.gen_bool(prob_entering as f64);

        #[cfg(debug_assertions)]
        if self.in_game {
            let debug = format!("id: {} -> entering game: {}", self.id, self.energy);
            log!(JsValue::from(&debug));
        } else {
            let debug = format!("id: {} -> not entering game: {}", self.id, self.energy);
            log!(JsValue::from(&debug));
        }
    }

    pub fn update(&mut self, neighbors: Vec<&SatelliteEnergy>) {
        if self.in_game {
            self.energy -= self.cost;

            // Clamp energy to 0
            if self.energy < 0.0 {
                self.energy = 0.0;
            }
            let debug = format!("id: {} -> consumed {}", self.id, self.energy);
            log!(JsValue::from(&debug));
            return;
        }

        // Calculate utility based on decisions made by neighbors
        let neighbors_in_game = neighbors.iter().fold(0, |acc, n| acc + (n.in_game as u32));

        // If no neighbors are in the game, get no payoff
        if neighbors_in_game == 0 {
            return;
        }
        else {
            // Recharge energy
            self.energy += self.gain;
            
            // Clamp energy to max energy
            if self.energy > self.max_energy {
                self.energy = self.max_energy;
            }

            let debug = format!("id: {} -> recharge {}", self.id, self.energy);
            log!(JsValue::from(&debug));
        }
    }

    pub fn energy(&self) -> f32 {
        self.energy
    }
}

pub fn render(sat: &SatelliteProperties, position: &SatellitePosition, game: &SatelliteEnergy, onclick_cb: Callback<usize>) -> Html {
    let color = format!("hsl({:.3}, 100%, 50%)", sat.hue);
    let x = format!("{:.3}", position.position.x);
    let y = format!("{:.3}", position.position.y);
    let callback = onclick_cb.clone();
    let id = sat.id;
    let opacity = if game.in_game { "1.0" } else { "0.5" };

    html! {
        // Create a circle when clicked it will cause a callback to update self.selected
        <circle cx={x} cy={y} r="5" fill={color} opacity={opacity} onclick={move |_|{callback.emit(id)}}>
        if sat.selected {
            <animate attributeName="r" values="5; 15; 5" dur="1s" repeatCount="indefinite" />
        }
        </circle>
    }
}
