use crate::automata::IR;
use std::{iter::once, ops::RangeInclusive};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum MetaCharacter {
    D,
    H,
    L,
    S,
    W,
    Dot,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum CharacterClassItem {
    Byte(u8),
    Meta(MetaCharacter),
    ByteRange(RangeInclusive<u8>),
}

impl From<CharacterClassItem> for CharacterClass {
    fn from(value: CharacterClassItem) -> Self {
        Self {
            negated: false,
            items: vec![value],
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct CharacterClass {
    negated: bool,
    items: Vec<CharacterClassItem>,
}

impl CharacterClass {
    pub(crate) fn new(negated: bool, items: Vec<CharacterClassItem>) -> Self {
        Self { negated, items }
    }
}

use CharacterClassItem::*;
impl From<CharacterClass> for Vec<u8> {
    fn from(value: CharacterClass) -> Self {
        let v: Self = value
            .items
            .into_iter()
            .flat_map(|item| match item {
                Byte(x) => vec![x],
                Meta(m) => match m {
                    MetaCharacter::D => (b'0'..=b'9').collect(),
                    MetaCharacter::H => (b'a'..=b'f')
                        .chain(b'A'..=b'F')
                        .chain(b'0'..=b'9')
                        .collect(),
                    MetaCharacter::L => (b'a'..=b'z')
                        .chain(b'A'..=b'Z')
                        .chain(once(b'_'))
                        .collect(),
                    MetaCharacter::S => vec![b' ', b'\r', b'\n', b'\t'],
                    MetaCharacter::W => (b'a'..=b'z')
                        .chain(b'A'..=b'Z')
                        .chain(b'0'..=b'9')
                        .chain(once(b'_'))
                        .collect(),
                    MetaCharacter::Dot => {
                        (0..=255u8).filter(|&x| x != b'\n').collect()
                    }
                },
                ByteRange(r) => r.into_iter().collect(),
            })
            .collect();
        if value.negated {
            (0..=255u8).filter(|x| !v.contains(x)).collect()
        } else {
            v
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum AST {
    WithCharacterClass(CharacterClass),
    Concatenation(Vec<AST>),
    Alternation(Vec<AST>),
    Star(Box<AST>),
    Plus(Box<AST>),
    QnMk(Box<AST>),
}

impl From<AST> for IR {
    fn from(ast: AST) -> Self {
        use AST::*;
        use IR::*;

        match ast {
            WithCharacterClass(v) => L(v.into()),
            Concatenation(v) => C(v.into_iter().map(Self::from).collect()),
            Alternation(v) => U(v.into_iter().map(Self::from).collect()),
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
