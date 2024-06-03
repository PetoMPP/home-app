use crate::Sensor;
use yew::prelude::*;

#[derive(Properties, Clone, PartialEq)]
pub struct HomeProps {
    pub sensors: Vec<Sensor>,
}

#[function_component(Home)]
pub fn home(props: &HomeProps) -> Html {
    html! {
        <div class={"flex-col"}>
            <h1 class={"text-2xl pb-4"}>{"Welcome to Home API!"}</h1>
            <p class={"text-xl"}> {"Here are the sensors available:"}</p>
            <table class={"table table-zebra"}>
                <thead>
                    <tr>
                        <th>{"ID"}</th>
                        <th>{"Features"}</th>
                    </tr>
                </thead>
                <tbody>
                    { for props.sensors.iter().map(|sensor| html! {
                        <tr>
                            <td>{&sensor.id}</td>
                            <td>{sensor.features}</td>
                        </tr>
                    }) }
                </tbody>
            </table>
        </div>
    }
}
