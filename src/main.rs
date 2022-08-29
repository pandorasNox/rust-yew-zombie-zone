use std::rc::Rc;

use yew::{prelude::*, virtual_dom::VNode};
use yewdux::prelude::*;

mod game;

enum Msg {
    Reset,
    Start,
    Stop,
    Next,
}

impl Reducer<game::State> for Msg {
    fn apply(&self, state: Rc<game::State>) -> Rc<game::State> {
        state
    }
}

fn main() {
    println!("Hello, world!");
    yew::start_app::<App>();
}

#[function_component(App)]
fn app() -> Html {
    let (state, _) = use_store::<game::State>();

    html! {
        <>
            <h1>{"Welcome to the Zombie Zone"}</h1>
            <Grid />
        </>
    }
}

#[function_component(Grid)]
fn grid() -> Html {
    let (state, _) = use_store::<game::State>();

    let mut lanes: Vec<VNode> = vec![];
    for i in 0..state.grid.len() {
        let mut fields: Vec<VNode> = vec![];

        match &state.grid[i] {
            Some(lane) => {
                for i in 0..lane.len() {
                    fields.push(html!{
                        <div>{"_"}</div>
                    })
                }
            },
            None => {},
        }

        let fields_as_html = fields.into_iter().collect::<Html>();

        lanes.push(html!{
            <>
                <div class="flex flex-row gap-2">
                    <div>{"Lane "}{i+1}{":"}</div>
                    <div>{"|"}</div>
                    {fields_as_html}
                    <div>{"|"}</div>
                </div>
            </>
        })
    }

    let lanes_as_html = lanes.into_iter().collect::<Html>();

    html! {
        <div class="text-3xl">
            {lanes_as_html}
        </div>
    }
}
