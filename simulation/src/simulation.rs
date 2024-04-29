use gloo::timers::callback::Interval;
use yew::{html, Callback, Component, Context, Html, Properties};

use crate::cluster::{Cluster, ClusterMap};
use crate::components::info_panel;
use crate::math::Vector2D;
use crate::quadtree::{box2d::Box2d, quadtree::QuadTree, types::*};
use crate::satellite::{SatelliteEnergy, SatellitePosition, SatelliteProperties};
use crate::satellite;
use crate::settings::Settings;

pub const SIZE: Vector2D = Vector2D::new(1200.0, 1200.0);

#[derive(Debug)]
pub enum Msg {
    Tick,
    CommsTick,
    GameTick,
    ClickedSat(usize),
}

#[derive(Clone, Debug, PartialEq, Properties)]
pub struct Props {
    pub settings: Settings,
    #[prop_or_default]
    pub generation: usize,
    #[prop_or_default]
    pub paused: bool,
    #[prop_or(false)]
    pub show_qtree: bool,
}

pub struct Simulation {
    entity_props: Vec<SatelliteProperties>,
    entity_positions: Vec<SatellitePosition>,
    entity_energy: Vec<SatelliteEnergy>,
    interval: Interval,
    comms_interval: Interval,
    game_interval: Interval,
    generation: usize,
    qtree: Option<QuadTree<usize>>,
    show_qtree: bool,
    selected_satellite_id: Option<usize>,
    cluster_map: ClusterMap,
}
impl Component for Simulation {
    type Message = Msg;
    type Properties = Props;

    fn create(ctx: &Context<Self>) -> Self {
        let settings = ctx.props().settings.clone();

        let mut entity_props = Vec::with_capacity(settings.boids);
        let mut entity_positions = Vec::with_capacity(settings.boids);
        let mut entity_energy = Vec::with_capacity(settings.boids);

        for id in 0..settings.boids {
            let properties = SatelliteProperties::new_random(id);
            let position = SatellitePosition::new_random(&properties);
            let game = SatelliteEnergy::new_random(id, &settings);

            entity_props.push(properties);
            entity_positions.push(position);
            entity_energy.push(game);
        }

        let interval = {
            let link = ctx.link().clone();
            Interval::new(settings.tick_interval_ms as u32, move || {
                link.send_message(Msg::Tick)
            })
        };

        let comms_interval = {
            let link = ctx.link().clone();
            Interval::new(1000 as u32, move || link.send_message(Msg::CommsTick))
        };

        let game_interval = {
            let link = ctx.link().clone();
            Interval::new(333 as u32, move || link.send_message(Msg::GameTick))
        };

        let generation = ctx.props().generation;

        Self {
            entity_props,
            entity_positions,
            entity_energy,
            interval,
            comms_interval,
            game_interval,
            generation,
            qtree: None,
            show_qtree: false,
            selected_satellite_id: None,
            cluster_map: ClusterMap::new(),
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Tick => {
                let Props {
                    ref settings,
                    paused,
                    ..
                } = *ctx.props();

                if paused {
                    false
                } else {
                    // update entity position
                    for (pos, props) in self
                        .entity_positions
                        .iter_mut()
                        .zip(self.entity_props.iter())
                    {
                        pos.update(props, settings);
                    }

                    true
                }
            }
            Msg::GameTick => {
                let Props {
                    paused,
                    ..
                } = *ctx.props();

                if paused {
                    false
                } else {
                    for cluster in self.cluster_map.clusters() {
                        if cluster.size() < 2 {
                            continue;
                        }

                        for &id in cluster.members() {
                            self.entity_energy.get_mut(id).expect("Couldn't get sat in cluster").update_game(cluster);
                        }
                        
                        // All sats in cluster should've made a decision to enter or leave
                        
                        for id in 0..cluster.size() {
                            let (first, sec) = cluster.members().split_at(id);
                            let (current, other) = sec.split_at(1);
                            let sat_ptr  = self.entity_energy.as_mut_ptr();
                            unsafe {
                                let neighbors = other.iter().chain(first).filter_map(|&eid| sat_ptr.add(eid).as_ref()).collect::<Vec<_>>();
                                self.entity_energy.get_mut(current[0]).unwrap().update(neighbors);
                            }
                        }
                    }
 
                    true
                }
            }
            Msg::ClickedSat(id) => {
                if self.selected_satellite_id == Some(id) {
                    self.selected_satellite_id = None;
                    self.entity_props[id].set_selected(false);
                } else {
                    if self.selected_satellite_id.is_some() {
                        let prev_id = self.selected_satellite_id.unwrap();
                        self.entity_props[prev_id].set_selected(false);
                    }
                    self.selected_satellite_id = Some(id);
                    self.entity_props[id].set_selected(true);
                }
                true
            }
            Msg::CommsTick => {
                let Props {
                    ref settings,
                    paused,
                    ..
                } = *ctx.props();

                if paused {
                    false
                } else {
                    // Create quadtree
                    let mut qtree = QuadTree::new(
                        Box2d::new(Point::new(0.0, SIZE.y), Point::new(SIZE.x, 0.0)),
                        4,
                    );

                    // Build quadtree for efficient Entity search
                    for (id, entity) in self.entity_positions.iter().enumerate() {
                        let position = entity.screen_position();
                        qtree.insert(Point::new(position.x, position.y), id);
                    }

                    // Iterate through all satellites to create a list of cluster head candidates based on energy if exceeds threshold

                    let cluster_head_candidates = self
                        .entity_energy
                        .iter()
                        .enumerate()
                        .filter_map(|(id, energy)| {
                            if energy.energy() > settings.energy_threshold {
                                Some(id)
                            } else {
                                None
                            }
                        })
                        .collect::<Vec<_>>();

                    // Consolidate cluster heads based on distance

                    let mut cluster_heads: Vec<usize> = Vec::new();

                    for id in cluster_head_candidates {
                        let position = self.entity_positions[id].screen_position();
                        let mut is_cluster_head = true;

                        for head in &cluster_heads {
                            let head_pos = self.entity_positions[*head].screen_position();
                            let distance = (position - head_pos).magnitude();

                            if distance < settings.cluster_distance {
                                is_cluster_head = false;
                                break;
                            }
                        }

                        if is_cluster_head {
                            cluster_heads.push(id);
                        }
                    }

                    // Create edge list of members to their nearest cluster heads
                    let mut clusters = ClusterMap::new();

                    // Create clusters using cluster heads
                    for ch_id in cluster_heads.iter() {
                        let mut cluster = Cluster::new(*ch_id);
                        if let Some(prev_cluster) = self.cluster_map.get(cluster.head()) {
                            cluster.set_color(prev_cluster.color());
                            self.entity_props[cluster.head()].set_color(prev_cluster.color());
                        } else {
                            cluster.set_color(self.entity_props[cluster.head()].color());
                        }
                        clusters.insert(cluster);
                    }

                    // Assign members to the nearest cluster head
                    for prop in self.entity_props.iter_mut() {
                        if cluster_heads.contains(&prop.id()) {
                            // skip assignment for cluster heads
                            continue;
                        }

                        let id = prop.id(); 
                        let pos = self.entity_positions.get(id).unwrap();
                        let position = pos.screen_position();
                        let mut nearest_distance = f32::INFINITY;
                        let mut nearest_head = None;

                        for head in &cluster_heads {
                            let head_pos = self.entity_positions[*head].screen_position();
                            let distance = (position - head_pos).magnitude();

                            if distance < nearest_distance {
                                nearest_distance = distance;
                                nearest_head = Some(*head);
                            }
                        }

                        if let Some(head) = nearest_head {
                            let cluster = clusters.get_mut(head).unwrap();
                            cluster.add_member(id);
                        }
                    }

                    // Set cluster colors to the average color of all members
                    for cluster in clusters.clusters_mut() {
                        if cluster.size() < 2 {
                            continue;
                        }

                        // Mix member colors
                        let mut member_color = cluster.members().iter().map(|id| self.entity_props[*id].color()).sum::<f32>();
                        member_color /= cluster.members().len() as f32;
                        let head_color = cluster.color();
                        let mut color = (head_color + member_color) / 2.0;
                        color %= 360.0;

                        // Set color
                        cluster.set_color(color);
                        self.entity_props[cluster.head()].set_color(color);
                        for member in cluster.members() {
                            self.entity_props[*member].set_color(color);
                        }
                    }

                    self.cluster_map = clusters;
                    self.qtree = Some(qtree);

                    true
                }
            }
        }
    }

    fn changed(&mut self, ctx: &Context<Self>, old_props: &Self::Properties) -> bool {
        let props = ctx.props();

        if old_props.show_qtree != props.show_qtree || self.generation != props.generation {
            self.show_qtree = props.show_qtree;
        }

        let should_reset =
            old_props.settings != props.settings || self.generation != props.generation;

        self.generation = props.generation;

        if should_reset {
            // Clear entity info
            self.entity_props.clear();
            self.entity_positions.clear();
            self.entity_energy.clear();

            self.selected_satellite_id = None;
            self.cluster_map = ClusterMap::new();

            let settings = &props.settings;

            // Generate new entities
            for id in 0..settings.boids {
                let properties = SatelliteProperties::new_random(id);
                let position = SatellitePosition::new_random(&properties);
                let game = SatelliteEnergy::new_random(id, &settings);

                self.entity_props.push(properties);
                self.entity_positions.push(position);
                self.entity_energy.push(game);
            }

            // as soon as the previous task is dropped it is cancelled.
            // We don't need to worry about manually stopping it.
            self.interval = {
                let link = ctx.link().clone();
                Interval::new(settings.tick_interval_ms as u32, move || {
                    link.send_message(Msg::Tick)
                })
            };

            // as soon as the previous task is dropped it is cancelled.
            // We don't need to worry about manually stopping it.
            self.comms_interval = {
                let link = ctx.link().clone();
                Interval::new(1000 as u32, move || {
                    link.send_message(Msg::CommsTick)
                })
            };

            self.game_interval = {
                let link = ctx.link().clone();
                Interval::new(333 as u32, move || link.send_message(Msg::GameTick))
            };

            true
        } else {
            false
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let view_box = format!("0 0 {} {}", SIZE.x, SIZE.y);
        let link = ctx.link().clone();
        let onclick_cb = Callback::from(move |id| link.send_message(Msg::ClickedSat(id)));
        let num_sats = self.entity_props.len();

        html! {
            <svg class="simulation-window" viewBox={view_box} preserveAspectRatio="xMidYMid">

                { self.cluster_map.clusters().iter().map(|e| crate::cluster::render(e, &self.entity_positions)).collect::<Vec<_>>() }

                { (0..num_sats).map(|id| {
                    satellite::render(&self.entity_props[id], &self.entity_positions[id], &self.entity_energy[id], onclick_cb.clone())
                }).collect::<Html>() }

                if let Some(id) = self.selected_satellite_id {
                    { info_panel::render(&self.entity_props[id], &self.entity_positions[id], &self.entity_energy[id]) }
                }

                if let Some(qtree) = self.qtree.as_ref() {
                    if self.show_qtree {
                        { qtree.render() }
                    }
                }

            </svg>
        }
    }
}
