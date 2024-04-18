use gloo::timers::callback::Interval;
use yew::{html, Component, Context, Html, Properties, Callback};

use crate::components::info_panel;
use crate::math::Vector2D;
use crate::quadtree::quadtree::QuadTree;
use crate::satellite::{SatelliteProperties, SatellitePosition, SatelliteEnergy, SatelliteComms};
use crate::settings::Settings;

pub const SIZE: Vector2D = Vector2D::new(1200.0, 1200.0);

#[derive(Debug)]
pub enum Msg {
    Tick,
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
    entities: Vec<Entity>,
    interval: Interval,
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

        let mut entities = Vec::with_capacity(settings.boids);

        for id in 0..settings.boids {
            let properties = SatelliteProperties::new_random(id);
            let position = SatellitePosition::new_random(&properties);
            let game = SatelliteEnergy::new_random();
            let communication = SatelliteComms::new(id);

            entities.push(Entity {
                id,
                position,
                properties,
                game,
                communication
            });
        }
    
        let interval = {
            let link = ctx.link().clone();
            Interval::new(settings.tick_interval_ms as u32, move || {
                link.send_message(Msg::Tick)
            })
        };

        let generation = ctx.props().generation;

        Self {
            entities,
            interval,
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
                }
                else {
                    // update entity position
                    for entity in self.entities.iter_mut() {
                        entity.position.update(&entity.properties, settings);
                    }

                    true
                }
            }
            Msg::ClickedSat(id) => {
                if self.selected_satellite_id == Some(id) {
                    self.selected_satellite_id = None;
                    self.entities[id].properties.set_selected(false);
                } else {
                    if self.selected_satellite_id.is_some() {
                        let prev_id = self.selected_satellite_id.unwrap();
                        self.entities[prev_id].properties.set_selected(false);
                    }
                    self.selected_satellite_id = Some(id);
                    self.entities[id].properties.set_selected(true);
                }
                true
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
            self.entities.clear();

            self.selected_satellite_id = None;

            let settings = &props.settings;

            // Generate new entities
            for id in 0..settings.boids {
                let properties = SatelliteProperties::new_random(id);
                let position = SatellitePosition::new_random(&properties);
                let game = SatelliteEnergy::new_random();
                let communication = SatelliteComms::new(id);

                self.entities.push(Entity {
                    id,
                    position,
                    properties,
                    game,
                    communication,
                });
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

                { for self.entities.iter().map(|e| e.render(onclick_cb.clone())) }

                if let Some(selected_satellite_id) = self.selected_satellite_id {
                    { info_panel::render(&self.entities[selected_satellite_id]) }
                }

                if self.qtree.is_some() && self.show_qtree {
                    { self.qtree.as_ref().unwrap().render() }
                }
            </svg>
        }
    }
}

