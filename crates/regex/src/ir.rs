use super::parser::AST;

// TODO: refactor to Vec<IR> instead of Box
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum IR {
    E,
    L(u8),
    U(Vec<IR>),
    C(Vec<IR>),
    K(Box<IR>),
}

impl IR {
    pub fn new(ast: AST) -> Self {
        use AST::*;
        use IR::*;

        match ast {
            DChr => U((b'0'..=b'9').map(L).collect()),
            WChr => U((b'a'..=b'z').chain(b'A'..=b'Z').map(L).collect()),
            SChr => U(vec![L(b' '), L(b'\r'), L(b'\n'), L(b'\t')]),
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
    use crate::automata::Automaton;
    use crate::automata::DFA;
    use crate::ir::IR;

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
        let ast = Conc(vec![
            QnMk(Box::new(Char(b'-'))),
            Plus(Box::new(DChr)),
            QnMk(Box::new(Conc(vec![Char(b'.'), Plus(Box::new(DChr))]))),
        ]);
        let ir = IR::new(ast);
        let dfa = DFA::new(&ir);
        assert!(dfa.accept("-1.234".as_bytes()));
        assert!(dfa.accept("1234".as_bytes()));
        assert!(!dfa.accept("".as_bytes()));
        assert!(!dfa.accept("12.ab".as_bytes()));
    }
}
