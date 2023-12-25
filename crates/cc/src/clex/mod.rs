// https://www.lysator.liu.se/c/ANSI-C-grammar-l.html

use anyhow::anyhow;
use lexer::{combinator::Parser, Action, Lexer};

mod token;
use token::{
    IntegerToken,
    KeywordToken::*,
    LiteralToken::*,
    OperatorToken::{self, *},
    Token::{self, *},
};

pub type ActionT = Action<Option<Token>, anyhow::Error>;
pub type LexerT = Lexer<Option<Token>, anyhow::Error>;

fn constant(x: Token) -> ActionT {
    Action::C(Some(x))
}

macro_rules! keyword {
    ($x:ident) => {
        (casey::lower!(stringify!($x)), constant(Keyword($x)))
    };
}

fn op(x: OperatorToken) -> ActionT {
    constant(Operator(x))
}

fn identifier(s: &[u8]) -> anyhow::Result<Option<Token>> {
    let t = std::str::from_utf8(s)?;
    Ok(Some(Identifier(t.to_owned())))
}

fn integer_literal(s: &[u8]) -> anyhow::Result<Option<Token>> {
    use IntegerToken::*;
    let i = s.partition_point(|x| (b'0'..=b'9').contains(x));
    let (l, r) = s.split_at(i);
    let t = std::str::from_utf8(l)?;
    let k: i64 = t.parse()?;
    let x = match r {
        b"" | b"l" => Ok(L(k as i32)),
        b"ll" => Ok(LL(k as i64)),
        b"u" | b"ul" | b"lu" => Ok(UL(k as u32)),
        b"ull" | b"llu" => Ok(ULL(k as u64)),
        _ => Err(anyhow!("integer_literal {}", t)),
    };
    Ok(Some(Literal(LInt(x?))))
}

pub fn clex() -> anyhow::Result<LexerT> {
    let v: Vec<(&str, ActionT)> = vec![
        ("//(.)*\n|/\\*(.|\n)*\\*/|(\\s)+", Action::C(None)),
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
        (r"\-=", op(SubAsn)),
        (r"\*=", op(MulAsn)),
        (r"/=", op(DivAsn)),
        (r"%=", op(ModAsn)),
        (r"&=", op(AndAsn)),
        (r"\^=", op(XorAsn)),
        (r"\|=", op(OrAsn)),
        (r">>", op(Shr)),
        (r"<<", op(Shl)),
        (r"\+\+", op(Inc)),
        (r"\-\-", op(Dec)),
        (r"\->", op(Ptr)),
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
        (r"\-", op(Minus)),
        (r"\+", op(Plus)),
        (r"\*", op(Ast)),
        (r"/", op(Div)),
        (r"%", op(Mod)),
        (r"<", op(Lt)),
        (r">", op(Gt)),
        (r"\^", op(Caret)),
        (r"\|", op(BitOr)),
        (r"\?", op(QnMk)),
        (r"[a-zA-Z_](\w)*", Action::F(identifier)),
        (r"(\-)?(\d)+([uUlL])*", Action::F(integer_literal)),
    ];
    Lexer::new(v.into_iter()).map_err(|s| anyhow!("clex:ParseRegexError {}", s.0))
}

pub fn tokens(lexer: LexerT, code: &str) -> anyhow::Result<Vec<Token>> {
    Ok(lexer
        .items(code.as_bytes())
        .collect::<anyhow::Result<Vec<Option<Token>>>>()?
        .into_iter()
        .filter_map(|x| x)
        .collect())
}

#[cfg(test)]
mod tests {
    use crate::clex::*;
    use lexer::combinator::Parser;

    #[test]
    fn clex_test() -> anyhow::Result<()> {
        let lexer = clex()?;
        let code = r#"
            int main() {
                return 0;
            }
        "#;
        let tokens = tokens(lexer, code)?;
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
        Ok(())
    }
}
