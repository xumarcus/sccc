#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum AST {
    DChr,
    WChr,
    SChr,
    Char(u8),
    CCls(Vec<u8>),
    Conc(Vec<AST>),
    Altr(Vec<AST>),
    // Require brackets
    Star(Box<AST>),
    Plus(Box<AST>),
    QnMk(Box<AST>),
}
use AST::*;

pub(crate) trait Parser {
    type Item;

    fn run<'a, 'b>(&'b self, s: &'a [u8]) -> Option<(Self::Item, &'a [u8])>;
    fn collect(self) -> Collect<Self>
    where
        Self: Sized,
    {
        Collect(self)
    }
    fn or<Q: Parser<Item = Self::Item>>(self, other: Q) -> Or<Self, Q>
    where
        Self: Sized,
    {
        Or(self, other)
    }
    fn bind<Q: Parser, F: Fn(Self::Item) -> Q>(self, f: F) -> Bind<Self, Q, F>
    where
        Self: Sized,
    {
        Bind(self, f)
    }
    fn zip<Q: Parser>(self, other: Q) -> Zip<Self, Q>
    where
        Self: Sized,
    {
        Zip(self, other)
    }
    fn zip_ref<Q: Parser>(self, other: &Q) -> ZipRef<Self, Q>
    where
        Self: Sized,
    {
        ZipRef(self, other)
    }
    fn then<Q: Parser>(self, other: Q) -> Then<Self, Q>
    where
        Self: Sized,
    {
        Then(self, other)
    }
    fn then_ref<Q: Parser>(self, other: &Q) -> ThenRef<Self, Q>
    where
        Self: Sized,
    {
        ThenRef(self, other)
    }
    fn skip<Q: Parser>(self, other: Q) -> Skip<Self, Q>
    where
        Self: Sized,
    {
        Skip(self, other)
    }
    fn skip_ref<Q: Parser>(self, other: &Q) -> SkipRef<Self, Q>
    where
        Self: Sized,
    {
        SkipRef(self, other)
    }
    fn map<T, F: Fn(Self::Item) -> T>(self, f: F) -> Map<Self, T, F>
    where
        Self: Sized,
    {
        Map(self, f)
    }
    fn filter<F: Fn(&Self::Item) -> bool>(self, f: F) -> Filter<Self, F>
    where
        Self: Sized,
    {
        Filter(self, f)
    }
    fn filter_map<T, F: Fn(Self::Item) -> Option<T>>(
        self,
        f: F,
    ) -> FilterMap<Self, T, F>
    where
        Self: Sized,
    {
        FilterMap(self, f)
    }
    fn intersperse(self, sep: u8) -> Intersperse<Self>
    where
        Self: Sized,
    {
        Intersperse(self, sep)
    }
    fn between(self, op: u8, cl: u8) -> Between<Self>
    where
        Self: Sized,
    {
        Between(self, op, cl)
    }
}

struct Collect<P: Parser>(P);
impl<P: Parser> Parser for Collect<P> {
    type Item = Vec<P::Item>;
    fn run<'a, 'b>(
        &'b self,
        mut s: &'a [u8],
    ) -> Option<(Self::Item, &'a [u8])> {
        let mut v: Self::Item = Vec::new();
        while let Some((x, t)) = self.0.run(s) {
            v.push(x);
            s = t;
        }
        Some((v, s))
    }
}

struct Or<P: Parser, Q: Parser<Item = P::Item>>(P, Q);
impl<P: Parser, Q: Parser<Item = P::Item>> Parser for Or<P, Q> {
    type Item = P::Item;
    fn run<'a, 'b>(&'b self, s: &'a [u8]) -> Option<(Self::Item, &'a [u8])> {
        self.0.run(s).or_else(|| self.1.run(s))
    }
}

struct Bind<P: Parser, Q: Parser, F: Fn(P::Item) -> Q>(P, F);
impl<P: Parser, Q: Parser, F: Fn(P::Item) -> Q> Parser for Bind<P, Q, F> {
    type Item = Q::Item;
    fn run<'a, 'b>(&'b self, s: &'a [u8]) -> Option<(Self::Item, &'a [u8])> {
        let (x, t) = self.0.run(s)?;
        self.1(x).run(t)
    }
}

struct Zip<P: Parser, Q: Parser>(P, Q);
impl<P: Parser, Q: Parser> Parser for Zip<P, Q> {
    type Item = (P::Item, Q::Item);
    fn run<'a, 'b>(&'b self, s: &'a [u8]) -> Option<(Self::Item, &'a [u8])> {
        let (x, t) = self.0.run(s)?;
        let (y, u) = self.1.run(t)?;
        Some(((x, y), u))
    }
}

struct ZipRef<'c, P: Parser, Q: Parser>(P, &'c Q);
impl<'c, P: Parser, Q: Parser> Parser for ZipRef<'c, P, Q> {
    type Item = (P::Item, Q::Item);
    fn run<'a, 'b>(&'b self, s: &'a [u8]) -> Option<(Self::Item, &'a [u8])> {
        let (x, t) = self.0.run(s)?;
        let (y, u) = self.1.run(t)?;
        Some(((x, y), u))
    }
}

struct Then<P: Parser, Q: Parser>(P, Q);
impl<P: Parser, Q: Parser> Parser for Then<P, Q> {
    type Item = Q::Item;
    fn run<'a, 'b>(&'b self, s: &'a [u8]) -> Option<(Self::Item, &'a [u8])> {
        let (_, t) = self.0.run(s)?;
        self.1.run(t)
    }
}

struct ThenRef<'c, P: Parser, Q: Parser>(P, &'c Q);
impl<'c, P: Parser, Q: Parser> Parser for ThenRef<'c, P, Q> {
    type Item = Q::Item;
    fn run<'a, 'b>(&'b self, s: &'a [u8]) -> Option<(Self::Item, &'a [u8])> {
        let (_, t) = self.0.run(s)?;
        self.1.run(t)
    }
}

struct Skip<P: Parser, Q: Parser>(P, Q);
impl<P: Parser, Q: Parser> Parser for Skip<P, Q> {
    type Item = P::Item;
    fn run<'a, 'b>(&'b self, s: &'a [u8]) -> Option<(Self::Item, &'a [u8])> {
        let (x, t) = self.0.run(s)?;
        let (_, u) = self.1.run(t)?;
        Some((x, u))
    }
}

struct SkipRef<'c, P: Parser, Q: Parser>(P, &'c Q);
impl<'c, P: Parser, Q: Parser> Parser for SkipRef<'c, P, Q> {
    type Item = P::Item;
    fn run<'a, 'b>(&'b self, s: &'a [u8]) -> Option<(Self::Item, &'a [u8])> {
        let (x, t) = self.0.run(s)?;
        let (_, u) = self.1.run(t)?;
        Some((x, u))
    }
}

struct Map<P: Parser, T, F: Fn(P::Item) -> T>(P, F);
impl<P: Parser, T, F: Fn(P::Item) -> T> Parser for Map<P, T, F> {
    type Item = T;
    fn run<'a, 'b>(&'b self, s: &'a [u8]) -> Option<(Self::Item, &'a [u8])> {
        let (x, t) = self.0.run(s)?;
        Some((self.1(x), t))
    }
}

struct Filter<P: Parser, F: Fn(&P::Item) -> bool>(P, F);
impl<P: Parser, F: Fn(&P::Item) -> bool> Parser for Filter<P, F> {
    type Item = P::Item;
    fn run<'a, 'b>(&'b self, s: &'a [u8]) -> Option<(Self::Item, &'a [u8])> {
        self.0.run(s).filter(|(x, _)| self.1(x))
    }
}

struct FilterMap<P: Parser, T, F: Fn(P::Item) -> Option<T>>(P, F);
impl<P: Parser, T, F: Fn(P::Item) -> Option<T>> Parser for FilterMap<P, T, F> {
    type Item = T;
    fn run<'a, 'b>(&'b self, s: &'a [u8]) -> Option<(Self::Item, &'a [u8])> {
        let (x, t) = self.0.run(s)?;
        Some((self.1(x)?, t))
    }
}

struct Intersperse<P: Parser>(P, u8);
impl<P: Parser> Parser for Intersperse<P> {
    type Item = Vec<P::Item>;
    fn run<'a, 'b>(&'b self, s: &'a [u8]) -> Option<(Self::Item, &'a [u8])> {
        let (x, t) = self.0.run(s)?;
        let p = satisfy(self.1).then_ref(&self.0);
        let (mut v, u) = p.collect().run(t)?;
        v.insert(0, x);
        Some((v, u))
    }
}

struct Between<P: Parser>(P, u8, u8);
impl<P: Parser> Parser for Between<P> {
    type Item = P::Item;
    fn run<'a, 'b>(&'b self, s: &'a [u8]) -> Option<(Self::Item, &'a [u8])> {
        let (_, t) = satisfy(self.1).run(s)?;
        let (x, u) = self.0.run(t)?;
        let (_, v) = satisfy(self.2).run(u)?;
        Some((x, v))
    }
}

struct PChar;
impl Parser for PChar {
    type Item = u8;
    fn run<'a, 'b>(&'b self, s: &'a [u8]) -> Option<(Self::Item, &'a [u8])> {
        let (h, t) = s.split_first()?;
        Some((*h, t))
    }
}

fn satisfy(a: u8) -> impl Parser<Item = ()> {
    PChar.filter_map(move |x| if x == a { Some(()) } else { None })
}

fn between<P: Parser>(p: P, op: u8, cl: u8) -> impl Parser<Item = P::Item> {
    satisfy(op).then(p).skip(satisfy(cl))
}

struct RecursiveDescentParser<T>(Box<dyn Parser<Item = T>>);
impl<T> Parser for RecursiveDescentParser<T> {
    type Item = T;

    fn run<'a, 'b>(&'b self, s: &'a [u8]) -> Option<(Self::Item, &'a [u8])> {
        self.0.run(s)
    }
}

fn ast_dws() -> impl Parser<Item = AST> {
    satisfy(b'\\').then(PChar.filter_map(|x| match x {
        b'd' => Some(DChr),
        b'w' => Some(WChr),
        b's' => Some(SChr),
        _ => None,
    }))
}

const ESCAPED: [u8; 9] = [b'*', b'+', b'?', b'|', b'(', b')', b'[', b']', b'\\'];

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

fn ast_atom(depth: usize) -> RecursiveDescentParser<AST> {
    let p = ast_ccls().or(ast_dws()).or(ast_char().map(AST::Char));
    if depth == 0 {
        RecursiveDescentParser(Box::new(p))
    } else {
        RecursiveDescentParser(Box::new(
            between(ast_regex(depth - 1), b'(', b')')
                .zip(PChar.filter_map(cons))
                .map(|(ast, constructor)| constructor(Box::new(ast)))
                .or(p),
        ))
    }
}

pub(crate) fn ast_regex(depth: usize) -> impl Parser<Item = AST> {
    ast_atom(depth)
        .collect()
        .filter_map(|mut v| match v.len() {
            0 => None,
            1 => v.pop(),
            _ => Some(Conc(v)),
        })
        .intersperse(b'|')
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
    fn pat_atom() {
        let pattern = r"(a)?".as_bytes();
        assert!(matches!(ast_atom(1).run(pattern), Some((QnMk(_), _))))
    }

    #[test]
    fn pat_atom_bad_char() {
        let pattern = r"*".as_bytes();
        assert_eq!(ast_atom(1).run(pattern), None);
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
