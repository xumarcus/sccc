use std::{collections::HashSet, mem::MaybeUninit};

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
struct Ntl(usize);
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
struct Tml(usize);

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub enum Symbol {
    N(Ntl),
    T(Tml),
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
struct LR0Item {
    ntl: Ntl,
    pid: usize,
    dot: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct SI(HashSet<LR0Item>, HashSet<Ntl>);
impl SI {
    fn new() -> Self {
        Self(HashSet::new(), HashSet::new())
    }

    fn unclosured() -> Self {
        Self(HashSet::new(), HashSet::from([Ntl(0)]))
    }

    fn is_empty(&self) -> bool {
        self.0.is_empty() && self.1.is_empty()
    }
}

type Derivation = Vec<Symbol>;
type Grammar = Vec<Vec<Derivation>>;

#[derive(Clone, Debug, PartialEq, Eq)]
struct Production(Ntl, Derivation);

fn symbol<'a>(p: &'a [Vec<Derivation>], x: &LR0Item) -> Option<&'a Symbol> {
    let LR0Item { ntl, pid, dot } = x;
    p[ntl.0][*pid].get(*dot)
}

fn closure(p: &[Vec<Derivation>], mut s: SI) -> SI {
    loop {
        let mut t = s.clone();
        for x in &s.0 {
            if let Some(Symbol::N(ntl)) = symbol(p, x) {
                t.1.insert(*ntl);
            }
        }
        for x in &s.1 {
            for d in &p[x.0] {
                if let Some(Symbol::N(ntl)) = d.first() {
                    t.1.insert(*ntl);
                }
            }
        }
        if s == t {
            return s;
        } else {
            s = t;
        }
    }
}

fn goto(p: &[Vec<Derivation>], s: &SI, sym: &Symbol) -> SI {
    let mut t = SI::new();
    for x in &s.0 {
        if let Some(a) = symbol(p, x) {
            if sym == a {
                t.0.insert(LR0Item {
                    dot: x.dot + 1,
                    ..*x
                });
            }
        }
    }
    for x in &s.1 {
        for (pid, d) in p[x.0].iter().enumerate() {
            if let Some(a) = d.first() {
                if sym == a {
                    t.0.insert(LR0Item {
                        ntl: *x,
                        pid,
                        dot: 1,
                    });
                }
            }
        }
    }
    closure(p, t)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Action {
    Shift(usize),
    Reduce(Ntl, usize),
    Accept,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct LREntry<const N: usize, const T: usize> {
    actions: [Option<Action>; T],
    gotos: [Option<usize>; N],
}

impl<const N: usize, const T: usize> Default for LREntry<N, T> {
    fn default() -> Self {
        Self {
            actions: [None; T],
            gotos: [None; N],
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct LR<const N: usize, const T: usize>(Vec<LREntry<N, T>>);

fn run<const N: usize, const T: usize>(lr: &LR<N, T>, s: &[Tml]) -> bool {
    use Action::*;
    let mut st = vec![0];
    for x in s {
        loop {
            let q = st.last().unwrap();
            match lr.0[*q].actions[x.0] {
                Some(Shift(r)) => {
                    st.push(r);
                    break;
                }
                Some(Reduce(n, l)) => {
                    st.truncate(st.len().saturating_sub(l));
                    let q = st.last().unwrap();
                    let e = &lr.0[*q];
                    st.push(e.gotos[n.0].unwrap());
                }
                Some(Accept) => return true,
                None => return false,
            }
        }
    }
    false
}

type CharacteristicAutomaton<const T: usize> = Vec<(SI, [Option<usize>; T])>;
fn initialize<const N: usize, const T: usize>(
    p: &[Vec<Derivation>],
) -> (LR<N, T>, CharacteristicAutomaton<T>) {
    let mut lr = vec![LREntry::default()];
    let mut ca = vec![(closure(p, SI::unclosured()), [None; T])];
    for i in 0.. {
        if i >= ca.len() {
            break;
        }
        for x in 0..N {
            let t = goto(p, &ca[i].0, &Symbol::N(Ntl(x)));
            if !t.is_empty() {
                let j =
                    ca.iter().position(|(u, _)| u == &t).unwrap_or_else(|| {
                        lr.push(LREntry::default());
                        ca.push((t, [None; T]));
                        ca.len() - 1
                    });
                lr[i].gotos[x] = Some(j);
            }
        }
        for x in 0..T {
            let t = goto(p, &ca[i].0, &Symbol::T(Tml(x)));
            if !t.is_empty() {
                let j =
                    ca.iter().position(|(u, _)| u == &t).unwrap_or_else(|| {
                        lr.push(LREntry::default());
                        ca.push((t, [None; T]));
                        ca.len() - 1
                    });
                ca[i].1[x] = Some(j);
            }
        }
    }
    println!("CA {} {}", lr.len(), ca.len());
    (LR(lr), ca)
}

#[derive(Clone, Default, Debug, PartialEq, Eq)]
struct First(HashSet<Tml>, bool);

fn compute_first<const N: usize>(p: &[Vec<Derivation>]) -> [First; N] {
    let mut out = std::array::from_fn(|_| First::default());
    loop {
        let mut changed = false;
        for (i, rs) in p.iter().enumerate() {
            for r in rs {
                'no_epsilon: {
                    for sym in r {
                        match sym {
                            Symbol::N(Ntl(j)) => {
                                if i != *j {
                                    // get_many_mut
                                    for x in out[*j].0.clone() {
                                        changed |= out[i].0.insert(x);
                                    }
                                }
                                if !out[*j].1 {
                                    break 'no_epsilon;
                                }
                            }
                            Symbol::T(Tml(j)) => {
                                changed |= out[i].0.insert(Tml(*j));
                                break 'no_epsilon;
                            }
                        }
                    }
                    out[i].1 = true;
                }
            }
        }
        if !changed {
            break;
        }
    }
    out
}

fn first_of_t<const N: usize>(first: &[First; N], t: &[Symbol]) -> First {
    let mut out = First::default();
    out.1 = true;
    for sym in t {
        match sym {
            Symbol::N(Ntl(j)) => {
                out.0.extend(&first[*j].0);
                if !first[*j].1 {
                    out.1 = false;
                    break;
                }
            }
            Symbol::T(Tml(j)) => {
                out.0.insert(Tml(*j));
                out.1 = false;
                break;
            }
        }
    }
    out
}

fn compute_follow<const N: usize>(
    p: &[Vec<Derivation>],
    first: &[First; N],
) -> [HashSet<Tml>; N] {
    let mut out = std::array::from_fn(|_| HashSet::new());
    out[0].insert(Tml(0));
    loop {
        let mut changed = false;
        for (i, rs) in p.iter().enumerate() {
            for r in rs {
                for (j, sym) in r.iter().enumerate() {
                    if let Symbol::N(Ntl(k)) = sym {
                        let x = first_of_t(first, &r[(j + 1)..]);
                        for t in x.0 {
                            changed |= out[*k].insert(t);
                        }
                        if x.1 {
                            // get many mut?
                            for t in out[i].clone() {
                                changed |= out[*k].insert(t);
                            }
                        }
                    }
                }
            }
        }
        if !changed {
            break;
        }
    }
    out
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct Conflict(Action, Action);

fn set_action<const N: usize, const T: usize>(
    e: &mut LREntry<N, T>,
    t: &Tml,
    a: Action,
) -> Result<(), Conflict> {
    match e.actions[t.0].replace(a) {
        Some(x) => Err(Conflict(x, a)),
        None => Ok(()),
    }
}

fn slr<const N: usize, const T: usize>(
    p: &[Vec<Derivation>],
    follow: &[HashSet<Tml>; N],
) -> Result<LR<N, T>, Conflict> {
    use Action::*;
    let (mut lr, ca) = initialize::<N, T>(p);
    for (i, (s, ts)) in ca.iter().enumerate() {
        for x in &s.0 {
            match symbol(p, x) {
                Some(Symbol::T(tml)) => {
                    if let Some(j) = ts[tml.0] {
                        set_action(&mut lr.0[i], tml, Shift(j))?;
                    }
                }
                None => {
                    if x.ntl == Ntl(0) {
                        set_action(&mut lr.0[i], &Tml(0), Accept)?;
                    } else {
                        for a in &follow[x.ntl.0] {
                            set_action(
                                &mut lr.0[i],
                                &a,
                                Reduce(x.ntl, p[x.ntl.0][x.pid].len()),
                            )?;
                        }
                    }
                }
                _ => (),
            }
        }
        for x in &s.1 {
            for d in &p[x.0] {
                match d.first() {
                    Some(Symbol::T(tml)) => {
                        if let Some(j) = ts[tml.0] {
                            set_action(&mut lr.0[i], tml, Shift(j))?;
                        }
                    }
                    None => {
                        if x == &Ntl(0) {
                            set_action(&mut lr.0[i], &Tml(0), Accept)?;
                        } else {
                            for a in &follow[x.0] {
                                set_action(&mut lr.0[i], &a, Reduce(*x, d.len()))?;
                            }
                        }
                    }
                    _ => (),
                }
            }
        }
    }
    Ok(lr)
}

#[cfg(test)]
mod tests {
    use crate::{
        closure, compute_first, compute_follow, goto, run, slr, Derivation,
        Ntl, Symbol::*, Tml, SI,
    };

    /*
       E' -> E
       E -> E + T | T
       T -> T * F | F
       F -> (E) | id
    */
    /*
       grammar! {
           E' -> E;
           E -> E O(PLUS) T | T;

           F -> O(LBRACK) E O(RBRACK) | Id;
       }
    */
    fn grammar_slr() -> [Vec<Derivation>; 4] {
        [
            vec![vec![N(Ntl(1))]],
            vec![vec![N(Ntl(1)), T(Tml(1)), N(Ntl(2))], vec![N(Ntl(2))]],
            vec![vec![N(Ntl(2)), T(Tml(2)), N(Ntl(3))], vec![N(Ntl(3))]],
            vec![vec![T(Tml(3)), N(Ntl(1)), T(Tml(4))], vec![T(Tml(5))]],
        ]
    }

    fn grammar_428() -> [Vec<Derivation>; 6] {
        [
            /* S */ vec![vec![N(Ntl(1))]],
            /* E */ vec![vec![N(Ntl(3)), N(Ntl(2))]],
            /* E'*/ vec![vec![T(Tml(1)), N(Ntl(3)), N(Ntl(2))], vec![]],
            /* T */ vec![vec![N(Ntl(5)), N(Ntl(4))]],
            /* T'*/ vec![vec![T(Tml(2)), N(Ntl(5)), N(Ntl(4))], vec![]],
            /* F */
            vec![vec![T(Tml(3)), N(Ntl(1)), T(Tml(4))], vec![T(Tml(5))]],
        ]
    }

    #[test]
    fn test_goto() {
        let p = grammar_slr();
        let s = closure(&p, SI::unclosured());
        println!("s {:#?}", s);
        let i4 = goto(&p, &s, &T(Tml(3)));
        println!("i4 {:#?}", i4);
    }

    #[test]
    fn test_slr() {
        let p = grammar_slr();
        let first = compute_first::<4>(&p);
        let follow = compute_follow(&p, &first);
        let lr = slr::<4, 6>(&p, &follow).unwrap();

        // (id + id) * id
        assert!(run(
            &lr,
            &[Tml(3), Tml(5), Tml(1), Tml(5), Tml(4), Tml(2), Tml(5), Tml(0)]
        ));
    }

    #[test]
    fn test_428() {
        let p = grammar_428();
        let first = compute_first::<6>(&p);
        println!("FIRST {:#?}", &first);
        let follow = compute_follow::<6>(&p, &first);
        println!("FOLLOW {:#?}", &follow);
    }
}
