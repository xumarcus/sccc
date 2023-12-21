use automata::{Automaton, DFA};
use combinator::Parser;
use ir::IR;
use parser::ast_regex;

mod automata;
mod ir;
mod parser;

pub struct Regex(DFA);

impl Regex {
    pub fn new(r: &[u8]) -> Option<Self> {
        let (ast, _) = ast_regex(10).run(r).filter(|(_, t)| t.is_empty())?;
        let ir = IR::new(ast);
        let dfa = DFA::new(&ir);
        Some(Regex(dfa))
    }

    pub fn accept(&self, s: &[u8]) -> bool {
        self.0.accept(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn regex_new() {
        assert!(Regex::new(r"\w?".as_bytes()).is_none());
        assert!(Regex::new(r"a(b(c(d)+)*)?(e(f)?(g)?)*".as_bytes()).is_some());
    }

    #[test]
    fn regex_email() {
        let x = Regex::new(r"(\w)+(.(\w)+)?@(\w)+.(\w)+".as_bytes()).unwrap();
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
            let x = Regex::new(r.as_bytes()).unwrap();
            assert_eq!(x.accept(s.as_bytes()), b, "{} {} {}", r, s, b);
        }
    }
}
