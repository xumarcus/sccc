use std::iter::once;

use crate::automata::IR;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum MetaCharacter {
    D,
    H,
    L,
    S,
    W,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum AST {
    Dot,
    Meta(MetaCharacter),
    Char(u8),
    CCls(Vec<u8>),
    Conc(Vec<AST>),
    Altr(Vec<AST>),
    Star(Box<AST>),
    Plus(Box<AST>),
    QnMk(Box<AST>),
}

impl From<AST> for IR {
    fn from(ast: AST) -> Self {
        use AST::*;
        use IR::*;

        match ast {
            Meta(m) => match m {
                MetaCharacter::D => L((b'0'..=b'9').collect()),
                MetaCharacter::H => L((b'a'..=b'f')
                    .chain(b'A'..=b'F')
                    .chain(b'0'..=b'9')
                    .collect()),
                MetaCharacter::L => L((b'a'..=b'z')
                    .chain(b'A'..=b'Z')
                    .chain(once(b'_'))
                    .collect()),
                MetaCharacter::S => L(vec![b' ', b'\r', b'\n', b'\t']),
                MetaCharacter::W => L((b'a'..=b'z')
                    .chain(b'A'..=b'Z')
                    .chain(b'0'..=b'9')
                    .chain(once(b'_'))
                    .collect()),
            },
            Dot => L((0..=255u8).filter(|&x| x != b'\n').collect()),
            Char(x) => L(vec![x]),
            CCls(v) => L(v),
            Conc(v) => C(v.into_iter().map(Self::from).collect()),
            Altr(v) => U(v.into_iter().map(Self::from).collect()),
            Star(a) => K(Box::new(Self::from(*a))),
            Plus(a) => {
                let ir = Self::from(*a);
                C(vec![ir.clone(), K(Box::new(ir))])
            }
            QnMk(a) => {
                let ir = Self::from(*a);
                U(vec![E, ir])
            }
        }
    }
}
