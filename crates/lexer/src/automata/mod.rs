use super::combinator::Parser;

pub mod dfa;
pub mod nfa;
mod util;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Category(pub usize);

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum IR {
    E,
    L(u8),
    U(Vec<IR>),
    C(Vec<IR>),
    K(Box<IR>),
}

pub trait Automaton {
    type State;
    fn initial_state(&self) -> Self::State;
    fn transition(&self, q: &Self::State, x: u8) -> Option<Self::State>;
    fn category(&self, q: &Self::State) -> Option<Category>;

    fn transition_on<'a, 'b>(
        &'b self,
        q: &'b Self::State,
        s: &'a [u8],
    ) -> Option<(Self::State, &'a [u8])> {
        let (&x, t) = s.split_first()?;
        let z = self.transition(q, x)?;
        Some((z, t))
    }
}

impl<T: Automaton> Parser for T {
    type Item = Category;

    fn run<'a>(&self, mut s: &'a [u8]) -> Option<(Self::Item, &'a [u8])> {
        let mut q = self.initial_state();
        let mut r = self.category(&q).map(|c| (c, s));
        while let Some((z, t)) = self.transition_on(&q, s) {
            q = z;
            r = self.category(&q).map(|c| (c, t));
            s = t;
        }
        r
    }
}
