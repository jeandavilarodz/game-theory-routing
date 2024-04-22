///
/// This component renders the info and state of the clicked Satellite component instance
///

use yew::prelude::*;
use crate::simulation::SIZE;
use crate::satellite::*;


pub fn render(props: &SatelliteProperties, pos: &SatellitePosition, comms: &SatelliteComms, game: &SatelliteEnergy) -> Html {
    // Calculate X position offset for rendering the panel from the satellite based on distance from screen borders
    let x_offset = if pos.screen_position().x > (SIZE.x - 180.0) { -180.0 } else { 20.0 };
    let y_offset = if pos.screen_position().y > (SIZE.y - 100.0) { -100.0 } else { 20.0 };
    let x = format!("{:.3}", pos.screen_position().x + x_offset);
    let y = format!("{:.3}", pos.screen_position().y + y_offset);

    // Render a table in svg format for the satellite info
    html! {
        <svg id="info-panel" x={x} y={y}>
            <rect x="0" y="0" width="160" height="80" fill="dark-gray" opacity="0.75" rx="15" />
            <text x="16" y="26" font-weight="bold" fill="white">
                {format!("ID: {}", props.id())}
            </text>
            // rectangle that will inidicate the satellite's energy levels with a green bar
            <text x="16" y="44" font-weight="bold" fill="white">
                {format!("Energy: {}", game.energy())}
            </text>
            <text x="16" y="62" font-weight="bold" fill="white">
                {format!("No. packets: {}", comms.packets().len())}
            </text>
        </svg>
    }
}
