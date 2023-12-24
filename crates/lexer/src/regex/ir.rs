use std::iter::once;

use super::parser::AST;
use crate::{automata::IR, regex::parser::MetaCharacter};

impl IR {
    pub fn new(ast: AST) -> Self {
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
