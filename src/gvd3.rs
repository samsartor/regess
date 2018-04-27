use stdweb::{Reference, InstanceOf, Value};
use petgraph::Graph;
use dot::{Node, Edge, write_dot};

#[derive(Clone, Debug, ReferenceType)]
pub struct Gvd3(Reference);

impl Gvd3 {
    pub fn render(&mut self, g: &Graph<Node, Edge>) {
        let mut dot = Vec::new();
        write_dot(g, &mut dot).expect("could not write to vec");
        let dot = String::from_utf8(dot).unwrap();
        js! { @(no_return)
            @{&self.0}.renderDot(@{dot});
        };
    }

}

impl InstanceOf for Gvd3 {
    fn instance_of(r: &Reference) -> bool {
        match js!( return @{r}.constructor.name == "Graphviz"; ) { // TODO: this sucks
            Value::Bool(is) => is,
            _ => panic!("constructor name equality is not bool"),
        }
    }
}
