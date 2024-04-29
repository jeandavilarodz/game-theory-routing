use gloo::storage::{LocalStorage, Storage};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct Settings {
    /// amount of boids
    pub boids: usize,
    // time between each simulation tick
    pub tick_interval_ms: u64,
    /// view distance of a boid
    pub visible_range: f32,
    /// distance boids try to keep between each other
    pub min_distance: f32,
    /// max speed
    pub max_speed: f32,
    /// force multiplier for pulling boids together
    pub cohesion_factor: f32,
    /// force multiplier for separating boids
    pub separation_factor: f32,
    /// force multiplier for matching velocity of other boids
    pub alignment_factor: f32,
    /// controls turn speed to avoid leaving boundary
    pub turn_speed_ratio: f32,
    /// percentage of the size to the boundary at which a boid starts turning away
    pub border_margin: f32,
    /// factor for adapting the average color of the swarm
    pub color_adapt_factor: f32,
    /// Energy threshold to become a cluster head
    pub energy_threshold: f32,
    /// Distance threshold between cluster heads
    pub cluster_distance: f32,
    /// Energy cost of communication
    pub comms_cost: f32,
    /// Energy gain from environment
    pub energy_gain: f32,
}
impl Settings {
    const KEY: &'static str = "yew.boids.settings";

    pub fn load() -> Self {
        LocalStorage::get(Self::KEY).unwrap_or_default()
    }

    pub fn remove() {
        LocalStorage::delete(Self::KEY);
    }

    pub fn store(&self) {
        let _ = LocalStorage::set(Self::KEY, self);
    }
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            boids: 64,
            tick_interval_ms: 50,
            visible_range: 40.0,
            min_distance: 15.0,
            max_speed: 30.0,
            alignment_factor: 0.35,
            cohesion_factor: 0.20,
            separation_factor: 0.15,
            turn_speed_ratio: 0.33,
            border_margin: 0.1,
            color_adapt_factor: 0.05,
            energy_threshold: 50.0,
            cluster_distance: 100.0,
            comms_cost: 2.0,
            energy_gain: 3.0,
        }
    }
}
