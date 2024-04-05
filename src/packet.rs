// This will encapsulate data pertaining to a packet traveling though the network
use crate::satellite::Satellite;
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

    pub fn render_path(&self, satellites: &Vec<Satellite>) -> Html {
        // Render current path as lines from source to destination using SVG polyline
        let mut points = String::new();

        for hop in &self.path {
            let from = satellites.get(hop.from).unwrap().position;
            let to = satellites.get(hop.to).unwrap().position;

            points.push_str(&format!("{},{} ", from.x, from.y));
            points.push_str(&format!("{},{} ", to.x, to.y));
        }

        html! {
            <polyline points={points} stroke="gray" fill="none" />
        }
    }
}

