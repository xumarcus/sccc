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
    Tilde,
    Minus,
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
    LInt(IntegerToken),
    LFloat(FloatToken),
    LChar(u8),
    LString(String),
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
    Keyword(KeywordToken),
    Operator(OperatorToken),
    Literal(LiteralToken),
    Identifier(String),
}
