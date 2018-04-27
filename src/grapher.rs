use regex_syntax::ast::{self, Ast, parse::ParserBuilder};
use failure::Error;
use petgraph::{self, prelude::*};
use dot::{Node, Edge};

pub type Nodei = NodeIndex<u32>;
pub type Edgei = EdgeIndex<u32>;
pub type Graph = petgraph::Graph<Node, Edge>;

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
        left: Nodei,
        right: Nodei,
        machine: &mut Graph
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
                        machine.add_node(Node::mid())
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
                        let s = machine.add_node(Node::mid());
                        let j = machine.add_node(Node::mid());
                        machine.add_edge(j, s, Edge::Epsilon);
                        machine.add_edge(left, s, Edge::Epsilon);
                        machine.add_edge(j, right, Edge::Epsilon);
                        self.connect(&rep.ast, s, j, machine)?;
                    },
                    ZeroOrMore => {
                        let m = machine.add_node(Node::mid());
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

    pub fn machine(&self, regex: &str) -> Result<Graph, Error> {
        let ast = self.parser.build().parse(regex)?;

        let mut gr = Graph::new();
        let left = gr.add_node(Node::start());
        let right = gr.add_node(Node::end());
        self.connect(&ast, left, right, &mut gr)?;

        simplify(&mut gr);

        Ok(gr)
    }
}

fn find_collapse(g: &Graph) -> Option<(Nodei, Nodei)> {
    for node in g.node_indices() {
        // terminal nodes are an exception
        if g[node].end { continue }

        let mut edges = g.edges(node);
        // if has outgoing edge
        let edge = match edges.next() {
            Some(e) => e,
            None => continue,
        };
        // if only edge is epsilon
        match (edge.weight(), edges.next()) {
            (Edge::Epsilon, None) => return Some((node, edge.target())),
            _ => (),
        }
    }
    None
}

fn find_split_end(g: &Graph) -> Option<Edgei> {
    for node in g.node_indices() {
        if g.edges(node).count() == 0 { // terminal node
            for edge in g.edges_directed(node, Direction::Incoming) {
                if *edge.weight() == Edge::Epsilon { // can split
                    return Some(edge.id());
                }
            }
        }
    }

    None
}

// TODO: tests and optimizations (`StableGraph`?)
// TODO: simplify "b*c"
fn simplify(g: &mut Graph) {
    // deduplicate
    let mut set = ::std::collections::HashSet::new();
    g.retain_edges(|g, e| set.insert((g[e].clone(), g.edge_endpoints(e).unwrap())));

    // split terminals
    while let Some(e) = find_split_end(g) {
        let (s, t) = g.edge_endpoints(e).unwrap();
        g[s] = Node::merge(g[s], g[t]);
        g.remove_edge(e);
    }

    // remove single epsilons
    while let Some((s, t)) = find_collapse(g) {
        let mut to_add = Vec::new();
        for e in g.edges_directed(s, Direction::Incoming) {
            to_add.push((e.source(), t, e.weight().clone()));
        }
        for (a, b, w) in to_add {
            g.add_edge(a, b, w);
        }
        g[t] = Node::merge(g[s], g[t]);
        g.remove_node(s);
    }

    // remove unconnected
    g.retain_nodes(|g, n| g.neighbors_undirected(n).count() > 0);
}
