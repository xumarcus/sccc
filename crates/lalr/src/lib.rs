use core::slice;
use std::{
    collections::{HashMap, HashSet},
    hash::Hash,
};

fn get_two_mut<T, const M: usize>(
    a: &mut [T; M],
    i: usize,
    j: usize,
) -> Option<(&mut T, &mut T)> {
    if i != j {
        unsafe {
            let x = &mut *(a.get_unchecked_mut(i) as *mut _);
            let y = &mut *(a.get_unchecked_mut(j) as *mut _);
            Some((x, y))
        }
    } else {
        None
    }
}

fn get_index_mut<T: Eq>(v: &mut Vec<T>, t: T) -> usize {
    v.iter().position(|x| x == &t).unwrap_or_else(|| {
        v.push(t);
        v.len() - 1
    })
}

/*
    Internals
*/

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
enum _N {
    Entry,
    N(usize),
}
type NotEntry = usize;

impl _N {
    fn unwrap(self) -> NotEntry {
        match self {
            _N::Entry => unreachable!(),
            _N::N(i) => i,
        }
    }
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
enum _T {
    Sentinel,
    T(usize),
}
type NonSentinel = usize;

impl _T {
    fn unwrap(self) -> NonSentinel {
        match self {
            _T::Sentinel => unreachable!(),
            _T::T(i) => i,
        }
    }
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
enum _S {
    N(_N),
    T(_T),
}
const INIT_SYMBOL: _S = _S::N(_N::N(0));

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
struct PID {
    ntl: NotEntry,
    idx: usize,
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
struct LRItem {
    pid: Option<PID>,
    dot: usize,
}
const INIT_ITEM: LRItem = LRItem { pid: None, dot: 0 };

impl From<PID> for LRItem {
    fn from(value: PID) -> Self {
        Self {
            pid: Some(value),
            dot: 0,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct SetOfItems {
    kernel: HashSet<LRItem>,
    nonkernel: HashSet<NotEntry>,
}

impl SetOfItems {
    fn is_empty(&self) -> bool {
        let Self { kernel, nonkernel } = self;
        kernel.is_empty() && nonkernel.is_empty()
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct First(HashSet<NonSentinel>, bool);

#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct Follow(HashSet<_T>);

#[derive(Clone, Debug, PartialEq, Eq)]
struct MapOfItems(HashMap<LRItem, HashSet<Option<_T>>>);

#[derive(Clone, Debug, PartialEq, Eq)]
struct CharacteristicAutomaton<const NC: usize, const TC: usize> {
    collection: Vec<SetOfItems>,
    goto_n: [Option<usize>; NC],
    goto_t: [Option<usize>; TC],
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum Action {
    Shift(usize),
    Reduce(PID),
    Accept,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct LR<const NC: usize, const TC: usize> {
    goto_n: [Option<usize>; NC],
    action: [Option<Action>; TC],
}

fn upper_bound<T: Clone + Eq>(mut s: T, f: impl Fn(&T, T) -> T) -> T {
    loop {
        let t: T = f(&s, s.clone());
        if s == t {
            return s;
        } else {
            s = t;
        }
    }
}

type _D = Vec<_S>;
struct _G<const NC: usize, const TC: usize>([Vec<_D>; NC]);

impl<const NC: usize, const TC: usize> _G<NC, TC> {
    fn symbol(&self, item: &LRItem) -> Option<&_S> {
        match item.pid {
            Some(pid) => self.production(&pid).get(item.dot),
            None => match item.dot {
                0 => Some(&INIT_SYMBOL),
                _ => None,
            },
        }
    }

    fn shifted(&self, item: &LRItem) -> Option<LRItem> {
        self.symbol(item).map(|_| LRItem {
            dot: item.dot + 1,
            pid: item.pid,
        })
    }

    fn pids(&self, ntl: NotEntry) -> impl Iterator<Item = PID> {
        (0..self.0[ntl].len()).map(move |idx| PID { ntl, idx })
    }

    fn production<'a>(&'a self, pid: &PID) -> &'a [_S] {
        &self.0[pid.ntl][pid.idx]
    }

    fn indexed_productions<'a>(
        &'a self,
    ) -> impl Iterator<Item = (PID, &'a [_S])> {
        self.0.iter().enumerate().flat_map(|(ntl, ds)| {
            ds.iter()
                .enumerate()
                .map(move |(idx, d)| (PID { ntl, idx }, d.as_slice()))
        })
    }

    fn from_set<'a>(
        &'a self,
        s: &'a SetOfItems,
    ) -> impl Iterator<Item = LRItem> + 'a {
        s.kernel.iter().copied().chain(
            s.nonkernel
                .iter()
                .flat_map(|&ntl| self.pids(ntl).map(LRItem::from)),
        )
    }

    fn lr0_closure(&self, kernel: HashSet<LRItem>) -> SetOfItems {
        let s = SetOfItems {
            kernel,
            nonkernel: HashSet::new(),
        };
        upper_bound(s, |s, mut t| {
            for x in self.from_set(s) {
                if let Some(_S::N(n)) = self.symbol(&x) {
                    match n {
                        _N::Entry => t.kernel.insert(INIT_ITEM),
                        _N::N(i) => t.nonkernel.insert(*i),
                    };
                }
            }
            t
        })
    }

    fn lr0_goto(&self, s: &SetOfItems, sym: _S) -> SetOfItems {
        let mut kernel = HashSet::new();
        for x in self.from_set(s) {
            if let Some(a) = self.symbol(&x) {
                if sym == *a {
                    kernel.insert(self.shifted(&x).unwrap());
                }
            }
        }
        self.lr0_closure(kernel)
    }

    // stable does not allow [NC + TC]
    fn lr0_characteristic_automaton(&self) -> CharacteristicAutomaton<NC, TC> {
        let kernel = HashSet::from([INIT_ITEM]);
        let mut collection = vec![self.lr0_closure(kernel)];
        let mut goto_n = [None; NC];
        let mut goto_t = [None; TC];
        for i in 0.. {
            if i >= collection.len() {
                break;
            }
            for i in 0..NC {
                let t = self.lr0_goto(&collection[i], _S::N(_N::N(i)));
                if !t.is_empty() {
                    goto_n[i] = Some(get_index_mut(&mut collection, t));
                }
            }
            for i in 0..TC {
                let t = self.lr0_goto(&collection[i], _S::T(_T::T(i)));
                if !t.is_empty() {
                    goto_t[i] = Some(get_index_mut(&mut collection, t));
                }
            }
        }
        CharacteristicAutomaton {
            collection,
            goto_n,
            goto_t,
        }
    }

    fn compute_first(&self) -> [First; NC] {
        let mut out = std::array::from_fn(|_| First::default());
        loop {
            let mut changed = false;
            for (pid, d) in self.indexed_productions() {
                let i = pid.ntl;
                'no_epsilon: {
                    for s in d {
                        match s {
                            _S::N(_N::Entry) => unreachable!(),
                            _S::N(_N::N(j)) => {
                                match get_two_mut(&mut out, i, *j) {
                                    Some((a, b)) => {
                                        for x in &b.0 {
                                            changed |= a.0.insert(*x);
                                        }
                                    }
                                    _ => (),
                                }
                                if !out[*j].1 {
                                    break 'no_epsilon;
                                }
                            }
                            _S::T(_T::Sentinel) => unreachable!(),
                            _S::T(_T::T(j)) => {
                                changed |= out[i].0.insert(*j);
                                break 'no_epsilon;
                            }
                        }
                    }
                    out[i].1 = true;
                }
            }
            if !changed {
                break;
            }
        }
        out
    }

    fn first(firsts: &[First; NC], t: &[_S]) -> First {
        let mut out = First(HashSet::new(), true);
        for sym in t {
            match sym {
                _S::N(_N::Entry) => unreachable!(),
                _S::N(_N::N(j)) => {
                    out.0.extend(&firsts[*j].0);
                    if !firsts[*j].1 {
                        out.1 = false;
                        break;
                    }
                }
                _S::T(_T::Sentinel) => unreachable!(),
                _S::T(_T::T(j)) => {
                    out.0.insert(*j);
                    out.1 = false;
                    break;
                }
            }
        }
        out
    }

    /*
    fn compute_follow(&self, firsts: &[First; NC]) -> [Follow; NC] {
        let mut out = std::array::from_fn(|_| Follow::default());
        out[0].0.insert(_T::Sentinel);
        loop {
            let mut changed = false;
            for (pid, d) in self.indexed_productions() {
                for (k, sym) in d.iter().enumerate() {
                    if let _S::N(_N::N(n)) = sym {
                        let x = Self::first(firsts, &d[(k + 1)..]);
                        for t in x.0 {
                            changed |= out[*n].0.insert(_T::T(t));
                        }
                        if x.1 {
                            let i = pid.ntl;
                            match get_two_mut(&mut out, i, *n) {
                                Some((a, b)) => {
                                    for t in &a.0 {
                                        changed |= b.0.insert(*t);
                                    }
                                }
                                _ => (),
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
     */

    fn lr1_closure(&self, firsts: &[First; NC], s: MapOfItems) -> MapOfItems {
        upper_bound(s, |s, mut t| {
            for (item, lookaheads) in &s.0 {
                let rhs: &[_S] = match item.pid {
                    Some(pid) => &self.production(&pid)[item.dot..],
                    None => slice::from_ref(&INIT_SYMBOL),
                };
                if let Some((_S::N(n), beta)) = rhs.split_first() {
                    let x = Self::first(firsts, beta);
                    for item in self.pids(n.unwrap()).map(LRItem::from) {
                        let u = t.0.entry(item).or_insert_with(HashSet::new);
                        u.extend(x.0.iter().map(|&b| Some(_T::T(b))));
                        if x.1 {
                            u.extend(lookaheads.iter());
                        }
                    }
                }
            }
            t
        })
    }

    fn lr1_goto(&self, firsts: &[First; NC], s: &MapOfItems) -> MapOfItems {
        let t = MapOfItems(
            s.0.iter()
                .filter_map(|(item, lookaheads)| {
                    Some((self.shifted(item)?, lookaheads.clone()))
                })
                .collect(),
        );
        self.lr1_closure(firsts, t)
    }

    fn compute_lookaheads(
        &self,
        firsts: &[First; NC],
        iter: impl Iterator<Item = LRItem>,
    ) -> HashMap<LRItem, HashSet<Option<_T>>> {
        let mut res: HashMap<LRItem, HashSet<Option<_T>>> =
            HashMap::from([(INIT_ITEM, HashSet::from([Some(_T::Sentinel)]))]);
        let mut prg: HashMap<LRItem, HashSet<LRItem>> = HashMap::new();
        for item_a in iter {
            let s = self.lr1_closure(
                firsts,
                MapOfItems(HashMap::from([(item_a, HashSet::from([None]))])),
            );
            for (i, lookaheads) in s.0 {
                if let Some(item_b) = self.shifted(&i) {
                    for x in lookaheads {
                        if x != None {
                            res.entry(item_b)
                                .or_insert_with(HashSet::new)
                                .insert(x);
                        } else {
                            prg.entry(item_a)
                                .or_insert_with(HashSet::new)
                                .insert(item_b);
                        }
                    }
                }
            }
        }
        upper_bound(res, |s, mut t| {
            for (item_a, lookaheads) in s {
                if let Some(items) = prg.get(item_a) {
                    for &item_b in items {
                        t.entry(item_b)
                            .or_insert_with(HashSet::new)
                            .extend(lookaheads.iter());
                    }
                }
            }
            t
        })
    }

    fn lalr(&self) -> LR<NC, TC> {
        let ca = self.lr0_characteristic_automaton();
        let firsts = self.compute_first();
        let m = self.compute_lookaheads(
            &firsts,
            ca.collection.iter().flat_map(|s| s.kernel.iter()).copied(),
        );
        
    }
}

#[cfg(test)]
mod tests {
    use std::collections::{HashMap, HashSet};

    use crate::{LRItem, MapOfItems, INIT_ITEM, PID, _G, _N, _S, _T};

    fn grammar_slr() -> _G<3, 5> {
        let e = _S::N(_N::N(0));
        let t = _S::N(_N::N(1));
        let f = _S::N(_N::N(2));

        let add = _S::T(_T::T(0));
        let mul = _S::T(_T::T(1));
        let lb = _S::T(_T::T(2));
        let rb = _S::T(_T::T(3));
        let id = _S::T(_T::T(4));

        _G([
            vec![vec![e, add, t], vec![t]],
            vec![vec![t, mul, f], vec![f]],
            vec![vec![lb, e, rb], vec![id]],
        ])
    }

    fn grammar_not_slr() -> _G<3, 3> {
        let s = _S::N(_N::N(0));
        let l = _S::N(_N::N(1));
        let r = _S::N(_N::N(2));

        let eq = _S::T(_T::T(0));
        let star = _S::T(_T::T(1));
        let id = _S::T(_T::T(2));

        _G([
            vec![vec![l, eq, r], vec![r]],
            vec![vec![star, r], vec![id]],
            vec![vec![l]],
        ])
    }

    fn grammar_simple() -> _G<2, 2> {
        let s = _S::N(_N::N(0));
        let c = _S::N(_N::N(1));

        let _c = _S::T(_T::T(0));
        let _d = _S::T(_T::T(1));

        _G([vec![vec![c, c]], vec![vec![_c, c], vec![_d]]])
    }

    #[test]
    fn test_goto() {
        let g = grammar_slr();
        let kernel = HashSet::from([INIT_ITEM]);
        let s = g.lr0_closure(kernel);
        let t = g.lr0_goto(&s, _S::N(_N::N(1)));
        assert_eq!(
            t.kernel,
            HashSet::from([
                LRItem {
                    pid: Some(PID { ntl: 1, idx: 0 }),
                    dot: 1
                },
                LRItem {
                    pid: Some(PID { ntl: 0, idx: 1 }),
                    dot: 1
                }
            ])
        );
    }

    #[test]
    fn test_lr1_closure() {
        let g = grammar_simple();
        let s = MapOfItems(HashMap::from([(
            INIT_ITEM,
            HashSet::from([Some(_T::Sentinel)]),
        )]));
        let firsts = g.compute_first();
        let t = g.lr1_closure(&firsts, s);
        assert_eq!(
            t,
            MapOfItems(HashMap::from([
                (
                    LRItem {
                        pid: Some(PID { ntl: 1, idx: 1 },),
                        dot: 0,
                    },
                    HashSet::from([Some(_T::T(1,),), Some(_T::T(0,),),])
                ),
                (
                    LRItem {
                        pid: Some(PID { ntl: 0, idx: 0 },),
                        dot: 0,
                    },
                    HashSet::from([Some(_T::Sentinel,)]),
                ),
                (
                    LRItem {
                        pid: Some(PID { ntl: 1, idx: 0 },),
                        dot: 0,
                    },
                    HashSet::from([Some(_T::T(1,),), Some(_T::T(0,),),]),
                ),
                (
                    LRItem { pid: None, dot: 0 },
                    HashSet::from([Some(_T::Sentinel,)]),
                ),
            ]))
        )
    }

    #[test]
    fn test_compute_lookaheads() {
        let g = grammar_not_slr();
        let v = g.lr0_characteristic_automaton();
        let firsts = g.compute_first();
        let m = g.compute_lookaheads(
            &firsts,
            v.iter().flat_map(|s| s.kernel.iter().copied()),
        );
        assert_eq!(m.len(), 10);
    }
}
