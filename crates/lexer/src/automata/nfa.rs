use super::{util::*, Automaton, Category, IR};
use bit_set::{self, BitSet};
use std::fmt::{self, Debug, Formatter};

#[derive(Clone, PartialEq, Eq)]
struct NFANode {
    epsilon: BitSet,
    t: [BitSet; SIGMA],
}

impl Default for NFANode {
    fn default() -> Self {
        Self {
            epsilon: BitSet::new(),
            t: initialize(BitSet::new),
        }
    }
}

impl Debug for NFANode {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let strs = self
            .t
            .iter()
            .enumerate()
            .filter_map(|(i, v)| {
                char::from_u32(i as u32)
                    .filter(|_| !v.is_empty())
                    .map(|c| format!("{} -> {:?}", c, v))
            })
            .collect::<Vec<_>>();
        f.debug_struct("NFAEntry")
            .field("e", &self.epsilon)
            .field("t", &strs)
            .finish()
    }
}

pub(crate) struct NFABuilder {
    nodes: Vec<NFANode>,
    fs: BitSet,
}

impl NFABuilder {
    pub(crate) fn new() -> Self {
        Self {
            nodes: vec![NFANode::default()],
            fs: BitSet::new(),
        }
    }

    pub(crate) fn ir(mut self, ir: &IR) -> Self {
        self.add_ir(ir);
        self
    }

    pub(crate) fn add_ir(&mut self, ir: &IR) {
        let f = self.thompson(ir, 0);
        self.fs.insert(f);
    }

    // complete epsilon closure for NFA instance
    pub(crate) fn build(self) -> NFA {
        let Self { mut nodes, fs } = self;
        let n = nodes.len();
        let mut mark = vec![true; n];
        for (i, x) in nodes.iter_mut().enumerate() {
            x.epsilon.insert(i);
        }
        for _ in 0..n {
            for i in 0..n {
                if mark[i] {
                    let mut x = nodes[i].epsilon.clone();
                    for j in &nodes[i].epsilon {
                        x.union_with(&nodes[j].epsilon);
                    }
                    mark[i] = nodes[i].epsilon != x;
                    nodes[i].epsilon = x;
                }
            }
        }
        NFA { nodes, fs }
    }

    fn add_state(&mut self) -> usize {
        self.nodes.push(NFANode::default());
        self.nodes.len() - 1
    }

    fn thompson(&mut self, ir: &IR, mut q: usize) -> usize {
        use IR::*;
        match ir {
            E => {
                let f = self.add_state();
                self.nodes[q].epsilon.insert(f);
                f
            }
            L(v) => {
                let f = self.add_state();
                for &x in v {
                    self.nodes[q].t[x as usize].insert(f);
                }
                f
            }
            U(v) => {
                let fs: Vec<usize> = v
                    .iter()
                    .map(|x| {
                        let s = self.add_state();
                        self.nodes[q].epsilon.insert(s);
                        self.thompson(x, s)
                    })
                    .collect();
                let g = self.add_state();
                for f in fs {
                    self.nodes[f].epsilon.insert(g);
                }
                g
            }
            C(v) => {
                for x in v {
                    q = self.thompson(x, q);
                }
                q
            }
            K(x) => {
                let s = self.add_state();
                let f  = self.thompson(x, s);
                let g = self.add_state();
                self.nodes[q].epsilon.insert(s);
                self.nodes[q].epsilon.insert(g);
                self.nodes[f].epsilon.insert(s);
                self.nodes[f].epsilon.insert(g);
                g
            }
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct NFA {
    nodes: Vec<NFANode>,
    fs: BitSet,
}

impl Automaton for NFA {
    type State = BitSet;
    fn initial_state(&self) -> Self::State {
        self.nodes[0].epsilon.clone()
    }
    fn transition(&self, q: &Self::State, x: u8) -> Option<Self::State> {
        let mut s = BitSet::new();
        for i in q {
            for j in &self.nodes[i].t[x as usize] {
                s.union_with(&self.nodes[j].epsilon);
            }
        }
        Some(s).filter(|t| !t.is_empty())
    }
    fn category(&self, q: &Self::State) -> Option<Category> {
        self.fs.iter().position(|f| q.contains(f)).map(Category)
    }
}

#[cfg(test)]
mod tests {
    use super::IR::*;
    use super::*;
    use crate::automata::ParserAutomaton;
    use crate::combinator::Parser;

    fn ir_simple_1() -> IR {
        U(vec![E, C(vec![K(Box::new(L(vec![b'a']))), L(vec![b'b'])])])
    }

    fn ir_simple_2() -> IR {
        C(vec![L(vec![b'a']), L(vec![b'b']), L(vec![b'c'])])
    }

    fn ir() -> IR {
        K(Box::new(U(vec![
            L(vec![b'0']),
            K(Box::new(C(vec![
                L(vec![b'1']),
                K(Box::new(C(vec![
                    L(vec![b'0']),
                    K(Box::new(L(vec![b'1']))),
                    K(Box::new(C(vec![L(vec![b'0']), L(vec![b'1'])]))),
                    L(vec![b'0']),
                ]))),
                L(vec![b'1']),
            ]))),
        ])))
    }

    #[test]
    fn nfa_accept_simple_1() {
        let ir = ir_simple_1();
        let nfa = NFABuilder::new().ir(&ir).build();
        let p = ParserAutomaton(nfa);
        assert!(p.accept("".as_bytes()));
        assert!(p.accept("aaab".as_bytes()));
        assert!(!p.accept("c".as_bytes()));
        assert!(!p.accept("abab".as_bytes()));
    }

    #[test]
    fn nfa_accept_simple_2() {
        let ir = ir_simple_2();
        let nfa = NFABuilder::new().ir(&ir).build();
        let p = ParserAutomaton(nfa);
        assert!(!p.accept("".as_bytes()));
        assert!(p.accept("abc".as_bytes()));
        assert!(!p.accept("abcd".as_bytes()));
        assert!(!p.accept("cba".as_bytes()));
    }

    #[test]
    fn nfa_accept() {
        let ir = ir();
        let nfa = NFABuilder::new().ir(&ir).build();
        let p = ParserAutomaton(nfa);
        for x in 0..20 {
            let s = format!("{:b}", x);
            assert_eq!(p.accept(s.as_bytes()), x % 3 == 0);
        }
    }
}
