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
#[macro_use]
extern crate failure;
extern crate regex_syntax;
#[macro_use]
extern crate percent_encoding;

pub mod gvd3;
pub mod dot;
mod logger;
mod alert;
pub mod grapher;

use std::sync::Mutex;
use std::ops::Deref;

use stdweb::{js_export, Reference};
use stdweb::web::{self, event, Element, IElement, INonElementParentNode, IEventTarget};
use stdweb::web::html_element::InputElement;
use stdweb::unstable::{TryInto};

use failure::{Error, err_msg};

use gvd3::Gvd3;
use grapher::Grapher;
use alert::{Alerter, dismiss_alert};
use percent_encoding::{PATH_SEGMENT_ENCODE_SET, utf8_percent_encode};

define_encode_set! {
    /// This encode set is used in the URL parser for query strings.
    pub QUERY_SEGMENT_ENCODE_SET = [PATH_SEGMENT_ENCODE_SET] | {'&'}
}

lazy_static! {
    pub static ref GVD3: Mutex<Option<Gvd3>> = Mutex::new(None);
    pub static ref ALERT: Mutex<Option<Alerter>> = Mutex::new(None);
    static ref RENDER_GRAPH: Mutex<Option<GraphUpdate>> = Mutex::new(None);
}

pub fn main() {
    stdweb::initialize();

    static LOGGER: &log::Log = &logger::Logger;
    log::set_logger(LOGGER).unwrap();
    log::set_max_level(log::LevelFilter::Trace);

    match start() {
        Ok(()) => debug!("Rust app started successfully"),
        Err(e) => error!("{:?}", e),
    }
}

struct GraphUpdate {
    input: InputElement,
    grapher: Grapher,
}

impl GraphUpdate {
    pub fn fire(&self) {
        trace!("Rendering graph");

        let gr = match self.grapher.machine(&self.input.raw_value()) {
            Ok(gr) => { dismiss_alert(); gr },
            Err(e) => { error!("{:?}", e); return }
        };

        if let Some(ref mut gvd3) = *GVD3.lock().unwrap() {
            gvd3.render(&gr);
        }
    }
}

#[js_export]
pub fn render_regex() {
    match RENDER_GRAPH.lock().as_ref().map(Deref::deref) {
        Ok(&Some(ref g)) => g.fire(),
        Ok(_) => error!("graph renderer not bound"),
        _ => error!("could not lock graph renderer"),
    }
}

pub fn start() -> Result<(), Error> {
    let document = web::document();

    let input: InputElement = document
        .get_element_by_id("regex")
        .ok_or(err_msg("missing regex input"))?
        .try_into()?;

    let input_val = input.clone();
    let regexr_link: Element = document
        .get_element_by_id("regexr-link")
        .ok_or(err_msg("missing regexr link"))?;

    let update_regexr = move || {
        let val = input_val.raw_value();
        let val = utf8_percent_encode(&val, QUERY_SEGMENT_ENCODE_SET);
        match regexr_link.set_attribute(
            "href",
            &format!("https://regexr.com/?expression={}", val))
        {
            Err(e) => error!("could not set regexr link: {}", e),
            Ok(_) => (),
        }
    };
    update_regexr();
    input.add_event_listener(move |_: event::InputEvent| update_regexr());

    let show = document
        .get_element_by_id("show")
        .ok_or(err_msg("missing show button"))?;

    let grapher = Grapher::new();
    *RENDER_GRAPH.lock().unwrap() = Some(GraphUpdate { input, grapher });

    show.add_event_listener(|_: event::ClickEvent| render_regex());

    Ok(())
}

#[js_export]
pub fn set_display(on: Reference) {
    if let Some(gvd3) = on.downcast() {
        *GVD3.lock().unwrap() = Some(gvd3);
        debug!("Graphviz display object set");
    }
    render_regex();
}

#[js_export]
pub fn set_alert(on: Reference) {
    if let Some(alerter) = on.downcast() {
        *ALERT.lock().unwrap() = Some(alerter);
        debug!("Graphviz alert set");
    }
    info!("I'm written in Rust!");
}
