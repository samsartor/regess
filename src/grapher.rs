use regex_syntax::ast::{self, Ast, parse::ParserBuilder};
use regex_syntax::hir::{self, Hir, translate::TranslatorBuilder};
use failure::Error;
use petgraph::{self, prelude::*};
use dot::{Node, Edge};

pub type Nodei = NodeIndex<u32>;
pub type Edgei = EdgeIndex<u32>;
pub type Graph = petgraph::Graph<Node, Edge>;

pub struct Grapher {
    pub parser: ParserBuilder,
    pub trans: TranslatorBuilder,
}

impl Grapher {
    pub fn new() -> Grapher {
        Grapher {
            parser: ParserBuilder::new(),
            trans: TranslatorBuilder::new(),
        }
    }

    pub fn ast_edge(&self, ast: &Ast) -> Edge {
        Edge::new(format!("{}", ast))
    }

    pub fn try_hir_connect(
        &self,
        hir: &Hir,
        left: Nodei,
        right: Nodei,
        machine: &mut Graph
    ) -> Result<(), Option<Error>> {
        use self::hir::HirKind::*;
        match *hir.kind() {
            Empty => (),
            Literal(_) => {
                machine.add_edge(left, right, Edge::new(format!("{}", hir)));
            },
            Alternation(ref hirs) => for h in hirs {
                self.try_hir_connect(h, left, right, machine)?
            },
            Class(hir::Class::Unicode(ref ls)) => {
                let mut chars = Vec::new();
                for range in ls.iter() {
                    for c in u32::from(range.start())..=u32::from(range.end()) {
                        if let Some(c) = ::std::char::from_u32(c) {
                            chars.push(c);
                            if chars.len() > 4 { return Err(None) }
                        }
                    }
                }
                for c in chars {
                    machine.add_edge(left, right, Edge::new(format!("{}", c)));
                }
            },
            _ => return Err(None),
        }
        Ok(())
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
                use self::ast::{RepetitionKind::*, RepetitionRange::*};
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
                    Range(ref range) => {
                        let base = match *range {
                            Exactly(n) => n,
                            AtLeast(n) => n,
                            Bounded(n, _) => n,
                        };
                        let mut at = left;
                        for _ in 0..base {
                            let next = machine.add_node(Node::mid());
                            self.connect(&rep.ast, at, next, machine)?;
                            at = next;
                        }
                        match range {
                            AtLeast(_) => self.connect(&rep.ast, at, at, machine)?,
                            Bounded(min, max) if max >= min => {
                                for _ in 0..(max - min) {
                                    machine.add_edge(at, right, Edge::Epsilon);
                                    let next = machine.add_node(Node::mid());
                                    self.connect(&rep.ast, at, next, machine)?;
                                    at = next;
                                }
                            },
                            _ => (),
                        }
                        machine.add_edge(at, right, Edge::Epsilon);
                    },
                }
            },
            Assertion(_) => (),
            _ => {
                let hir = self.trans.build().translate("" /* TODO: need source */, ast)?;
                match self.try_hir_connect(&hir, left, right, machine) {
                    Ok(()) => (),
                    Err(None) => {
                        machine.add_edge(left, right, self.ast_edge(ast));
                    },
                    Err(Some(e)) => return Err(e),
                }
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


// - deduplicate
// - unify ε cycles
// - unify ε trees
// - backpropagate ε-termination

fn merge(nodes: &[Nodei], graph: &mut Graph) {
    use self::Direction::*;

    let mut node = Node::mid();
    let mut outs = Vec::new();
    let mut ins = Vec::new();

    for &n in nodes {
        node = Node::merge(node, graph[n]);
        outs.extend(graph
            .edges_directed(n, Outgoing)
            .map(|e| (n, e.weight().clone(), e.target())));
        ins.extend(graph
            .edges_directed(n, Incoming)
            .map(|e| (e.source(), e.weight().clone(), n))
            .filter(|&(s, _, t)| s != t)); // remove self edges
    }

    let node = graph.add_node(node);
    for (o, w, mut t) in outs {
        if o == t { t = node }; // re-target self edge
        graph.add_edge(node, t, w);
    }
    for (s, w, _) in ins {
        graph.add_edge(s, node, w);
    }
    for &n in nodes { graph.remove_node(n); }
}

// TODO: tests and optimizations (`StableGraph`?)
fn simplify(g: &mut Graph) {
    // ===============
    // | deduplicate |
    // ===============
    let mut set = ::std::collections::HashSet::new();
    g.retain_edges(|g, e| set.insert((g[e].clone(), g.edge_endpoints(e).unwrap())));

    // ==================
    // | unify ε cycles |
    // ==================
    // TODO

    // =================
    // | unify ε trees |
    // =================
    fn find_eps_edge(g: &Graph) -> Option<Edgei> {
        use self::Direction::*;

        for edge in g.edge_indices() {
            if g[edge] != Edge::Epsilon {
                // can't combine, has state change
                continue
            }

            let (a, b) = g.edge_endpoints(edge).unwrap();

            // no extra outputs that might not be universally connected
            let mut only_out = g.edges_directed(a, Outgoing).count() == 1;
            only_out &= !g[a].end;

            // no extra inputs that might not be universally connected
            let mut only_in = g.edges_directed(b, Incoming).count() == 1;
            only_in &= !g[b].start;

            if only_in || only_out {
                // can combine, no unconnected outer edges!
                return Some(edge)
            }
        }

        // couldn't find opportunity for simplification
        None
    }

    // do removal
    while let Some(e) = find_eps_edge(g) {
        let (a, b) = g.edge_endpoints(e).unwrap();
        if a != b {
            // nodes are different, merge
            merge(&[a, b], g);
        } else {
            // nodes are same, just remove edge
            g.remove_edge(e);
        }
    }

    // =============================
    // | backpropagate termination |
    // =============================
    // TODO

    // remove unconnected
    g.retain_nodes(|g, n| g.neighbors_undirected(n).count() > 0);
}
