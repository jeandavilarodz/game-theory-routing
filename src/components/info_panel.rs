///
/// This component renders the info and state of the clicked Satellite component instance
///

use yew::prelude::*;
use crate::satellite::Satellite;
use crate::simulation::SIZE;


#[derive(Debug, PartialEq, Properties)]
pub struct InfoPanelProps {
    pub satellite: Satellite,
}

pub struct InfoPanel {
    satellite: Satellite,
}

impl Component for InfoPanel {
    type Message = ();
    type Properties = InfoPanelProps;

    fn create(ctx: &Context<Self>) -> Self {
        Self {
            satellite: ctx.props().satellite.clone(),
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, _msg: Self::Message) -> bool {
        unimplemented!()
    }

    fn changed(&mut self, ctx: &Context<Self>, _old_props: &Self::Properties) -> bool {
        self.satellite = ctx.props().satellite.clone();
        true
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        // Calculate X position offset for rendering the panel from the satellite based on distance from screen borders
        let sat_screen_coord_x = self.satellite.position.x + (SIZE.x / 2.0);
        let sat_screen_coord_y = self.satellite.position.y + (SIZE.y / 2.0);
        let x = format!("{}", sat_screen_coord_x + if sat_screen_coord_x > (SIZE.x - 180.0) { -200.0 } else { 20.0 });
        let y = format!("{}", sat_screen_coord_y + if sat_screen_coord_y > (SIZE.y - 100.0) { -100.0 } else { 20.0 });

        // Render a table in svg format for the satellite info
        html! {
            <svg id="info-panel" x={format!("{}px", x)} y={format!("{}px", y)}>
                <rect x="0" y="0" width="160px" height="80px" fill="dark-gray" opacity="0.8" rx="15" />
                <text x="16px" y="26px" font-weight="bold" fill="white">
                    {format!("ID: {}", self.satellite.id)}
                </text>
                // rectangle that will inidicate the satellite's energy levels with a green bar
                <text x="16px" y="44px" font-weight="bold" fill="white">
                    {format!("Energy: {}", self.satellite.energy)}
                </text>
                <text x="16px" y="62px" font-weight="bold" fill="white">
                    {format!("No. packets: {}", self.satellite.packets.len())}
                </text>
            </svg>
        }
    }
}