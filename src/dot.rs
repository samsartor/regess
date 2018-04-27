use petgraph::Graph;

#[derive(Clone, Debug)]
pub struct Node {
    pub end: bool,
}

impl Node {
    pub fn new() -> Node { Node { end: false } }
    pub fn end() -> Node { Node { end: true } }
}

#[derive(Clone, Debug)]
pub struct Edge {
    pub label: String,
}

impl Edge {
    pub fn new<L: ToString>(label: L) -> Edge {
        Edge { label: label.to_string() }
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
        writeln!(to, "\tnode [shape={}, label=\"\"] {};", shape, id.index())?;
    }
    for id in graph.edge_indices() {
        let (left, right) = graph.edge_endpoints(id).unwrap();
        let edge = &graph[id];
        writeln!(to, "\t{} -> {} [label = \"{}\"];", left.index(), right.index(), edge.label.escape_debug())?;
    }
    writeln!(to, "}}")?;
    Ok(())
}
