// https://www.lysator.liu.se/c/ANSI-C-grammar-l.html

use lexer::Lexer;
use std::str::from_utf8;

mod token;
use token::{
    KeywordToken::*,
    LiteralToken::*,
    OperatorToken::{self, *},
    Token::{self, *},
};

use crate::clex::token::IntegerToken;

macro_rules! constant {
    ($x: expr) => {
        Box::new(move |_| $x)
    };
}

macro_rules! keyword {
    ($x:ident) => {
        (casey::lower!(stringify!($x)), constant!(Some(Keyword($x))))
    };
}

fn op(x: OperatorToken) -> lexer::Action<Option<Token>> {
    constant!(Some(Operator(x)))
}

pub fn clex() -> lexer::Result<Lexer<Option<Token>>> {
    let v: Vec<(&str, lexer::Action<Option<Token>>)> = vec![
        ("//(.)*\n|/\\*(.|\n)*\\*/|(\\s)+", constant!(None)),
        keyword!(Auto),
        keyword!(Break),
        keyword!(Case),
        keyword!(Char),
        keyword!(Const),
        keyword!(Continue),
        keyword!(Default),
        keyword!(Do),
        keyword!(Double),
        keyword!(Else),
        keyword!(Enum),
        keyword!(Extern),
        keyword!(Float),
        keyword!(For),
        keyword!(Goto),
        keyword!(If),
        keyword!(Int),
        keyword!(Long),
        keyword!(Register),
        keyword!(Return),
        keyword!(Short),
        keyword!(Signed),
        keyword!(Sizeof),
        keyword!(Static),
        keyword!(Struct),
        keyword!(Switch),
        keyword!(Typedef),
        keyword!(Union),
        keyword!(Unsigned),
        keyword!(Void),
        keyword!(Volatile),
        keyword!(While),
        (r"\.\.\.", op(Ellipsis)),
        (r">>=", op(ShrAsn)),
        (r"<<=", op(ShlAsn)),
        (r"\+=", op(AddAsn)),
        (r"-=", op(SubAsn)),
        (r"\*=", op(MulAsn)),
        (r"/=", op(DivAsn)),
        (r"%=", op(ModAsn)),
        (r"&=", op(AndAsn)),
        (r"^=", op(XorAsn)),
        (r"\|=", op(OrAsn)),
        (r">>", op(Shr)),
        (r"<<", op(Shl)),
        (r"\+\+", op(Inc)),
        (r"--", op(Dec)),
        (r"->", op(Ptr)),
        (r"&&", op(And)),
        (r"\|\|", op(Or)),
        (r"<=", op(Le)),
        (r">=", op(Ge)),
        (r"==", op(Eq)),
        (r"!=", op(Ne)),
        (r";", op(Semicolon)),
        (r"\{|<%", op(LBrace)),
        (r"\}|>%", op(RBrace)),
        (r",", op(Comma)),
        (r":", op(Colon)),
        (r"=", op(Assign)),
        (r"\(", op(LParen)),
        (r"\)", op(RParen)),
        (r"\[|<:", op(LSqBr)),
        (r"\]|>:", op(RSqBr)),
        (r"\.", op(Dot)),
        (r"&", op(BitAnd)),
        (r"!", op(Not)),
        (r"~", op(Tilde)),
        (r"-", op(Minus)),
        (r"\+", op(Plus)),
        (r"\*", op(Ast)),
        (r"/", op(Div)),
        (r"%", op(Mod)),
        (r"<", op(Lt)),
        (r">", op(Gt)),
        (r"^", op(Caret)),
        (r"\|", op(BitOr)),
        (r"\?", op(QnMk)),
        (
            r"\l(\l|\d)*",
            Box::new(|s| {
                Some(Identifier(from_utf8(s).expect("identifier").to_owned()))
            }),
        ),
        (
            r"(-)?(\d)+([uUlL])*",
            Box::new(|s| {
                use IntegerToken::*;
                let i = s.partition_point(|x| (b'0'..=b'9').contains(x));
                let (l, r) = s.split_at(i);
                let t = from_utf8(l).expect("integer");
                let k: i64 = t.parse().expect("in range");
                Some(Literal(LInt(match r {
                    b"" | b"l" => L(k as i32),
                    b"ll" => LL(k as i64),
                    b"u" | b"ul" | b"lu" => UL(k as u32),
                    b"ull" | b"llu" => ULL(k as u64),
                    _ => todo!(), // wrap in err instead
                })))
            }),
        ),
    ];
    Lexer::new(v.into_iter())
}

#[cfg(test)]
mod tests {
    use crate::clex::*;
    use lexer::combinator::Parser;

    #[test]
    fn clex_test() {
        let lexer = clex().unwrap();
        let code = r#"
            int main() {
                return 0;
            }
        "#;
        let tokens: Vec<Token> =
            lexer.items(code.as_bytes()).filter_map(|x| x).collect();
        assert_eq!(
            tokens,
            vec![
                Keyword(Int),
                Identifier("main".to_string()),
                Operator(LParen),
                Operator(RParen),
                Operator(LBrace),
                Keyword(Return),
                Literal(LInt(IntegerToken::L(0))),
                Operator(Semicolon),
                Operator(RBrace)
            ]
        );
    }
}
