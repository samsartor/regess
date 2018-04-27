use petgraph::Graph;

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum Node {
    Start,
    Mid,
    End,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum Edge {
    Epsilon,
    Labeled(String),
}

impl Edge {
    pub fn new<L: ToString>(label: L) -> Edge {
        Edge::Labeled(label.to_string())
    }
}

use std::io::{Write, Result as IoResult};

pub fn write_dot<W: Write>(graph: &Graph<Node, Edge>, to: &mut W) -> IoResult<()> {
    writeln!(to, "digraph  {{\n")?;
    for id in graph.node_indices() {
        let node = &graph[id];
        let shape = match node {
            Node::End => "doublecircle",
            _ => "circle",
        };
        let label = match node {
            Node::Start => "S",
            _ => "",
        };
        writeln!(to, "\tnode [shape={}, label=\"{}\"] {};", shape, label, id.index())?;
    }
    for id in graph.edge_indices() {
        let (left, right) = graph.edge_endpoints(id).unwrap();
        let label = match graph[id] {
            Edge::Epsilon => "ε",
            Edge::Labeled(ref l) => l.as_str(),
        };
        writeln!(to, "\t{} -> {} [label = \"{}\"];", left.index(), right.index(), label.escape_debug())?;
    }
    writeln!(to, "}}")?;
    Ok(())
}
