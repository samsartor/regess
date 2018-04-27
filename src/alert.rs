use stdweb::{Reference, InstanceOf, Value};

#[derive(Clone, Debug, ReferenceType)]
pub struct Alerter(Reference);

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum AlertMode {
    Success,
    Info,
    Error,
    Warn,
}

impl Alerter {
    pub fn show(&mut self, msg: &str, mode: AlertMode) {
        use self::AlertMode::*;
        let mode = match mode {
            Success => "success",
            Error => "danger",
            Info => "primary",
            Warn => "warning",
        };
        js! { @{&self.0}.show(@{msg}, @{mode}); }
    }

    pub fn dismiss(&mut self) {
        js! { @{&self.0}.dismiss(); }
    }
}

impl InstanceOf for Alerter {
    fn instance_of(r: &Reference) -> bool {
        match js!( return "show" in @{r} && "dismiss" in @{r} ) {
            Value::Bool(is) => is,
            _ => panic!("constructor name equality is not bool"),
        }
    }
}

pub fn dismiss_alert() {
    match *::ALERT.lock().unwrap() {
        Some(ref mut a) => a.dismiss(),
        None => (),
    }
}

pub fn show_alert(msg: &str, mode: AlertMode) {
    match *::ALERT.lock().unwrap() {
        Some(ref mut a) => a.show(msg, mode),
        None => (),
    }
}
