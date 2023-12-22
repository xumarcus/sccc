use std::str::FromStr;

use crate::{automata::IR, combinator::Parser};

use self::parser::ast_regex;

pub mod ir;
pub mod parser;

#[derive(Clone, Copy, Debug)]
pub struct ParseRegexError;

impl FromStr for IR {
    type Err = ParseRegexError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (ast, t) = ast_regex(10).run(s.as_bytes()).ok_or(ParseRegexError)?;
        if t.is_empty() {
            Ok(IR::new(ast))
        } else {
            Err(ParseRegexError)
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{automata::{dfa::DFA, nfa::NFA}, combinator::Parser};
    use super::*;

    fn dfa_from_regex(r: &str) -> Result<DFA, ParseRegexError> {
        let ir = r.parse()?;
        let mut nfa = NFA::new();
        nfa.extend_with(&ir);
        let dfa = DFA::new(&nfa);
        Ok(dfa)
    }

    #[test]
    fn regex_new() {
        assert!(dfa_from_regex(r"\w?").is_err());
        assert!(dfa_from_regex(r"a(b(c(d)+)*)?(e(f)?(g)?)*").is_ok());
    }

    #[test]
    fn regex_email() {
        let x = dfa_from_regex(r"(\w)+(.(\w)+)?@(\w)+.(\w)+").unwrap();
        assert!(x.accept("xumarcus.sg@gmail.com".as_bytes()));
        assert!(!x.accept("notan.email@com".as_bytes()));
    }

    #[test]
    fn regex_python() {
        let data = [
            (r"abc", "abc", true),
            (r"abc", "xbc", false),
            (r"abc", "axc", false),
            (r"abc", "abx", false),
            (r"(\w)?abc(\w)?", "xabcy", true),
            (r"(\w)?(\w)?abc", "ababc", true),
            (r"a(b)*c", "abc", true),
            (r"a(b)*bc", "abc", true),
            (r"a(b)*bc", "abbc", true),
            (r"a(b)*bc", "abbbbc", true),
            (r"a(b)+bc", "abbc", true),
            (r"a(b)+bc", "abc", false),
            (r"a(b)+bc", "abq", false),
            (r"a(b)+bc", "abbbbc", true),
            (r"a(b)?bc", "abbc", true),
            (r"a(b)?bc", "abc", true),
            (r"a(b)?bc", "abbbbc", false),
            (r"a(b)?c", "abc", true),
            (r"a[bc]d", "abc", false),
            (r"a[bc]d", "abd", true),
        ];
        for (r, s, b) in data {
            let x = dfa_from_regex(r).unwrap();
            assert_eq!(x.accept(s.as_bytes()), b, "{} {} {}", r, s, b);
        }
    }
}
