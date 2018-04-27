use regex_syntax::{ParserBuilder};
use failure::Error;
use petgraph::Graph;
use dot::{Node, Edge};

pub struct Grapher {
    pub parser: ParserBuilder,
}

impl Grapher {
    pub fn new() -> Grapher {
        Grapher {
            parser: ParserBuilder::new(),
        }
    }

    pub fn machine(&self, regex: &str) -> Result<Graph<Node, Edge>, Error> {
        let _hir = self.parser.build().parse(regex)?;

        let mut gr = Graph::new();
        let a = gr.add_node(Node::new());
        let b = gr.add_node(Node::new());
        let c = gr.add_node(Node::end());
        gr.add_edge(a, b, Edge::new("ε"));
        gr.add_edge(a, b, Edge::new("a"));
        gr.add_edge(b, c, Edge::new("b"));
        gr.add_edge(a, c, Edge::new("\\n"));

        Ok(gr)
    }
}
