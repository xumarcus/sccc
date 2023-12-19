use super::parser::AST;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum IR {
    E,
    L(u8),
    U(Box<IR>, Box<IR>),
    C(Box<IR>, Box<IR>),
    K(Box<IR>),
}

impl IR {
    fn u(a: Self, b: Self) -> Self {
        Self::U(Box::new(a), Box::new(b))
    }

    fn c(a: Self, b: Self) -> Self {
        Self::C(Box::new(a), Box::new(b))
    }

    pub fn new(ast: AST) -> Self {
        use AST::*;
        use IR::*;

        match ast {
            Digit => (b'0'..=b'9').map(L).reduce(Self::u).unwrap(),
            Letter => (b'a'..=b'z')
                .chain(b'A'..=b'Z')
                .map(L)
                .reduce(Self::u)
                .unwrap(),
            Character(x) => L(x),
            CharacterClass(v) => v.into_iter().map(L).reduce(Self::u).unwrap(),
            String(s) => s.bytes().map(L).reduce(Self::c).unwrap(),
            Concat(v) => v.into_iter().map(Self::new).reduce(Self::c).unwrap(),
            Union(v) => v.into_iter().map(Self::new).reduce(Self::u).unwrap(),
            Star(a) => K(Box::new(IR::new(*a))),
            Plus(a) => {
                let ir = IR::new(*a);
                C(Box::new(ir.clone()), Box::new(K(Box::new(ir))))
            }
            Question(a) => U(Box::new(E), Box::new(IR::new(*a))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::AST::*;
    use crate::automata::*;
    use crate::ir::IR;

    #[test]
    fn ir_new() {
        let ast = Star(Box::new(Union(vec![
            Character(b'0'),
            Star(Box::new(Concat(vec![
                Character(b'1'),
                Star(Box::new(Concat(vec![
                    Character(b'0'),
                    Star(Box::new(Character(b'1'))),
                    Star(Box::new(Concat(vec![
                        Character(b'0'),
                        Character(b'0'),
                    ]))),
                    Character(b'0'),
                ]))),
                Character(b'1'),
            ]))),
        ])));
        let ir = IR::new(ast);
        let dfa = DFA::new(&ir);
        for x in 0..20 {
            let s = format!("{:b}", x);
            assert_eq!(dfa.accept(s.as_bytes()), x % 3 == 0);
        }
    }
}
