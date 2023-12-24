use std::{iter::successors, rc::Rc};

mod ptrs;

pub trait Parser {
    type Item;
    fn run<'a>(&self, s: &'a [u8]) -> Option<(Self::Item, &'a [u8])>;

    fn accept(&self, s: &[u8]) -> bool {
        matches!(self.run(s), Some((_, [])))
    }
    fn items<'a>(&'a self, s: &'a [u8]) -> Items<'a, Self>
    where
        Self: Sized,
    {
        Items(self, s)
    }
    fn collect(self) -> Collect<Self>
    where
        Self: Sized,
    {
        Collect(self)
    }
    fn and<Q: Parser>(self, other: Q) -> And<Self, Q>
    where
        Self: Sized,
    {
        And(self, other)
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
    fn then<Q: Parser>(self, other: Q) -> Then<Self, Q>
    where
        Self: Sized,
    {
        Then(self, other)
    }
    fn skip<Q: Parser>(self, other: Q) -> Skip<Self, Q>
    where
        Self: Sized,
    {
        Skip(self, other)
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
}

pub struct Items<'a, P: Parser>(&'a P, &'a [u8]);
impl<'a, P: Parser> Iterator for Items<'a, P> {
    type Item = P::Item;
    fn next(&mut self) -> Option<Self::Item> {
        let Items(p, s) = self;
        let (x, t) = p.run(s)?;
        *s = t;
        Some(x)
    }
}
pub struct Collect<P: Parser>(P);
impl<P: Parser> Parser for Collect<P> {
    type Item = Vec<P::Item>;
    fn run<'a>(&self, s: &'a [u8]) -> Option<(Self::Item, &'a [u8])> {
        let (v, mut t): (Self::Item, Vec<&'a [u8]>) =
            successors(self.0.run(s), |(_, t)| self.0.run(t)).unzip();
        Some((v, t.pop().unwrap_or(s)))
    }
}

pub struct And<P: Parser, Q: Parser>(P, Q);
impl<P: Parser, Q: Parser> Parser for And<P, Q> {
    type Item = (P::Item, Q::Item);
    fn run<'a>(&self, s: &'a [u8]) -> Option<(Self::Item, &'a [u8])> {
        let (x, t) = self.0.run(s)?;
        let (y, u) = self.1.run(t)?;
        Some(((x, y), u))
    }
}

pub struct Or<P: Parser, Q: Parser<Item = P::Item>>(P, Q);
impl<P: Parser, Q: Parser<Item = P::Item>> Parser for Or<P, Q> {
    type Item = P::Item;
    fn run<'a>(&self, s: &'a [u8]) -> Option<(Self::Item, &'a [u8])> {
        self.0.run(s).or_else(|| self.1.run(s))
    }
}

pub struct Bind<P: Parser, Q: Parser, F: Fn(P::Item) -> Q>(P, F);
impl<P: Parser, Q: Parser, F: Fn(P::Item) -> Q> Parser for Bind<P, Q, F> {
    type Item = Q::Item;
    fn run<'a>(&self, s: &'a [u8]) -> Option<(Self::Item, &'a [u8])> {
        let (x, t) = self.0.run(s)?;
        self.1(x).run(t)
    }
}

pub struct Then<P: Parser, Q: Parser>(P, Q);
impl<P: Parser, Q: Parser> Parser for Then<P, Q> {
    type Item = Q::Item;
    fn run<'a>(&self, s: &'a [u8]) -> Option<(Self::Item, &'a [u8])> {
        let (_, t) = self.0.run(s)?;
        self.1.run(t)
    }
}

pub struct Skip<P: Parser, Q: Parser>(P, Q);
impl<P: Parser, Q: Parser> Parser for Skip<P, Q> {
    type Item = P::Item;
    fn run<'a>(&self, s: &'a [u8]) -> Option<(Self::Item, &'a [u8])> {
        let (x, t) = self.0.run(s)?;
        let (_, s) = self.1.run(t)?;
        Some((x, s))
    }
}

pub struct Map<P: Parser, T, F: Fn(P::Item) -> T>(P, F);
impl<P: Parser, T, F: Fn(P::Item) -> T> Parser for Map<P, T, F> {
    type Item = T;
    fn run<'a>(&self, s: &'a [u8]) -> Option<(Self::Item, &'a [u8])> {
        let (x, t) = self.0.run(s)?;
        Some((self.1(x), t))
    }
}

pub struct Filter<P: Parser, F: Fn(&P::Item) -> bool>(P, F);
impl<P: Parser, F: Fn(&P::Item) -> bool> Parser for Filter<P, F> {
    type Item = P::Item;
    fn run<'a>(&self, s: &'a [u8]) -> Option<(Self::Item, &'a [u8])> {
        self.0.run(s).filter(|(x, _)| self.1(x))
    }
}

pub struct FilterMap<P: Parser, T, F: Fn(P::Item) -> Option<T>>(P, F);
impl<P: Parser, T, F: Fn(P::Item) -> Option<T>> Parser for FilterMap<P, T, F> {
    type Item = T;
    fn run<'a>(&self, s: &'a [u8]) -> Option<(Self::Item, &'a [u8])> {
        let (x, t) = self.0.run(s)?;
        Some((self.1(x)?, t))
    }
}

pub struct ParserChar;
impl Parser for ParserChar {
    type Item = u8;
    fn run<'a>(&self, s: &'a [u8]) -> Option<(Self::Item, &'a [u8])> {
        let (h, t) = s.split_first()?;
        Some((*h, t))
    }
}

pub fn satisfy(a: u8) -> impl Parser<Item = ()> {
    ParserChar.filter_map(move |x| if x == a { Some(()) } else { None })
}

pub fn between<P: Parser>(p: P, op: u8, cl: u8) -> impl Parser<Item = P::Item> {
    satisfy(op).then(p).skip(satisfy(cl))
}

pub fn intersperse<P: Parser>(
    p: P,
    sep: u8,
) -> impl Parser<Item = Vec<P::Item>> {
    let p1 = Rc::new(p);
    let p2 = Rc::clone(&p1);
    p1.and(satisfy(sep).then(p2).collect()).map(|(x, mut v)| {
        v.insert(0, x);
        v
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn combinator_between_1() {
        let s = r"[a]".as_bytes();
        let p = between(ParserChar, b'[', b']');
        assert_eq!(p.run(s), Some((b'a', &[] as &[u8])));
    }

    #[test]
    fn combinator_between_2() {
        let s = r"[ab]".as_bytes();
        let p = between(ParserChar, b'[', b']');
        assert_eq!(p.run(s), None);
    }

    #[test]
    fn combinator_intersperse_1() {
        let s = r"a".as_bytes();
        let p = intersperse(ParserChar, b',');
        assert_eq!(p.run(s), Some((vec![b'a'], &[] as &[u8])));
    }

    #[test]
    fn combinator_intersperse_2() {
        let s = r"a,b,c".as_bytes();
        let p = intersperse(ParserChar, b',');
        assert_eq!(p.run(s), Some((vec![b'a', b'b', b'c'], &[] as &[u8])));
    }
}
