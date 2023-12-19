pub enum AST {
    Digit,
    Letter,
    CharacterClass(Vec<u8>),
    String(String),
    Concat(Vec<AST>),
    Union(Vec<AST>),
    Star(Box<AST>),
    Plus(Box<AST>),
    Question(Box<AST>),
}
