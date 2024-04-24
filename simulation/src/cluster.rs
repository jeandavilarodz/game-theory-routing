use crate::satellite::SatellitePosition;
/// This module will encapsulate information for clusters in the
/// network graph. A cluster is a group of nodes that are connected
/// to each other and are not connected to any other nodes outside
/// the cluster.
///
/// The cluster module will contain the following:
/// 1. A cluster head, which is the node that will act as a gateway to Earth for the cluster.
/// 2. cluster members, which are the nodes that are part of the cluster.
/// 3. A cluster ID, which is a unique identifier for the cluster.
/// 4. A cluster size, which is the number of nodes in the cluster.
///

use crate::simulation::SIZE;
use crate::math;

use std::collections::HashMap;
use yew::{html, Html};
use rand::Rng;


pub struct Cluster {
    head: usize,
    members: Vec<usize>,
    size: usize,
    color: f64,
}

impl Cluster {
    pub fn new(head: usize) -> Self {
        let mut rng = rand::thread_rng();

        Self {
            head,
            members: Vec::new(),
            size: 1,
            color: rng.gen::<f64>() * math::TAU,
        }
    }

    pub fn head(&self) -> usize {
        self.head
    }

    pub fn members(&self) -> &Vec<usize> {
        &self.members
    }

    pub fn size(&self) -> usize {
        self.size
    }

    pub fn color(&self) -> f64 {
        self.color
    }

    pub fn add_member(&mut self, node: usize) {
        self.members.push(node);
        self.size += 1;
    }

    pub fn set_color(&mut self, color: f64) {
        self.color = color;
    }
}

pub struct ClusterMap {
    map: HashMap<usize, Cluster>,
}

impl ClusterMap {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

    pub fn insert(&mut self, cluster: Cluster) {
        self.map.insert(cluster.head(), cluster);
    }

    pub fn get(&self, id: usize) -> Option<&Cluster> {
        self.map.get(&id)
    }

    pub fn get_mut(&mut self, id: usize) -> Option<&mut Cluster> {
        self.map.get_mut(&id)
    }

    pub fn remove(&mut self, id: usize) -> Option<Cluster> {
        self.map.remove(&id)
    }

    pub fn clusters(&self) -> Vec<&Cluster> {
        self.map.values().collect()
    }

    pub fn clusters_mut(&mut self) -> Vec<&mut Cluster> {
        self.map.values_mut().collect()
    }
}

pub fn render(cluster: &Cluster, satellites: &Vec<SatellitePosition>) -> Html {
    let head = satellites.get(cluster.head()).unwrap();

    let x1 = format!("{:.3}", head.screen_position().x);
    let y1 = format!("{:.3}", head.screen_position().y);
    let x2 = format!("{:.3}", SIZE.x / 2.0);
    let y2 = format!("{:.3}", SIZE.y / 2.0);

    html! {
        <g>
            { cluster.members().iter().filter_map(|id| satellites.get(*id)).map(|m| render_edge(&head, m)).collect::<Vec<_>>() }
            <line x1={x1} y1={y1} x2={x2} y2={y2} stroke="green" stroke-width="1" opacity="0.5" />
        </g>
    }
}

fn render_edge(sat1: &SatellitePosition, sat2: &SatellitePosition) -> Html {
    let sat1_pos = sat1.screen_position();
    let sat2_pos = sat2.screen_position();

    let x1 = format!("{:.3}", sat1_pos.x);
    let y1 = format!("{:.3}", sat1_pos.y);
    let x2 = format!("{:.3}", sat2_pos.x);
    let y2 = format!("{:.3}", sat2_pos.y);

    html! {
        <line x1={x1} y1={y1} x2={x2} y2={y2} stroke="gray" stroke-width="1" opacity="0.5" />
    }
}