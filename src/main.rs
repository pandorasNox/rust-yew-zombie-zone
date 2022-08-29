use std::rc::Rc;

use log::{info};
use yew::{prelude::*, virtual_dom::VNode};
use yewdux::prelude::*;

mod game;

enum Msg {
    Reset,
    Next,
    // Start,
    // Stop,
}

fn main() {
    println!("Hello, world!");
    wasm_logger::init(wasm_logger::Config::default());
    yew::start_app::<App>();
}

impl Reducer<game::State> for Msg {
    fn apply(&self, mut rc_state: Rc<game::State>) -> Rc<game::State> {
        let state = Rc::make_mut(&mut rc_state);

        match self {
            Msg::Reset => {
                log::info!("Msg::Reset");
                return Rc::new(game::State::new());
            }
            Msg::Next => {
                log::info!("Msg::Next");
                state.next();
                return Rc::new(state.clone());
            }
        }

        return Rc::new(state.clone());
    }
}

#[function_component(App)]
fn app() -> Html {
    let (state, dispatch) = use_store::<game::State>();

    let reset = dispatch.apply_callback(|_| Msg::Reset);
    let next = dispatch.apply_callback(|_| Msg::Next);

    html! {
        <>
            <h1>{"Welcome to the Zombie Zone"}</h1>
            <Grid />
            <div class="">
                <button onclick={reset} type="button" class="py-[.688rem] px-4 inline-flex justify-center items-center gap-2 rounded-md border-2 border-gray-200 font-semibold text-white hover:text-black hover:bg-white hover:border-white-500 focus:outline-none focus:ring-2 focus:ring-white-500 focus:ring-offset-2 transition-all text-sm dark:border-gray-700 dark:hover:border-white-500">
                {"Reset"}
                </button>
                <button onclick={next} type="button" class="py-[.688rem] px-4 inline-flex justify-center items-center gap-2 rounded-md border-2 border-gray-200 font-semibold text-white hover:text-black hover:bg-white hover:border-white-500 focus:outline-none focus:ring-2 focus:ring-white-500 focus:ring-offset-2 transition-all text-sm dark:border-gray-700 dark:hover:border-white-500">
                {"Next"}
                </button>
            </div>
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
                    let game::Field(field) = lane[i].clone();

                    if field.len() == 0 {
                        fields.push(html!{ <div>{"_"}</div> });
                        continue;
                    }

                    match field[0] {
                        game::Entity::Zombie => { fields.push(html!{<div>{"Z"}</div>}); }
                        game::Entity::Turret => { fields.push(html!{<div>{"T"}</div>}); }
                        game::Entity::Bullet => { fields.push(html!{<div>{"°"}</div>}); }
                    }
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
