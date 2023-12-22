use super::{Automaton, util::*, Category, IR};
use bit_set::{self, BitSet};
use std::fmt::{self, Debug, Formatter};

#[derive(Clone, PartialEq, Eq)]
struct NFAEntry {
    epsilon: Vec<usize>,
    t: [Vec<usize>; SIGMA],
}

impl Default for NFAEntry {
    fn default() -> Self {
        Self {
            epsilon: Vec::new(),
            t: initialize(Vec::new),
        }
    }
}

impl Debug for NFAEntry {
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

#[derive(Debug, PartialEq, Eq)]
pub struct NFA {
    entries: Vec<NFAEntry>,
    final_states: Vec<usize>,
}

impl NFA {
    pub fn new() -> Self {
        Self {
            entries: vec![NFAEntry::default()],
            final_states: Vec::new(),
        }
    }

    fn add_state(&mut self) -> usize {
        self.entries.push(NFAEntry::default());
        self.entries.len() - 1
    }

    pub fn extend_with(&mut self, ir: &IR) {
        let f = self.thompson(ir, 0);
        self.final_states.push(f);
    }

    fn thompson(&mut self, ir: &IR, mut q: usize) -> usize {
        use IR::*;
        match ir {
            E => {
                let f = self.add_state();
                self.entries[q].epsilon.push(f);
                f
            }
            L(x) => {
                let f = self.add_state();
                let a = *x as usize;
                self.entries[q].t[a].push(f);
                f
            }
            U(v) => {
                let fs: Vec<usize> = v
                    .iter()
                    .map(|x| {
                        let s = self.add_state();
                        self.entries[q].epsilon.push(s);
                        self.thompson(x, s)
                    })
                    .collect();
                let ff = self.add_state();
                for f in fs {
                    self.entries[f].epsilon.push(ff);
                }
                ff
            }
            C(v) => {
                for x in v {
                    q = self.thompson(x, q);
                }
                q
            }
            K(x) => {
                let s = self.add_state();
                let f = self.thompson(x, s);
                let ff = self.add_state();
                self.entries[q].epsilon.push(s);
                self.entries[q].epsilon.push(ff);
                self.entries[f].epsilon.push(s);
                self.entries[f].epsilon.push(ff);
                ff
            }
        }
    }

    fn epsilon_closure(&self, mut s: BitSet) -> BitSet {
        loop {
            let t: BitSet = s
                .iter()
                .flat_map(|x| self.entries[x].epsilon.iter().cloned())
                .collect();
            if t.is_subset(&s) {
                return s;
            } else {
                s.union_with(&t);
            }
        }
    }
}

impl Automaton for NFA {
    type State = BitSet;
    fn initial_state(&self) -> Self::State {
        let mut s = BitSet::new();
        s.insert(0);
        self.epsilon_closure(s)
    }
    fn transition(&self, q: &Self::State, x: u8) -> Option<Self::State> {
        Some(
            self.epsilon_closure(
                q.iter()
                    .flat_map(|a| self.entries[a].t[x as usize].iter().cloned())
                    .collect(),
            ),
        )
        .filter(|bs| !bs.is_empty())
    }
    fn category(&self, q: &Self::State) -> Option<Category> {
        self.final_states
            .iter()
            .position(|&f| q.contains(f))
            .map(Category)
    }
}

#[cfg(test)]
mod tests {
    use super::IR::*;
    use super::*;
    use crate::combinator::Parser;

    impl NFAEntry {
        fn with_e(mut self, q: usize) -> Self {
            self.epsilon.push(q);
            self
        }
        fn with_t(mut self, x: u8, q: usize) -> Self {
            self.t[x as usize].push(q);
            self
        }
    }

    // ((a*b)?)
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
    fn nfa_new() {
        let ir = ir_simple();
        let mut nfa = NFA::new();
        nfa.extend_with(&ir);
        let ans = NFA {
            entries: vec![
                NFAEntry::default().with_e(1).with_e(3),
                NFAEntry::default().with_e(2),
                NFAEntry::default().with_e(8),
                NFAEntry::default().with_e(4).with_e(6),
                NFAEntry::default().with_t(b'a', 5),
                NFAEntry::default().with_e(4).with_e(6),
                NFAEntry::default().with_t(b'b', 7),
                NFAEntry::default().with_e(8),
                NFAEntry::default(),
            ],
            final_states: vec![8],
        };
        assert_eq!(nfa, ans);
    }

    #[test]
    fn nfa_accept_simple() {
        let ir = ir_simple();
        let mut nfa = NFA::new();
        nfa.extend_with(&ir);
        assert!(nfa.accept("".as_bytes()));
        assert!(nfa.accept("aaab".as_bytes()));
        assert!(!nfa.accept("c".as_bytes()));
        assert!(!nfa.accept("abab".as_bytes()));
    }

    #[test]
    fn nfa_accept() {
        let ir = ir();
        let mut nfa = NFA::new();
        nfa.extend_with(&ir);
        for x in 0..20 {
            let s = format!("{:b}", x);
            assert_eq!(nfa.accept(s.as_bytes()), x % 3 == 0);
        }
    }
}
