#![feature(proc_macro, str_escape)]

#[macro_use]
extern crate stdweb;
#[macro_use]
extern crate stdweb_derive;
extern crate petgraph;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;

pub mod gvd3;
pub mod dot;
mod logger;

use std::sync::Mutex;
use stdweb::{js_export, Reference};
use stdweb::web::{self, event, INonElementParentNode, IEventTarget};
use petgraph::Graph;
use dot::{Edge, Node};
use gvd3::Gvd3;

lazy_static! {
    pub static ref GVD3: Mutex<Option<Gvd3>> = Mutex::new(None);
}

pub fn main() {
    stdweb::initialize();

    static LOGGER: &log::Log = &logger::Logger;
    log::set_logger(LOGGER).unwrap();
    log::set_max_level(log::LevelFilter::Trace);

    info!("Rust init");

    let mut gr = Graph::new();
    let a = gr.add_node(Node::new());
    let b = gr.add_node(Node::new());
    let c = gr.add_node(Node::end());
    gr.add_edge(a, b, Edge::new("ε"));
    gr.add_edge(a, c, Edge::new("\\n"));
    gr.add_edge(b, c, Edge::new("!"));

    if let Some(b) = web::document().get_element_by_id("show") {
        b.add_event_listener(move |_: event::ClickEvent| {
            debug!("Rendering graph");
            if let Some(ref mut gvd3) = *GVD3.lock().unwrap() {
                gvd3.render(&gr);
            }
        });
    }
}

#[js_export]
pub fn set_display(on: Reference) {
    info!("Graphviz display object set");
    if let Some(gvd3) = on.downcast() {
        *GVD3.lock().unwrap() = Some(gvd3);
    }
}
