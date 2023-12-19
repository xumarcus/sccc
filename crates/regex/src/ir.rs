use super::parser::AST;

#[derive(Clone)]
pub enum IR {
    E,
    L(u8),
    U(Box<IR>, Box<IR>),
    C(Box<IR>, Box<IR>),
    K(Box<IR>),
}

impl IR {
    pub fn new(ast: AST) -> Self {
        use AST::*;
        use IR::*;

        match ast {
            Digit => {
                (b'0'..=b'9').fold(E, |acc, x| U(Box::new(acc), Box::new(L(x))))
            }
            Letter => {
                (b'a'..=b'Z').fold(E, |acc, x| U(Box::new(acc), Box::new(L(x))))
            }
            CharacterClass(v) => v
                .into_iter()
                .fold(E, |acc, x| U(Box::new(acc), Box::new(L(x)))),
            String(s) => {
                s.bytes().fold(E, |acc, x| C(Box::new(acc), Box::new(L(x))))
            }
            Concat(v) => v
                .into_iter()
                .fold(E, |acc, x| C(Box::new(acc), Box::new(IR::new(x)))),
            Union(v) => v
                .into_iter()
                .fold(E, |acc, x| U(Box::new(acc), Box::new(IR::new(x)))),
            Star(a) => K(Box::new(IR::new(*a))),
            Plus(a) => {
                let ir = IR::new(*a);
                C(Box::new(ir.clone()), Box::new(K(Box::new(ir))))
            }
            Question(a) => U(Box::new(E), Box::new(IR::new(*a))),
        }
    }
}
