#![feature(proc_macro)]

#[macro_use]
extern crate stdweb;

#[macro_use]
extern crate serde_derive;
extern crate serde_json;

extern crate petgraph;

use stdweb::{web, js_export};
use web::{
    event,
    INode,
    INonElementParentNode,
    IEventTarget,
    IElement,
    IHtmlElement,
};

pub fn main() {
    stdweb::initialize();

    if let Some(button) = web::document().get_element_by_id("clickme") {
        button.add_event_listener(|e: event::ClickEvent| testfun("clicked"));
    }
}

pub struct Graph {

}

#[js_export]
pub fn testfun(text: &str) {
    use web::alert;

    alert(&format!("Rust says \"{}\"!", text))
}
