
use crate::{Analyser, Lexer, Parseable, Tokenizer};
use crate::param::ParamFile;

type ParamLexer = Lexer;
struct ParamParser;

enum ParamToken {
    ClassKeyword,
    DeleteKeyword,
    EnumKeyword,
    SemiColon,
    Colon,
    LeftCurly,
    RightCurly,
    LeftSquare,
    RightSquare,
    Assign,
    AddAssign,
    SubAssign,
    Comma,
    Identifier([u8]),
    String {
        quoted: bool,
        data: [u8]
    },
    Integer(i32),
    Float(f32),
    Double(f64),
    Unknown([u8])

}

impl Parseable for ParamFile {
    type E = ();
    type P = ();

    fn try_parse(lexer: &mut Lexer) -> Result<Self, Self::E> {
        todo!()
    }
}

impl Tokenizer for ParamLexer {
    type Token = ParamToken;

    fn next_token(&mut self) -> &mut Self::Token {
        todo!()
    }

}
