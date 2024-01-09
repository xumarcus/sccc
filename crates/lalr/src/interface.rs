#![feature(generic_const_exprs)]

pub trait Finite: Clone + Default {
    const COUNT: usize;
    fn similar(&self, other: &Self) -> bool;
    fn id(&self) -> usize;
}

pub trait Integral: Finite + Copy + From<usize> {}

enum Symbol<N: Integral, T: Finite> {
    N(N),
    T(T),
}

enum SemanticAction<T: Finite, U> {
    FromToken(fn(&T) -> U),
    Op0(fn() -> U),
    Op1(fn(U) -> U),
    Op2(fn(U, U) -> U),
    Op3(fn(U, U, U) -> U),
}

struct Production<N: Integral, T: Finite, U> {
    derivation: Vec<Symbol<N, T>>,
    semantic_action: SemanticAction<T, U>,
}

struct Grammar<N: Integral, T: Finite, U>([Vec<Production<N, T, U>>; N::COUNT])
where
    [(); N::COUNT + T::COUNT]: Sized;

/*
struct LALR<N: Integral, T: Finite, U>();
impl<N: Integral, T: Finite, U> LALR<N, T, U> where [(); N::COUNT + T::COUNT]: Sized {
    fn new(grammar: &Grammar<N, T, U>) -> Self {

    }

    fn run<'a>(&'a self, s: &'a [T]) -> Option<(U, &'a [T])> {

    }
}
 */

/*
macro_rules! symbol {
    ($a:ident $_:ident % $x:ident) => { $a($x) };
    ($_:ident $b:ident ^ $x:ident) => { $b($x) };
}

macro_rules! grammar {
    (	%: $a:ident
        ^: $b:ident
        p: $($_:ident -> $( $( $t:tt $x:ident ),* )&+);+
    ) => {
        $(
            $(
                println!("{:?}", vec![$( symbol!($a $b $t $x) ),*]);
            )&+
        );+
    }
}
*/
