// https://www.lysator.liu.se/c/ANSI-C-grammar-l.html

use std::str::from_utf8;

use lexer::Lexer;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KeywordToken {
    Auto,
    Break,
    Case,
    Char,
    Const,
    Continue,
    Default,
    Do,
    Double,
    Else,
    Enum,
    Extern,
    Float,
    For,
    Goto,
    If,
    Int,
    Long,
    Register,
    Return,
    Short,
    Signed,
    Sizeof,
    Static,
    Struct,
    Switch,
    Typedef,
    Union,
    Unsigned,
    Void,
    Volatile,
    While,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OperatorToken {
    Ellipsis,
    ShrAsn,
    ShlAsn,
    AddAsn,
    SubAsn,
    MulAsn,
    DivAsn,
    ModAsn,
    AndAsn,
    XorAsn,
    OrAsn,
    Shr,
    Shl,
    Inc,
    Dec,
    Ptr,
    And,
    Or,
    Le,
    Ge,
    Eq,
    Ne,
    Semicolon,
    LBrace,
    RBrace,
    Comma,
    Colon,
    Assign,
    LParen,
    RParen,
    LSqBr,
    RSqBr,
    Dot,
    BitAnd,
    Not,
    Minus,
    Tilde,
    Plus,
    Ast,
    Div,
    Mod,
    Lt,
    Gt,
    Caret,
    BitOr,
    QnMk,
}

#[derive(Clone, Debug, PartialEq)]
pub enum LiteralToken {
    Integer(IntegerToken),
    Float(FloatToken),
    Char(u8),
    String(String),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IntegerToken {
    L(i32),
    LL(i64),
    UL(u32),
    ULL(u64),
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum FloatToken {
    F(f32),
    L(f64),
}

#[derive(Clone, Debug, PartialEq)]
pub enum Token {
    Comment(String),
    Keyword(KeywordToken),
    Operator(OperatorToken),
    Literal(LiteralToken),
    Identifier(String),
    Space,
}

macro_rules! constant {
    ($x:expr) => {
        Box::new(|_| $x)
    };
}

use FloatToken::*;
use IntegerToken::*;
use KeywordToken::*;
use OperatorToken::*;
use Token::*;

pub fn clex() -> lexer::Result<Lexer<Token>> {
    let v: Vec<(&str, lexer::Action<Token>)> = vec![
        (r"auto", constant!(Keyword(Auto))),
        (r"break", constant!(Keyword(Break))),
        (r"case", constant!(Keyword(Case))),
        (r"char", constant!(Keyword(Char))),
        (r"const", constant!(Keyword(Const))),
        (r"continue", constant!(Keyword(Continue))),
        (r"default", constant!(Keyword(Default))),
        (r"do", constant!(Keyword(Do))),
        (r"double", constant!(Keyword(Double))),
        (r"else", constant!(Keyword(Else))),
        (r"enum", constant!(Keyword(Enum))),
        (r"extern", constant!(Keyword(Extern))),
        (r"float", constant!(Keyword(Float))),
        (r"for", constant!(Keyword(For))),
        (r"goto", constant!(Keyword(Goto))),
        (r"if", constant!(Keyword(If))),
        (r"int", constant!(Keyword(Int))),
        (r"long", constant!(Keyword(Long))),
        (r"register", constant!(Keyword(Register))),
        (r"return", constant!(Keyword(Return))),
        (r"short", constant!(Keyword(Short))),
        (r"signed", constant!(Keyword(Signed))),
        (r"sizeof", constant!(Keyword(Sizeof))),
        (r"static", constant!(Keyword(Static))),
        (r"struct", constant!(Keyword(Struct))),
        (r"switch", constant!(Keyword(Switch))),
        (r"typedef", constant!(Keyword(Typedef))),
        (r"union", constant!(Keyword(Union))),
        (r"unsigned", constant!(Keyword(Unsigned))),
        (r"void", constant!(Keyword(Void))),
        (r"volatile", constant!(Keyword(Volatile))),
        (r"while", constant!(Keyword(While))),
        (r";", constant!(Operator(Semicolon))),
        (r"\(", constant!(Operator(LParen))),
        (r"\)", constant!(Operator(RParen))),
        (r"\[", constant!(Operator(LSqBr))),
        (r"\]", constant!(Operator(RSqBr))),
        (r"{", constant!(Operator(LBrace))),
        (r"}", constant!(Operator(RBrace))),
        (
            r"\l(\l|\d)*",
            Box::new(|s| {
                Identifier(from_utf8(s).expect("identifier").to_owned())
            }),
        ),
        (r"(\s)+", constant!(Space)),
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
                return true;
            }
        "#;
        assert_eq!(
            lexer.items(code.as_bytes()).filter(|x| x != &Space).count(),
            9
        );
    }
}
