use std::iter::once;

use super::parser::AST;
use crate::{automata::IR, regex::parser::MetaCharacter};

impl IR {
    pub fn new(ast: AST) -> Self {
        use AST::*;
        use IR::*;

        match ast {
            Meta(m) => match m {
                MetaCharacter::D => U((b'0'..=b'9').map(L).collect()),
                MetaCharacter::H => U((b'a'..=b'f')
                    .chain(b'A'..=b'F')
                    .chain(b'0'..=b'9')
                    .map(L)
                    .collect()),
                MetaCharacter::L => U((b'a'..=b'z')
                    .chain(b'A'..=b'Z')
                    .chain(once(b'_'))
                    .map(L)
                    .collect()),
                MetaCharacter::S => {
                    U(vec![L(b' '), L(b'\r'), L(b'\n'), L(b'\t')])
                }
                MetaCharacter::W => U((b'a'..=b'z')
                    .chain(b'A'..=b'Z')
                    .chain(b'0'..=b'9')
                    .chain(once(b'_'))
                    .map(L)
                    .collect()),
            },
            Char(x) => L(x),
            CCls(v) => U(v.into_iter().map(L).collect()),
            Conc(v) => C(v.into_iter().map(Self::new).collect()),
            Altr(v) => U(v.into_iter().map(Self::new).collect()),
            Star(a) => K(Box::new(IR::new(*a))),
            Plus(a) => {
                let ir = IR::new(*a);
                C(vec![ir.clone(), K(Box::new(ir))])
            }
            QnMk(a) => {
                let ir = IR::new(*a);
                U(vec![E, ir])
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::automata::dfa::DFA;
    use crate::automata::nfa::NFA;
    use crate::automata::IR;
    use crate::combinator::Parser;

    use super::AST::*;
    use super::IR::*;

    #[test]
    fn ir_direct_translate() {
        let ast = Star(Box::new(Altr(vec![
            Char(b'0'),
            Star(Box::new(Conc(vec![
                Char(b'1'),
                Star(Box::new(Conc(vec![
                    Char(b'0'),
                    Star(Box::new(Char(b'1'))),
                    Star(Box::new(Conc(vec![Char(b'0'), Char(b'0')]))),
                    Char(b'0'),
                ]))),
                Char(b'1'),
            ]))),
        ])));
        let ir = K(Box::new(U(vec![
            L(b'0'),
            K(Box::new(C(vec![
                L(b'1'),
                K(Box::new(C(vec![
                    L(b'0'),
                    K(Box::new(L(b'1'))),
                    K(Box::new(C(vec![L(b'0'), L(b'0')]))),
                    L(b'0'),
                ]))),
                L(b'1'),
            ]))),
        ])));
        assert_eq!(IR::new(ast), ir);
    }

    #[test]
    fn ir_indirect_translate() {
        use crate::regex::parser::MetaCharacter;
        let ast = Conc(vec![
            QnMk(Box::new(Char(b'-'))),
            Plus(Box::new(Meta(MetaCharacter::D))),
            QnMk(Box::new(Conc(vec![
                Char(b'.'),
                Plus(Box::new(Meta(MetaCharacter::D))),
            ]))),
        ]);
        let ir = IR::new(ast);
        let mut nfa = NFA::new();
        nfa.extend_with(&ir);
        let dfa = DFA::new(&nfa);
        assert!(dfa.accept("-1.234".as_bytes()));
        assert!(dfa.accept("1234".as_bytes()));
        assert!(!dfa.accept("".as_bytes()));
        assert!(!dfa.accept("12.ab".as_bytes()));
    }
}
