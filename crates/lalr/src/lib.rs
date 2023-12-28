#![feature(generic_const_exprs)]

use std::ops::Index;

pub trait Finite: Clone + Default {
    const COUNT: usize;
    fn similar(&self, other: &Self) -> bool;
    fn id(&self) -> usize;
}

pub trait Integral: Finite + Copy {}

enum Nonterminal<N: Integral> {
    Entry,
    N(N),
}

enum Terminal<T: Finite> {
    Sentinel,
    T(T),
}

enum Symbol<N: Integral, T: Finite> {
    N(Nonterminal<N>),
    T(Terminal<T>),
}

type Derivation<N: Integral, T: Finite> = Vec<Symbol<N, T>>;
struct Grammar<N: Integral, T: Finite>(Vec<Vec<Derivation<N, T>>>);

/*
    grammar! {
        entry: E;
        E ->
            | E -PLUS T {}
            | T {};
        T ->
            | T -TIMES F {}
            | F {};
        F ->
            | -LBRACK E -RBRACK {}
            | id {};
    }
 */
macro_rules! grammar {

}

struct RuleIndex<N: Integral>(N, usize);
impl<N: Integral, T: Finite> Index<RuleIndex<N>> for Grammar<N, T> {
    type Output = (N, Derivation<N, T>);

    fn index(&self, index: RuleIndex<N>) -> &Self::Output {
        &(index.0, self.0[index.0.id()][index.1])
    }
}

impl<N: Integral, T: Finite> Grammar<N, T> where [(); N::COUNT + T::COUNT]: Sized {
    fn closure(&self, )
}