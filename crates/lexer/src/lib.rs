use automata::{dfa::DFA, nfa::NFABuilder, Category, ParserAutomaton};
use combinator::Parser;

mod automata;
pub mod combinator;
mod regex;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ParseRegexError(pub String);
pub type Result<T> = core::result::Result<T, ParseRegexError>;
pub type Action<T> = Box<dyn Fn(&[u8]) -> T>;

pub struct Lexer<T> {
    parser: ParserAutomaton<DFA>,
    actions: Vec<Action<T>>,
}

impl<T> Lexer<T> {
    pub fn new<'a>(
        iter: impl Iterator<Item = (&'a str, Action<T>)>,
    ) -> Result<Self> {
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

impl<T> Parser for Lexer<T> {
    type Item = T;
    fn run<'a>(&self, s: &'a [u8]) -> Option<(T, &'a [u8])> {
        self.parser.run(s).map(|(c, t)| {
            let Category(i) = c;
            let offset = unsafe { t.as_ptr().offset_from(s.as_ptr()) } as usize;
            let (r, _) = s.split_at(offset);
            (self.actions[i](r), t)
        })
    }
}

#[cfg(test)]
mod tests {
    use std::str::from_utf8;

    use crate::{combinator::Parser, Action, Lexer};

    #[test]
    fn lex_multi_simple() {
        let v: Vec<(&str, Action<usize>)> =
            vec![(r"a", Box::new(|_| 0)), (r"b", Box::new(|_| 1))];
        let lexer = Lexer::new(v.into_iter()).unwrap();
        assert_eq!(lexer.run("a".as_bytes()).unwrap().0, 0);
        assert_eq!(lexer.run("b".as_bytes()).unwrap().0, 1);
    }

    #[test]
    fn lex_multi() {
        let v: Vec<(&str, Action<isize>)> = vec![
            (r"\d\d\d", Box::new(|_| 42)),
            (
                r"(-)?[123456789](\d)+",
                Box::new(|s| from_utf8(s).unwrap().parse().unwrap()),
            ),
            (r"0(\d)+", Box::new(|_| 1)),
        ];
        let lexer = Lexer::new(v.into_iter()).unwrap();
        assert_eq!(lexer.run("123".as_bytes()).unwrap().0, 42);
        assert_eq!(lexer.run("1234".as_bytes()).unwrap().0, 1234);
        assert_eq!(lexer.run("-123".as_bytes()).unwrap().0, -123);
        assert_eq!(lexer.run("0456".as_bytes()).unwrap().0, 1);
        assert_eq!(lexer.run("123a".as_bytes()).unwrap().1[0], b'a');
        assert_eq!(lexer.run("1234a".as_bytes()).unwrap().1[0], b'a');
        assert_eq!(lexer.run("-123a".as_bytes()).unwrap().1[0], b'a');
        assert_eq!(lexer.run("0456a".as_bytes()).unwrap().1[0], b'a');
    }
}
