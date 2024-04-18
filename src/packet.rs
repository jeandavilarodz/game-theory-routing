// This will encapsulate data pertaining to a packet traveling though the network
use crate::satellite::SatellitePosition;
use yew::{html, Html};

#[derive(Clone, Debug, PartialEq)]
pub struct Hop {
    from: usize,
    to: usize,
    time: u64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Packet {
    id: usize,
    source: usize,
    ttl: usize,
    path: Vec<Hop>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct PacketSource {
    id: usize,
    sequence_number: usize,
}

impl Packet {
    pub fn new(id: usize, source: usize, ttl: usize) -> Self {
        Self {
            id,
            source,
            ttl,
            path: Vec::with_capacity(ttl as usize),
        }
    }

    pub fn add_hop(&mut self, node: usize, time: u64) {
        // append a new hop to the path list from previous node to current node
        let last = self.path.last().map_or(self.source, |hop| hop.to);
        self.path.push(Hop { from: last, to: node, time });
    }

    pub fn render_path(&self, satellites: &Vec<SatellitePosition>) -> Html {
        // Render current path as lines from source to destination using SVG polyline
        let mut points = String::new();

        for hop in &self.path {
            let from = satellites.get(hop.from).unwrap().screen_position();
            let to = satellites.get(hop.to).unwrap().screen_position();

            points.push_str(&format!("{},{} ", from.x, from.y));
            points.push_str(&format!("{},{} ", to.x, to.y));
        }

        html! {
            <polyline points={points} stroke="gray" fill="none" />
        }
    }
}

impl PacketSource {
    pub fn new(id: usize) -> Self {
        Self {
            id,
            sequence_number: 0,
        }
    }

    pub fn next(&mut self) -> Packet {
        self.sequence_number += 1;
        Packet::new(self.sequence_number, self.id, 10)
    }

    pub fn id(&self) -> usize {
        self.id
    }
}