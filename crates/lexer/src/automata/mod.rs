use super::combinator::Parser;

pub(super) mod dfa;
pub(super) mod nfa;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Category(pub(crate) usize);

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum IR {
    E,
    L(Vec<u8>),
    U(Vec<IR>),
    C(Vec<IR>),
    K(Box<IR>),
}

pub(crate) trait Automaton {
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

pub(crate) struct ParserAutomaton<T: Automaton>(pub(crate) T);

impl<T: Automaton> Parser for ParserAutomaton<T> {
    type Item = Category;

    fn run<'a>(&self, mut s: &'a [u8]) -> Option<(Self::Item, &'a [u8])> {
        let mut q = self.0.initial_state();
        let mut r = self.0.category(&q).map(|c| (c, s));
        while let Some((z, t)) = self.0.transition_on(&q, s) {
            q = z;
            r = self.0.category(&q).map(|c| (c, t));
            s = t;
        }
        r
    }
}

pub const SIGMA: usize = 256;
