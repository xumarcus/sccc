use std::rc::{Rc, Weak};

use super::Parser;

impl<T> Parser for Box<dyn Parser<Item = T>> {
    type Item = T;
    fn run<'a>(&self, s: &'a [u8]) -> Option<(Self::Item, &'a [u8])> {
        self.as_ref().run(s)
    }
}

impl<T> Parser for Rc<dyn Parser<Item = T>> {
    type Item = T;
    fn run<'a>(&self, s: &'a [u8]) -> Option<(Self::Item, &'a [u8])> {
        self.as_ref().run(s)
    }
}

impl<T> Parser for Weak<dyn Parser<Item = T>> {
    type Item = T;
    fn run<'a>(&self, s: &'a [u8]) -> Option<(Self::Item, &'a [u8])> {
        self.upgrade().expect("Cyclic referenced").run(s)
    }
}

impl<P: Parser> Parser for Box<P> {
    type Item = P::Item;
    fn run<'a>(&self, s: &'a [u8]) -> Option<(Self::Item, &'a [u8])> {
        self.as_ref().run(s)
    }
}

impl<P: Parser> Parser for Rc<P> {
    type Item = P::Item;
    fn run<'a>(&self, s: &'a [u8]) -> Option<(Self::Item, &'a [u8])> {
        self.as_ref().run(s)
    }
}

impl<P: Parser> Parser for Weak<P> {
    type Item = P::Item;
    fn run<'a>(&self, s: &'a [u8]) -> Option<(Self::Item, &'a [u8])> {
        self.upgrade().expect("Cyclic referenced").run(s)
    }
}
