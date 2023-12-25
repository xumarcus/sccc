use std::{
    ops::RangeInclusive,
    rc::{Rc, Weak},
};

use crate::combinator::*;

use super::ast::{
    CharacterClass, CharacterClassItem, MetaCharacter,
    AST::{self, *},
};

const ESCAPED: [u8; 14] = [
    b'^', b'-', b'.', b'*', b'+', b'?', b'|', b'(', b')', b'[', b']', b'{',
    b'}', b'\\',
];

fn meta() -> impl Parser<Item = MetaCharacter> {
    use MetaCharacter::*;
    satisfy(b'\\')
        .then(ParserChar.filter_map(|x| match x {
            b'd' => Some(D),
            b'h' => Some(H),
            b'l' => Some(L),
            b's' => Some(S),
            b'w' => Some(W),
            _ => None,
        }))
        .or(satisfy(b'.').map(|_| Dot))
}

fn char() -> impl Parser<Item = u8> {
    satisfy(b'\\')
        .then(ParserChar.filter(|x| ESCAPED.contains(x)))
        .or(ParserChar.filter(|x| !ESCAPED.contains(x)))
}

fn atom() -> impl Parser<Item = CharacterClassItem> {
    char()
        .map(CharacterClassItem::Byte)
        .or(meta().map(CharacterClassItem::Meta))
}

fn ast_ccls() -> impl Parser<Item = AST> {
    between(
        optional(satisfy(b'^'))
            .zip_with(
                char()
                    .zip_with(satisfy(b'-').then(char()), RangeInclusive::new)
                    .map(CharacterClassItem::ByteRange)
                    .or(atom())
                    .collect(),
                |opt, v| {
                    if v.is_empty() {
                        None
                    } else {
                        Some(WithCharacterClass(CharacterClass::new(
                            opt.is_some(),
                            v,
                        )))
                    }
                },
            )
            .filter_map(|x| x),
        b'[',
        b']',
    )
}

pub(crate) fn ast_regex() -> Rc<Box<dyn Parser<Item = AST>>> {
    Rc::new_cyclic(|me: &Weak<Box<dyn Parser<Item = AST>>>| {
        Box::new(
            intersperse(
                atom()
                    .map(|x| WithCharacterClass(CharacterClass::from(x)))
                    .or(ast_ccls())
                    .or(between(me.clone(), b'(', b')')
                        .zip_with(ParserChar, |ast, x| {
                            let ast = Box::new(ast);
                            match x {
                                b'*' => Some(Star(ast)),
                                b'+' => Some(Plus(ast)),
                                b'?' => Some(QnMk(ast)),
                                _ => None,
                            }
                        })
                        .filter_map(|x| x))
                    .collect()
                    .filter_map(|mut v| match v.len() {
                        0 => None,
                        1 => v.pop(),
                        _ => Some(Concatenation(v)),
                    }),
                b'|',
            )
            .filter_map(|mut v| match v.len() {
                0 => None,
                1 => v.pop(),
                _ => Some(Alternation(v)),
            }),
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pat_meta() {
        let pattern = r".".as_bytes();
        assert!(meta().accept(pattern));
    }

    #[test]
    fn pat_atom() {
        let pattern = r"a".as_bytes();
        assert!(atom().accept(pattern));
    }

    #[test]
    fn pat_ccls() {
        let pattern = r"[a\[\d\]c]".as_bytes();
        assert!(ast_ccls().accept(pattern));
    }

    #[test]
    fn pat_ccls_range() {
        let pattern = r"[eb-da]".as_bytes();
        assert!(ast_ccls().accept(pattern));
    }

    #[test]
    fn pat_ccls_range_neg() {
        let pattern = r"[^b-d\d]".as_bytes();
        assert!(ast_ccls().accept(pattern));
    }

    #[test]
    fn pat_regex_single_char() {
        let pattern = r"a".as_bytes();
        assert!(ast_regex().accept(pattern));
    }

    #[test]
    fn pat_regex_qnmk() {
        let pattern = r"(a)?".as_bytes();
        assert!(ast_regex().accept(pattern));
    }

    #[test]
    fn pat_regex_no_bracket() {
        let pattern = r"\w?".as_bytes();
        assert!(!ast_regex().accept(pattern));
    }

    #[test]
    fn pat_regex_altr_conc() {
        let pattern = r"a|bc".as_bytes();
        assert!(ast_regex().accept(pattern));
    }

    #[test]
    fn pat_1() {
        let pattern = r"a|(b)?".as_bytes();
        assert!(ast_regex().accept(pattern));
    }

    #[test]
    fn pat_2() {
        let pattern = r"(b)?c".as_bytes();
        assert!(ast_regex().accept(pattern));
    }

    #[test]
    fn pat_3() {
        let pattern = r"a|bc".as_bytes();
        assert!(ast_regex().accept(pattern));
    }

    #[test]
    fn pat_4() {
        let pattern = r"a|(b(cd)*)?e".as_bytes();
        assert!(ast_regex().accept(pattern));
    }

    #[test]
    fn pat_5() {
        let pattern = r"(\-)?[1-9](\d)+".as_bytes();
        assert!(ast_regex().accept(pattern));
    }
}
