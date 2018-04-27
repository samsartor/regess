use petgraph::Graph;

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub struct Node {
    pub start: bool,
    pub end: bool,
}

impl Node {
    pub fn new(start: bool, end: bool) -> Node {
        Node { start, end }
    }

    pub fn start() -> Node { Node::new(true, false) }
    pub fn end() -> Node { Node::new(false, true) }
    pub fn mid() -> Node { Node::new(false, false) }

    pub fn merge(a: Node, b: Node) -> Node {
        Node::new(a.start || b.start, a.end || b.end)
    }
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
        let shape = match node.end {
            true => "doublecircle",
            false => "circle",
        };
        let label = match node.start {
            true => "S",
            false => "",
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
