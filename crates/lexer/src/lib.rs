use automata::{dfa::DFA, nfa::NFABuilder, Category, ParserAutomaton};
use combinator::Parser;

mod automata;
pub mod combinator;
mod regex;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ParseRegexError(pub String);

pub enum Action<T, E> {
    C(T),
    F(fn(&[u8]) -> Result<T, E>),
}

pub struct Lexer<T, E> {
    parser: ParserAutomaton<DFA>,
    actions: Vec<Action<T, E>>,
}

impl<T, E> Lexer<T, E> {
    pub fn new<'a>(
        iter: impl Iterator<Item = (&'a str, Action<T, E>)>,
    ) -> Result<Self, ParseRegexError> {
        let mut builder = NFABuilder::new();
        let mut actions = Vec::new();
        for (regex, action) in iter {
            let ir = regex.parse()?;
            builder.add_ir(&ir);
            actions.push(action);
        }
        let nfa = builder.build();
        let dfa = DFA::new(&nfa);
        let parser = ParserAutomaton(dfa);
        Ok(Self { parser, actions })
    }
}

impl<T: Clone, E> Parser for Lexer<T, E> {
    type Item = Result<T, E>;
    fn run<'a>(&self, s: &'a [u8]) -> Option<(Self::Item, &'a [u8])> {
        use Action::*;
        self.parser.run(s).map(|(c, t)| {
            let Category(i) = c;
            let offset = unsafe { t.as_ptr().offset_from(s.as_ptr()) } as usize;
            let r = match &self.actions[i] {
                C(x) => Ok(x.clone()),
                F(f) => f(s.split_at(offset).0),
            };
            (r, t)
        })
    }
}

#[cfg(test)]
mod tests {
    use std::str::from_utf8;

    use crate::{combinator::Parser, Action, Lexer};

    fn from_bytes(s: &[u8]) -> Result<isize, ()> {
        Ok(from_utf8(s).unwrap().parse().unwrap())
    }

    #[test]
    fn lex_multi_simple() {
        let v: Vec<(&str, Action<usize, ()>)> =
            vec![(r"a", Action::C(0)), (r"b", Action::C(1))];
        let lexer = Lexer::new(v.into_iter()).unwrap();
        assert_eq!(lexer.run("a".as_bytes()).unwrap().0.unwrap(), 0);
        assert_eq!(lexer.run("b".as_bytes()).unwrap().0.unwrap(), 1);
    }

    #[test]
    fn lex_multi() {
        let v: Vec<(&str, Action<isize, ()>)> = vec![
            (r"\d\d\d", Action::C(42)),
            (r"(\-)?[1-9](\d)+", Action::F(from_bytes)),
            (r"0(\d)+", Action::C(1)),
        ];
        let lexer = Lexer::new(v.into_iter()).unwrap();
        assert_eq!(lexer.run("123".as_bytes()).unwrap().0.unwrap(), 42);
        assert_eq!(lexer.run("1234".as_bytes()).unwrap().0.unwrap(), 1234);
        assert_eq!(lexer.run("-123".as_bytes()).unwrap().0.unwrap(), -123);
        assert_eq!(lexer.run("0456".as_bytes()).unwrap().0.unwrap(), 1);
        assert_eq!(lexer.run("123a".as_bytes()).unwrap().1[0], b'a');
        assert_eq!(lexer.run("1234a".as_bytes()).unwrap().1[0], b'a');
        assert_eq!(lexer.run("-123a".as_bytes()).unwrap().1[0], b'a');
        assert_eq!(lexer.run("0456a".as_bytes()).unwrap().1[0], b'a');
    }
}
