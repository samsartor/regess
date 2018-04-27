use regex_syntax::ast::{self, Ast, parse::ParserBuilder};
use failure::Error;
use petgraph::{Graph, graph::NodeIndex};
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

    pub fn connect(
        &self,
        ast: &Ast,
        left: NodeIndex<u32>,
        right: NodeIndex<u32>,
        machine: &mut Graph<Node, Edge>
    ) -> Result<(), Error> {
        use self::Ast::*;

        match *ast {
            Empty(_) => {
                machine.add_edge(left, right, Edge::Epsilon);
            },
            Alternation(ref alt) => {
                for ast in &alt.asts {
                    self.connect(ast, left, right, machine)?;
                }
            },
            Concat(ref con) => {
                let con = &con.asts;
                let mut l = left;
                for (ind, ast) in con.iter().enumerate() {
                    let r = if ind == con.len() - 1 { right }
                    else {
                        machine.add_node(Node::Mid)
                    };
                    self.connect(ast, l, r, machine)?;
                    l = r;
                }
            },
            Group(ref gr) => self.connect(&gr.ast, left, right, machine)?,
            Repetition(ref rep) => {
                use self::ast::RepetitionKind::*;
                match rep.op.kind {
                    ZeroOrOne => {
                        machine.add_edge(left, right, Edge::Epsilon);
                        self.connect(&rep.ast, left, right, machine)?
                    },
                    OneOrMore => {
                        let s = machine.add_node(Node::Mid);
                        let j = machine.add_node(Node::Mid);
                        machine.add_edge(j, s, Edge::Epsilon);
                        machine.add_edge(left, s, Edge::Epsilon);
                        machine.add_edge(j, right, Edge::Epsilon);
                        self.connect(&rep.ast, s, j, machine)?;
                    },
                    ZeroOrMore => {
                        let m = machine.add_node(Node::Mid);
                        machine.add_edge(left, m, Edge::Epsilon);
                        machine.add_edge(m, right, Edge::Epsilon);
                        self.connect(&rep.ast, m, m, machine)?;
                    },
                    _ => bail!("unsupported repetition"),
                }
            },
            _ => {
                machine.add_edge(left, right, Edge::new(format!("{}", ast)));
            },
        }

        Ok(())
    }

    pub fn machine(&self, regex: &str) -> Result<Graph<Node, Edge>, Error> {
        let ast = self.parser.build().parse(regex)?;

        let mut gr = Graph::new();
        let left = gr.add_node(Node::Start);
        let right = gr.add_node(Node::End);
        self.connect(&ast, left, right, &mut gr)?;

        let mut set = ::std::collections::HashSet::new();
        gr.retain_edges(|g, e| set.insert((g[e].clone(), g.edge_endpoints(e).unwrap())));

        Ok(gr)
    }
}
