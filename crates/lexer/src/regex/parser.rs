#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MetaCharacter {
    D,
    H,
    L,
    S,
    W,
    Dot,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AST {
    Meta(MetaCharacter),
    Char(u8),
    CCls(Vec<u8>),
    Conc(Vec<AST>),
    Altr(Vec<AST>),
    Star(Box<AST>),
    Plus(Box<AST>),
    QnMk(Box<AST>),
}

use crate::combinator::*;
use MetaCharacter::*;
use AST::*;

fn ast_escaped() -> impl Parser<Item = AST> {
    satisfy(b'\\').then(
        PChar
            .filter_map(|x| match x {
                b'd' => Some(D),
                b'h' => Some(H),
                b'l' => Some(L),
                b's' => Some(S),
                b'w' => Some(W),
                _ => None,
            })
            .map(Meta),
    )
}

const ESCAPED: [u8; 12] = [
    b'.', b'*', b'+', b'?', b'|', b'(', b')', b'[', b']', b'{', b'}', b'\\',
];

fn ast_char() -> impl Parser<Item = u8> {
    satisfy(b'\\')
        .then(PChar.filter(|x| ESCAPED.contains(x)))
        .or(PChar.filter(|x| !ESCAPED.contains(x)))
}

fn ast_ccls() -> impl Parser<Item = AST> {
    between(
        ast_char().collect().filter(|v| !v.is_empty()).map(CCls),
        b'[',
        b']',
    )
}

fn cons(x: u8) -> Option<Box<dyn Fn(Box<AST>) -> AST>> {
    match x {
        b'*' => Some(Box::new(Star)),
        b'+' => Some(Box::new(Plus)),
        b'?' => Some(Box::new(QnMk)),
        _ => None,
    }
}

fn ast_atom(depth: usize) -> Box<dyn Parser<Item = AST>> {
    let p = ast_ccls()
        .or(ast_escaped())
        .or(ast_char().map(AST::Char))
        .or(satisfy(b'.').map(|_| Meta(Dot)));
    if depth == 0 {
        Box::new(p)
    } else {
        Box::new(
            between(ast_regex(depth - 1), b'(', b')')
                .and(PChar.filter_map(cons))
                .map(|(ast, constructor)| constructor(Box::new(ast)))
                .or(p),
        )
    }
}

pub(crate) fn ast_regex(depth: usize) -> impl Parser<Item = AST> {
    intersperse(
        ast_atom(depth).collect().filter_map(|mut v| match v.len() {
            0 => None,
            1 => v.pop(),
            _ => Some(Conc(v)),
        }),
        b'|',
    )
    .filter_map(|mut v| match v.len() {
        0 => None,
        1 => v.pop(),
        _ => Some(Altr(v)),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pat_escaped() {
        let pattern = r"\d".as_bytes();
        assert!(ast_escaped().accept(pattern));
    }

    #[test]
    fn pat_ccls() {
        let pattern = r"[a\[\]c]".as_bytes();
        assert!(ast_ccls().accept(pattern));
    }

    #[test]
    fn pat_atom_char() {
        let pattern = r"a".as_bytes();
        assert!(ast_atom(0).accept(pattern));
    }

    #[test]
    fn pat_atom_spq() {
        let pattern = r"(a)?".as_bytes();
        assert!(ast_atom(1).accept(pattern));
    }

    #[test]
    fn pat_atom_bad_char() {
        let pattern = r"*".as_bytes();
        assert!(!ast_atom(1).accept(pattern));
    }

    #[test]
    fn pat_char() {
        let pattern = r"a".as_bytes();
        assert!(ast_regex(0).accept(pattern));
    }

    #[test]
    fn pat_no_bracket() {
        let pattern = r"\w?".as_bytes();
        assert!(matches!(
            ast_regex(1).run(pattern),
            Some((Meta(MetaCharacter::W), [b'?']))
        ));
    }

    #[test]
    fn pat_altr() {
        let pattern = r"a|b".as_bytes();
        assert!(matches!(ast_regex(1).run(pattern), Some((Altr(_), []))));
    }

    #[test]
    fn pat_conc() {
        let pattern = r"ab".as_bytes();
        assert!(matches!(ast_regex(1).run(pattern), Some((Conc(_), []))));
    }

    #[test]
    fn pat_altr_conc() {
        let pattern = r"a|bc".as_bytes();
        assert!(ast_regex(1).accept(pattern));
    }

    #[test]
    fn pat_1() {
        let pattern = r"a|(b)?".as_bytes();
        assert!(ast_regex(3).accept(pattern));
    }

    #[test]
    fn pat_2() {
        let pattern = r"(b)?c".as_bytes();
        assert!(ast_regex(3).accept(pattern));
    }

    #[test]
    fn pat_3() {
        let pattern = r"a|bc".as_bytes();
        assert!(ast_regex(3).accept(pattern));
    }

    #[test]
    fn pat_4() {
        let pattern = r"a|(b(cd)*)?e".as_bytes();
        assert!(ast_regex(3).accept(pattern));
    }
}
