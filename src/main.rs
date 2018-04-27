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
extern crate failure;
extern crate regex_syntax;

pub mod gvd3;
pub mod dot;
mod logger;
mod alert;
pub mod grapher;

use std::sync::Mutex;

use stdweb::{js_export, Reference};
use stdweb::web::{self, event, INonElementParentNode, IEventTarget};
use stdweb::web::html_element::InputElement;
use stdweb::unstable::{TryInto};

use failure::{Error, err_msg};

use gvd3::Gvd3;
use grapher::Grapher;
use alert::{Alerter, dismiss_alert};

lazy_static! {
    pub static ref GVD3: Mutex<Option<Gvd3>> = Mutex::new(None);
    pub static ref ALERT: Mutex<Option<Alerter>> = Mutex::new(None);
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

pub fn start() -> Result<(), Error> {
    let document = web::document();

    let input: InputElement = document
        .get_element_by_id("regex")
        .ok_or(err_msg("missing regex input"))?
        .try_into()?;

    let show = document
        .get_element_by_id("show")
        .ok_or(err_msg("missing show button"))?;

    let gr = Grapher::new();

    show.add_event_listener(move |_: event::ClickEvent| {
        trace!("Rendering graph");

        let gr = match gr.machine(&input.raw_value()) {
            Ok(gr) => { dismiss_alert(); gr },
            Err(e) => { error!("{:?}", e); return }
        };

        if let Some(ref mut gvd3) = *GVD3.lock().unwrap() {
            gvd3.render(&gr);
        }
    });

    Ok(())
}

#[js_export]
pub fn set_display(on: Reference) {
    if let Some(gvd3) = on.downcast() {
        *GVD3.lock().unwrap() = Some(gvd3);
        debug!("Graphviz display object set");
    }
}

#[js_export]
pub fn set_alert(on: Reference) {
    if let Some(alerter) = on.downcast() {
        *ALERT.lock().unwrap() = Some(alerter);
        debug!("Graphviz alert set");
    }
    info!("I'm written in Rust!");
}
