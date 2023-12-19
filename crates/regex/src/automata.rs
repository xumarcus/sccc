use std::{
    fmt::{self, Debug, Formatter},
    mem::MaybeUninit,
};

use bit_set::{self, BitSet};

use super::ir::IR;

pub trait Automaton {
    fn accept(&self, b: &[u8]) -> bool;
}

#[derive(Clone, PartialEq, Eq)]
struct NFAEntry {
    epsilon: Vec<usize>,
    t: [Vec<usize>; 256],
}

impl Default for NFAEntry {
    fn default() -> Self {
        unsafe {
            let mut t: [MaybeUninit<Vec<usize>>; 256] =
                MaybeUninit::uninit().assume_init();
            for x in &mut t {
                *x = MaybeUninit::new(Vec::new());
            }
            Self {
                epsilon: Vec::new(),
                t: std::mem::transmute_copy(&t),
            }
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
struct NFA {
    tbl: Vec<NFAEntry>,
}

impl NFA {
    pub fn new(ir: &IR) -> Self {
        let mut nfa = Self {
            tbl: vec![NFAEntry::default()],
        };
        let _ = nfa.thompson(ir, 0);
        nfa
    }

    fn new_entry(&mut self) -> usize {
        self.tbl.push(NFAEntry::default());
        self.tbl.len() - 1
    }

    fn thompson(&mut self, ir: &IR, q: usize) -> usize {
        use IR::*;
        match ir {
            E => {
                let f = self.new_entry();
                self.tbl[q].epsilon.push(f);
                f
            }
            L(x) => {
                let f = self.new_entry();
                let a = *x as usize;
                self.tbl[q].t[a].push(f);
                f
            }
            U(a, b) => {
                let sa = self.new_entry();
                let fa = self.thompson(a, sa);
                let sb = self.new_entry();
                let fb = self.thompson(b, sb);
                let f = self.new_entry();
                self.tbl[q].epsilon.push(sa);
                self.tbl[q].epsilon.push(sb);
                self.tbl[fa].epsilon.push(f);
                self.tbl[fb].epsilon.push(f);
                f
            }
            C(a, b) => {
                let fa = self.thompson(a, q);
                let fb = self.thompson(b, fa);
                fb
            }
            K(a) => {
                let sa = self.new_entry();
                let fa = self.thompson(a, sa);
                let f = self.new_entry();
                self.tbl[q].epsilon.push(sa);
                self.tbl[q].epsilon.push(f);
                self.tbl[fa].epsilon.push(sa);
                self.tbl[fa].epsilon.push(f);
                f
            }
        }
    }

    fn epsilon_closure(&self, mut s: BitSet) -> BitSet {
        loop {
            let t: BitSet = s
                .iter()
                .flat_map(|x| self.tbl[x].epsilon.iter().cloned())
                .collect();
            if t.is_subset(&s) {
                return s;
            } else {
                s.union_with(&t);
            }
        }
    }

    fn initial_states(&self) -> BitSet {
        let mut s = BitSet::new();
        s.insert(0);
        self.epsilon_closure(s)
    }

    fn transition(&self, s: &BitSet, x: u8) -> BitSet {
        let a = x as usize;
        self.epsilon_closure(
            s.iter()
                .flat_map(|x| self.tbl[x].t[a].iter().cloned())
                .collect(),
        )
    }

    fn final_state(&self) -> usize {
        self.tbl.len() - 1
    }

    fn reversal(&self) -> Self {
        let n = self.tbl.len();
        let mut tbl: Vec<NFAEntry> = vec![NFAEntry::default(); n];
        for (i, e) in self.tbl.iter().enumerate() {
            for &j in &e.epsilon {
                tbl[n - 1 - j].epsilon.push(n - 1 - i);
            }
            for (x, v) in e.t.iter().enumerate() {
                for &j in v {
                    tbl[n - 1 - j].t[x].push(n - 1 - i);
                }
            }
        }
        Self { tbl }
    }

    fn determinization(&self) -> DFA {
        let s = self.initial_states();
        let mut tbl: Vec<DFAEntry> =
            vec![DFAEntry::default()
                .with_accepting(s.contains(self.final_state()))];
        let mut v = vec![s];
        for i in 0.. {
            if i >= v.len() {
                break;
            }
            for x in 0..=255u8 {
                let a = x as usize;
                let u = self.transition(&v[i], x);
                if !u.is_empty() {
                    let j =
                        v.iter().position(|w| w == &u).unwrap_or_else(|| {
                            tbl.push(DFAEntry::default().with_accepting(
                                u.contains(self.final_state()),
                            ));
                            v.push(u);
                            v.len() - 1
                        });
                    tbl[i].t[a] = Some(j);
                }
            }
        }
        DFA { tbl }
    }
}

impl From<DFA> for NFA {
    fn from(dfa: DFA) -> Self {
        let f = dfa.tbl.len();
        let mut tbl: Vec<NFAEntry> = vec![NFAEntry::default(); f + 1];
        for (i, e) in dfa.tbl.iter().enumerate() {
            for x in 0..=255 {
                if e.accepting {
                    tbl[i].epsilon.push(f);
                }
                if let Some(j) = e.t[x] {
                    tbl[i].t[x].push(j);
                }
            }
        }
        Self { tbl }
    }
}

impl Automaton for NFA {
    fn accept(&self, b: &[u8]) -> bool {
        let mut s = self.initial_states();
        for &x in b {
            s = self.transition(&s, x);
        }
        s.contains(self.final_state())
    }
}

#[derive(Clone, Copy, Debug)]
struct DFAEntry {
    accepting: bool,
    t: [Option<usize>; 256],
}

impl DFAEntry {
    fn with_accepting(self, accepting: bool) -> Self {
        Self {
            accepting,
            t: self.t,
        }
    }
}

impl Default for DFAEntry {
    fn default() -> Self {
        Self {
            accepting: false,
            t: [None; 256],
        }
    }
}

struct DFA {
    tbl: Vec<DFAEntry>,
}

impl DFA {
    pub fn new(ir: &IR) -> Self {
        Self::from(NFA::new(ir))
    }
}

impl From<NFA> for DFA {
    fn from(nfa: NFA) -> Self {
        let nr: NFA = nfa.reversal().determinization().into();
        nr.reversal().determinization()
    }
}

impl Automaton for DFA {
    fn accept(&self, b: &[u8]) -> bool {
        let mut s = 0;
        for &x in b {
            let a = x as usize;
            if let Some(t) = self.tbl[s].t[a] {
                s = t;
            } else {
                return false;
            }
        }
        self.tbl[s].accepting
    }
}

#[cfg(test)]
mod tests {
    use super::IR::*;
    use super::*;

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
        let a = Box::new(L(b'a'));
        let b = Box::new(L(b'b'));
        let c = Box::new(C(Box::new(K(a)), b));
        U(Box::new(E), c)
    }

    fn ir() -> IR {
        K(Box::new(U(
            Box::new(L(b'0')),
            Box::new(K(Box::new(C(
                Box::new(L(b'1')),
                Box::new(C(
                    Box::new(K(Box::new(C(
                        Box::new(L(b'0')),
                        Box::new(C(
                            Box::new(C(
                                Box::new(K(Box::new(L(b'1')))),
                                Box::new(K(Box::new(C(
                                    Box::new(L(b'0')),
                                    Box::new(L(b'0')),
                                )))),
                            )),
                            Box::new(L(b'0')),
                        )),
                    )))),
                    Box::new(L(b'1')),
                )),
            )))),
        )))
    }

    #[test]
    fn nfa_new() {
        let ir = ir_simple();
        let nfa = NFA::new(&ir);
        let ans = NFA {
            tbl: vec![
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
        };
        assert_eq!(nfa, ans);
    }

    #[test]
    fn nfa_accept_simple() {
        let ir = ir_simple();
        let nfa = NFA::new(&ir);
        assert!(nfa.accept("".as_bytes()));
        assert!(nfa.accept("aaab".as_bytes()));
        assert!(!nfa.accept("c".as_bytes()));
        assert!(!nfa.accept("abab".as_bytes()));
    }

    #[test]
    fn nfa_accept() {
        let ir = ir();
        let nfa = NFA::new(&ir);
        for x in 0..20 {
            let s = format!("{:b}", x);
            assert_eq!(nfa.accept(s.as_bytes()), x % 3 == 0);
        }
    }

    #[test]
    fn nfa_determinization_simple() {
        let ir = ir_simple();
        let dfa = NFA::new(&ir).determinization();
        assert_eq!(dfa.tbl.len(), 3);
        assert!(dfa.accept("".as_bytes()));
        assert!(dfa.accept("aaab".as_bytes()));
        assert!(!dfa.accept("c".as_bytes()));
        assert!(!dfa.accept("abab".as_bytes()));
    }

    #[test]
    fn nfa_determinization() {
        let ir = ir();
        let dfa = NFA::new(&ir).determinization();
        assert_eq!(dfa.tbl.len(), 8);
        for x in 0..20 {
            let s = format!("{:b}", x);
            assert_eq!(dfa.accept(s.as_bytes()), x % 3 == 0);
        }
    }

    #[test]
    fn nfa_reversal() {
        let ir = ir_simple();
        let nfa = NFA::new(&ir).reversal();
        assert!(nfa.accept("".as_bytes()));
        assert!(nfa.accept("baaa".as_bytes()));
        assert!(!nfa.accept("c".as_bytes()));
        assert!(!nfa.accept("baba".as_bytes()));
    }

    #[test]
    fn nfa_from_dfa() {
        let ir = ir_simple();
        let nfa = NFA::from(NFA::new(&ir).determinization());
        assert!(nfa.accept("".as_bytes()));
        assert!(nfa.accept("aaab".as_bytes()));
        assert!(!nfa.accept("c".as_bytes()));
        assert!(!nfa.accept("abab".as_bytes()));
    }

    #[test]
    fn dfa_accept_simple() {
        let ir = ir_simple();
        let dfa = DFA::new(&ir);
        assert_eq!(dfa.tbl.len(), 3);
        assert!(dfa.accept("".as_bytes()));
        assert!(dfa.accept("aaab".as_bytes()));
        assert!(!dfa.accept("c".as_bytes()));
        assert!(!dfa.accept("abab".as_bytes()));
    }

    #[test]
    fn dfa_accept() {
        let ir = ir();
        let dfa = DFA::new(&ir);
        assert_eq!(dfa.tbl.len(), 4);
        for x in 0..20 {
            let s = format!("{:b}", x);
            assert_eq!(dfa.accept(s.as_bytes()), x % 3 == 0);
        }
    }
}
