use std::collections::HashMap;

use super::Automaton;
use super::{nfa::NFA, util::SIGMA, Category};
use bit_set::BitSet;

#[derive(Clone, Copy, Debug)]
struct DFANode {
    c: Option<Category>,
    t: [Option<usize>; SIGMA],
}

impl DFANode {
    fn new(nfa: &NFA, s: &BitSet) -> Self {
        Self {
            c: nfa.category(s),
            t: [None; SIGMA],
        }
    }
}

pub(crate) struct DFA(Vec<DFANode>);

impl DFA {
    fn powerset_construction(nfa: &NFA) -> Self {
        let s = nfa.initial_state();
        let mut d = vec![DFANode::new(nfa, &s)];
        let mut v = vec![s.clone()];

        // Performance issue with email regex
        let mut m = HashMap::from([(s, 0)]);

        for i in 0.. {
            if i >= v.len() {
                break;
            }
            for x in 0..=255u8 {
                if let Some(u) = nfa.transition(&v[i], x) {
                    d[i].t[x as usize] = match m.get(&u) {
                        None => {
                            let k = v.len();
                            d.push(DFANode::new(nfa, &u));
                            v.push(u.clone());
                            m.insert(u, k);
                            Some(k)
                        }
                        a => a.cloned(),
                    };
                }
            }
        }
        DFA(d)
    }

    fn myhill_nerode(self) -> Self {
        use std::cmp::Ordering::*;

        let n = self.0.len();
        let mut mark: Vec<Vec<bool>> = (0..n)
            .map(|i| (0..i).map(|j| self.0[i].c != self.0[j].c).collect())
            .collect();
        loop {
            let mut exit = true;
            for i in 0..n {
                for j in 0..i {
                    if !mark[i][j] {
                        let markable = (0..SIGMA).any(|x| {
                            match (self.0[i].t[x], self.0[j].t[x]) {
                                (Some(a), Some(b)) => match a.cmp(&b) {
                                    Less => mark[b][a],
                                    Equal => false,
                                    Greater => mark[a][b],
                                },
                                (a, b) => a != b,
                            }
                        });
                        if markable {
                            mark[i][j] = true;
                            exit = false;
                        }
                    }
                }
            }
            if exit {
                break;
            }
        }

        let mut reindex: Vec<Result<usize, usize>> = vec![];
        let mut k = 0;
        for i in 0..n {
            if let Some(j) = mark[i].iter().position(|&x| !x) {
                reindex.push(reindex[j].and_then(|x| Err(x)));
            } else {
                reindex.push(Ok(k));
                k += 1;
            }
        }

        let mut d = Vec::new();
        for (i, e) in self.0.into_iter().enumerate() {
            if reindex[i].is_ok() {
                let DFANode { c, mut t } = e;
                for x in t.iter_mut() {
                    if let Some(t) = x {
                        *t = reindex[*t].unwrap_or_else(|z| z);
                    }
                }
                d.push(DFANode { c, t });
            }
        }
        DFA(d)
    }

    pub fn new(nfa: &NFA) -> Self {
        Self::myhill_nerode(Self::powerset_construction(nfa))
    }
}

impl Automaton for DFA {
    type State = usize;

    fn initial_state(&self) -> Self::State {
        0
    }

    fn transition(&self, q: &Self::State, x: u8) -> Option<Self::State> {
        self.0[*q].t[x as usize]
    }

    fn category(&self, q: &Self::State) -> Option<Category> {
        self.0[*q].c
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        automata::{
            dfa::DFA,
            nfa::NFABuilder,
            IR::{self, *}, ParserAutomaton,
        },
        combinator::Parser,
    };

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
    fn dfa_accept_simple_1() {
        let ir = ir_simple_1();
        let nfa = NFABuilder::new().ir(&ir).build();
        let dfa = DFA::new(&nfa);
        assert_eq!(dfa.0.len(), 3);
        let p = ParserAutomaton(dfa);
        assert!(p.accept("".as_bytes()));
        assert!(p.accept("aaab".as_bytes()));
        assert!(!p.accept("c".as_bytes()));
        assert!(!p.accept("abab".as_bytes()));
    }

    #[test]
    fn dfa_accept_simple_2() {
        let ir = ir_simple_2();
        let nfa = NFABuilder::new().ir(&ir).build();
        let dfa = DFA::new(&nfa);
        assert_eq!(dfa.0.len(), 4);
        let p = ParserAutomaton(dfa);
        assert!(!p.accept("".as_bytes()));
        assert!(p.accept("abc".as_bytes()));
        assert!(!p.accept("abcd".as_bytes()));
        assert!(!p.accept("cba".as_bytes()));
    }

    #[test]
    fn dfa_accept() {
        let ir = ir();
        let nfa = NFABuilder::new().ir(&ir).build();
        let dfa = DFA::new(&nfa);
        assert_eq!(dfa.0.len(), 3);
        let p = ParserAutomaton(dfa);
        for x in 0..20 {
            let s = format!("{:b}", x);
            assert_eq!(p.accept(s.as_bytes()), x % 3 == 0, "s: {}", s);
        }
    }
}
