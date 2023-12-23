use std::str::FromStr;

use crate::{automata::IR, combinator::Parser};

use self::parser::ast_regex;

pub mod ir;
pub mod parser;

#[derive(Clone, Debug)]
pub struct ParseRegexError(String);

impl FromStr for IR {
    type Err = ParseRegexError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        ast_regex(10)
            .run(s.as_bytes())
            .and_then(|(ast, t)| Some(IR::new(ast)).filter(|_| t.is_empty()))
            .ok_or_else(|| ParseRegexError(s.to_owned()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        automata::{dfa::DFA, nfa::NFABuilder},
        combinator::Parser,
    };

    fn dfa_from_regex(r: &str) -> Result<DFA, ParseRegexError> {
        let ir = r.parse()?;
        let nfa = NFABuilder::new().ir(&ir).build();
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
        let x = dfa_from_regex(r"(\w)+(\.(\w)+)?@(\w)+\.(\w)+").unwrap();
        assert!(x.accept("xumarcus.sg@gmail.com".as_bytes()));
        assert!(!x.accept("notan.email@com".as_bytes()));
    }

    #[ignore] // TODO: improve compilation for dot
    #[test]
    fn regex_clex() {
        let x = dfa_from_regex("//(.)*\n|/\\*(.|\n)*\\*/").unwrap();
        assert!(x.accept("// a\n".as_bytes()));
        assert!(x.accept("/* a\nb\n */".as_bytes()));
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
