use std::collections::HashMap;

use super::Automaton;
use super::{nfa::NFA, util::SIGMA, Category};
use bit_set::BitSet;

#[derive(Clone, Copy, Debug)]
struct DFAEntry {
    c: Option<Category>,
    t: [Option<usize>; SIGMA],
}

impl DFAEntry {
    fn new(nfa: &NFA, s: &BitSet) -> Self {
        Self {
            c: nfa.category(s),
            t: [None; SIGMA],
        }
    }
}

pub(crate) struct DFA(Vec<DFAEntry>);

impl DFA {
    fn powerset_construction(nfa: &NFA) -> Self {
        let s = nfa.initial_state();
        let mut d = vec![DFAEntry::new(nfa, &s)];
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
                            d.push(DFAEntry::new(nfa, &u));
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
                let DFAEntry { c, mut t } = e;
                for x in t.iter_mut() {
                    if let Some(t) = x {
                        *t = reindex[*t].unwrap_or_else(|z| z);
                    }
                }
                d.push(DFAEntry { c, t });
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
            nfa::NFA,
            IR::{self, *},
        },
        combinator::Parser,
    };

    fn ir_simple() -> IR {
        U(vec![E, C(vec![K(Box::new(L(b'a'))), L(b'b')])])
    }

    fn ir() -> IR {
        K(Box::new(U(vec![
            L(b'0'),
            K(Box::new(C(vec![
                L(b'1'),
                K(Box::new(C(vec![
                    L(b'0'),
                    K(Box::new(L(b'1'))),
                    K(Box::new(C(vec![L(b'0'), L(b'0')]))),
                    L(b'0'),
                ]))),
                L(b'1'),
            ]))),
        ])))
    }

    #[test]
    fn dfa_accept_simple() {
        let ir = ir_simple();
        let mut nfa = NFA::new();
        nfa.extend_with(&ir);
        let dfa = DFA::new(&nfa);
        assert_eq!(dfa.0.len(), 3);
        assert!(dfa.accept("".as_bytes()));
        assert!(dfa.accept("aaab".as_bytes()));
        assert!(!dfa.accept("c".as_bytes()));
        assert!(!dfa.accept("abab".as_bytes()));
    }

    #[test]
    fn dfa_accept() {
        let ir = ir();
        let mut nfa = NFA::new();
        nfa.extend_with(&ir);
        let dfa = DFA::new(&nfa);
        assert_eq!(dfa.0.len(), 3);
        for x in 0..20 {
            let s = format!("{:b}", x);
            assert_eq!(dfa.accept(s.as_bytes()), x % 3 == 0, "s: {}", s);
        }
    }
}
