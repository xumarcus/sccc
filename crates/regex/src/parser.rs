pub enum AST {
    Digit,
    Letter,
    Character(u8),
    CharacterClass(Vec<u8>),
    String(String),
    Concat(Vec<AST>),
    Union(Vec<AST>),
    Star(Box<AST>),
    Plus(Box<AST>),
    Question(Box<AST>),
}
