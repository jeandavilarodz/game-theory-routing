use std::char::MAX;

use gloo::timers::callback::Interval;
use yew::{html, Callback, Component, Context, Html, Properties};

use crate::components::info_panel;
use crate::math::Vector2D;
use crate::quadtree::{box2d::Box2d, quadtree::QuadTree, types::*};
use crate::satellite::{SatelliteComms, SatelliteEnergy, SatellitePosition, SatelliteProperties, MAX_DISTANCE};
use crate::settings::Settings;

use gloo::console::log;
use wasm_bindgen::JsValue;

pub const SIZE: Vector2D = Vector2D::new(1200.0, 1200.0);

#[derive(Debug)]
pub enum Msg {
    Tick,
    CommsTick,
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

pub struct Entity {
    pub id: usize,
    pub position: SatellitePosition,
    pub properties: SatelliteProperties,
    pub communication: SatelliteComms,
    pub game: SatelliteEnergy,
}

impl Entity {
    fn render(&self, onclick_cb: Callback<usize>) -> Html {
        crate::satellite::render(&self.properties, &self.position, onclick_cb)
    }
}

pub struct Simulation {
    entity_props: Vec<SatelliteProperties>,
    entity_positions: Vec<SatellitePosition>,
    entity_comms: Vec<SatelliteComms>,
    entity_energy: Vec<SatelliteEnergy>,
    interval: Interval,
    comms_interval: Interval,
    generation: usize,
    qtree: Option<QuadTree<usize>>,
    show_qtree: bool,
    selected_satellite_id: Option<usize>,
}
impl Component for Simulation {
    type Message = Msg;
    type Properties = Props;

    fn create(ctx: &Context<Self>) -> Self {
        let settings = ctx.props().settings.clone();

        let mut entity_props = Vec::with_capacity(settings.boids);
        let mut entity_positions = Vec::with_capacity(settings.boids);
        let mut entity_comms = Vec::with_capacity(settings.boids);
        let mut entity_energy = Vec::with_capacity(settings.boids);

        for id in 0..settings.boids {
            let properties = SatelliteProperties::new_random(id);
            let position = SatellitePosition::new_random(&properties);
            let game = SatelliteEnergy::new_random();
            let communication = SatelliteComms::new(id);

            entity_props.push(properties);
            entity_positions.push(position);
            entity_comms.push(communication);
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

        let generation = ctx.props().generation;

        Self {
            entity_props,
            entity_positions,
            entity_comms,
            entity_energy,
            interval,
            comms_interval,
            generation,
            qtree: None,
            show_qtree: false,
            selected_satellite_id: None,
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

                    let curr_comms_state = self.entity_comms.as_mut_slice();
                    let num_iters = self.entity_props.len();

                    for id in 0..num_iters {
                        let visible_range = (self.entity_props[id].distance() / MAX_DISTANCE) * (SIZE.y / 2.0);
                        let position = self.entity_positions[id].screen_position();

                        let neighbor_ids = qtree
                            .query_range(Box2d::new(
                                Point::new(position.x - visible_range, position.y + visible_range),
                                Point::new(position.x + visible_range, position.y - visible_range),
                            ))
                            .iter()
                            .filter_map(|e| if id != *e.value { Some(*e.value) } else { None })
                            .collect::<Vec<_>>();

                        let neigh_pos = neighbor_ids
                            .iter()
                            .map(|i| &self.entity_positions[*i])
                            .collect::<Vec<_>>();


                        let (first, sec) = curr_comms_state.split_at_mut(id);
                        
                        // let debug = format!("{}:\nfirst: {:?}\nsec: {:?}", id, first, sec);
                        // log!(JsValue::from(&debug));
                        
                        let (current, other) = sec.split_at_mut(1);
                        
                        // let debug = format!("{}:\nother: {:?}\ncurrent:{:?}", id, other, current);
                        // log!(JsValue::from(&debug));
                        
                        let neigh_comms = other
                            .iter_mut()
                            .chain(first)
                            .filter_map(|e| {
                                if neighbor_ids.contains(&e.id()) {
                                    Some(e)
                                } else {
                                    None
                                }
                            })
                            .collect::<Vec<_>>();
                        

                        // let debug = format!("{}: {:?}", id, neigh_comms);
                        // log!(JsValue::from(&debug));

                        let ent_props = &self.entity_props[id];
                        let ent_pos = &self.entity_positions[id];
                        current[0].update(ent_props, ent_pos, neigh_pos, neigh_comms, settings);
                    }
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
            self.entity_comms.clear();
            self.entity_energy.clear();

            self.selected_satellite_id = None;

            let settings = &props.settings;

            // Generate new entities
            for id in 0..settings.boids {
                let properties = SatelliteProperties::new_random(id);
                let position = SatellitePosition::new_random(&properties);
                let game = SatelliteEnergy::new_random();
                let communication = SatelliteComms::new(id);

                self.entity_props.push(properties);
                self.entity_positions.push(position);
                self.entity_comms.push(communication);
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

            true
        } else {
            false
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let view_box = format!("0 0 {} {}", SIZE.x, SIZE.y);
        let link = ctx.link().clone();
        let onclick_cb = Callback::from(move |id| link.send_message(Msg::ClickedSat(id)));

        html! {
            <svg class="simulation-window" viewBox={view_box} preserveAspectRatio="xMidYMid">

                { (0..self.entity_props.len()).map(|id| {
                    crate::satellite::render(&self.entity_props[id], &self.entity_positions[id], onclick_cb.clone())
                }).collect::<Html>() }

                if let Some(id) = self.selected_satellite_id {
                    { info_panel::render(&self.entity_props[id], &self.entity_positions[id], &self.entity_comms[id], &self.entity_energy[id]) }
                }

                if self.qtree.is_some() && self.show_qtree {
                    { self.qtree.as_ref().unwrap().render() }
                }
            </svg>
        }
    }
}
