use gloo::timers::callback::Interval;
use yew::{html, Component, Context, Html, Properties};

use crate::boid::Boid;
use crate::math::Vector2D;
use crate::quadtree::quadtree::QuadTree;
use crate::settings::Settings;
use gloo::console::log;
use wasm_bindgen::JsValue;

pub const SIZE: Vector2D = Vector2D::new(1600.0, 1000.0);

#[derive(Debug)]
pub enum Msg {
    Tick,
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
    boids: Vec<Boid>,
    interval: Interval,
    generation: usize,
    qtree: Option<QuadTree<usize>>,
    show_qtree: bool,
}
impl Component for Simulation {
    type Message = Msg;
    type Properties = Props;

    fn create(ctx: &Context<Self>) -> Self {
        let settings = ctx.props().settings.clone();
        let boids = (0..settings.boids)
            .map(|_| Boid::new_random(&settings))
            .collect();

        let interval = {
            let link = ctx.link().clone();
            Interval::new(settings.tick_interval_ms as u32, move || {
                link.send_message(Msg::Tick)
            })
        };

        let generation = ctx.props().generation;
        Self {
            boids,
            interval,
            generation,
            qtree: None,
            show_qtree: false,
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
                    let (new_boids, qtree) = Boid::update_all(settings, &mut self.boids);
                    _ = std::mem::replace(&mut self.boids, new_boids);
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
            self.boids.clear();

            let settings = &props.settings;
            self.boids
                .resize_with(settings.boids, || Boid::new_random(settings));

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

    fn view(&self, _ctx: &Context<Self>) -> Html {
        let view_box = format!("0 0 {} {}", SIZE.x, SIZE.y);

        /*
        if let Some(qtree) = &self.qtree {
            log!(JsValue::from("Quadtree present"));
        }
        */

       html! {
            <svg class="simulation-window" viewBox={view_box}>
                { for self.boids.iter().map(Boid::render) }

                if self.qtree.is_some() && self.show_qtree {
                    { self.qtree.as_ref().unwrap().render() }
                }
            </svg>
       }
    }
}
