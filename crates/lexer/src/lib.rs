use automata::{dfa::DFA, nfa::NFA, Category};
use combinator::Parser;
use regex::ParseRegexError;

mod automata;
mod combinator;
mod regex;

pub struct Lexer<T> {
    dfa: DFA,
    actions: Vec<Box<dyn Fn(&[u8]) -> T>>,
}

impl<T> Lexer<T> {
    pub fn new<'a>(
        iter: impl Iterator<Item = (&'a str, Box<dyn Fn(&[u8]) -> T>)>,
    ) -> Result<Self, ParseRegexError> {
        let mut nfa = NFA::new();
        let mut actions = Vec::new();
        for (regex, action) in iter {
            let ir = regex.parse()?;
            nfa.extend_with(&ir);
            actions.push(action);
        }
        let dfa = DFA::new(&nfa);
        Ok(Self { dfa, actions })
    }

    pub fn lex<'a>(&self, s: &'a [u8]) -> Option<(T, &'a [u8])> {
        self.dfa.run(s).map(|(c, t)| {
            let Category(i) = c;
            let offset = unsafe {
                t.as_ptr().offset_from(s.as_ptr())
            } as usize;
            let (r, _) = s.split_at(offset);
            (self.actions[i](r), t)
        })
    }
}

#[cfg(test)]
mod tests {
    use std::str::from_utf8;

    use crate::Lexer;

    #[test]
    fn lex_multi_simple() {
        let v: Vec<(&str, Box<dyn Fn(&[u8]) -> usize>)> = vec![
            (r"a", Box::new(|_| 0)),
            (r"b", Box::new(|_| 1)),
        ];
        let lexer = Lexer::new(v.into_iter()).unwrap();
        assert_eq!(lexer.lex("a".as_bytes()).unwrap().0, 0);
        assert_eq!(lexer.lex("b".as_bytes()).unwrap().0, 1);
    }

    #[test]
    fn lex_multi() {
        let v: Vec<(&str, Box<dyn Fn(&[u8]) -> isize>)> = vec![
            (r"\d\d\d", Box::new(|_| 42)),
            (r"(-)?[123456789](\d)+", Box::new(|s| from_utf8(s).unwrap().parse().unwrap())),
            (r"0(\d)+", Box::new(|_| 1)),
        ];
        let lexer = Lexer::new(v.into_iter()).unwrap();
        assert_eq!(lexer.lex("123".as_bytes()).unwrap().0, 42);
        assert_eq!(lexer.lex("1234".as_bytes()).unwrap().0, 1234);
        assert_eq!(lexer.lex("-123".as_bytes()).unwrap().0, -123);
        assert_eq!(lexer.lex("0456".as_bytes()).unwrap().0, 1);
        
    }
}