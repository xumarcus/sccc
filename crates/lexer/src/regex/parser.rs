#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum AST {
    DChr,
    WChr,
    SChr,
    Char(u8),
    CCls(Vec<u8>),
    Conc(Vec<AST>),
    Altr(Vec<AST>),
    Star(Box<AST>),
    Plus(Box<AST>),
    QnMk(Box<AST>),
}
use AST::*;
use crate::combinator::*;

fn ast_dws() -> impl Parser<Item = AST> {
    satisfy(b'\\').then(PChar.filter_map(|x| match x {
        b'd' => Some(DChr),
        b'w' => Some(WChr),
        b's' => Some(SChr),
        _ => None,
    }))
}

const ESCAPED: [u8; 9] =
    [b'*', b'+', b'?', b'|', b'(', b')', b'[', b']', b'\\'];

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
    let p = ast_ccls().or(ast_dws()).or(ast_char().map(AST::Char));
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
    fn pat_dws() {
        let pattern = r"\d".as_bytes();
        assert_eq!(ast_dws().run(pattern), Some((DChr, &[] as &[u8])))
    }

    #[test]
    fn pat_ccls() {
        let pattern = r"[a\[\]c]".as_bytes();
        assert_eq!(
            ast_ccls().run(pattern),
            Some((CCls(vec![b'a', b'[', b']', b'c']), &[] as &[u8]))
        )
    }

    #[test]
    fn pat_atom_char() {
        let pattern = r"a".as_bytes();
        assert!(matches!(ast_atom(0).run(pattern), Some((Char(_), _))))
    }

    #[test]
    fn pat_atom_spq() {
        let pattern = r"(a)?".as_bytes();
        assert!(matches!(ast_atom(1).run(pattern), Some((QnMk(_), _))))
    }

    #[test]
    fn pat_atom_bad_char() {
        let pattern = r"*".as_bytes();
        assert_eq!(ast_atom(1).run(pattern), None);
    }

    #[test]
    fn pat_char() {
        let pattern = r"a".as_bytes();
        assert!(matches!(ast_regex(0).run(pattern), Some((Char(_), _))))
    }

    #[test]
    fn pat_no_bracket() {
        let pattern = r"\w?".as_bytes();
        assert!(matches!(ast_regex(1).run(pattern), Some((WChr, [b'?']))));
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
        assert!(matches!(ast_regex(1).run(pattern), Some((_, []))));
    }

    #[test]
    fn pat_1() {
        let pattern = r"a|(b)?".as_bytes();
        assert!(matches!(ast_regex(5).run(pattern), Some((_, []))));
    }

    #[test]
    fn pat_2() {
        let pattern = r"(b)?c".as_bytes();
        assert!(matches!(ast_regex(5).run(pattern), Some((_, []))));
    }

    #[test]
    fn pat_3() {
        let pattern = r"a|bc".as_bytes();
        assert!(matches!(ast_regex(5).run(pattern), Some((_, []))));
    }

    #[test]
    fn pat_4() {
        let pattern = r"a|(b(cd)*)?e".as_bytes();
        assert!(matches!(ast_regex(5).run(pattern), Some((_, []))));
    }
}
