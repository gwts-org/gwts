use anyhow::Result;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TokenKind {
    /// Starts with a letter and then contains any number of letters, digits, and `_`.
    Alphanumeric(String),
    /// Starts with a number and then contains any number of letters, digits, and `_`.
    Numeric(String),
    /// `,`
    Comma,
    /// `:`
    Colon,
    /// `;`
    Semicolon,
    /// `[`
    OpenBracket,
    /// `]`
    CloseBracket,
    /// `{`
    OpenCurly,
    /// `}`
    CloseCurly,
}

#[derive(Clone, Debug)]
pub struct Token {
    pub kind: TokenKind,
    pub start: usize,
    pub len: usize,
}

pub fn tokens(content: &str) -> Result<Vec<Token>> {
    let mut tokens: Vec<Token> = Vec::new();

    let mut offs: usize = 0;
    let mut content = content;
    while !content.is_empty() {
        // Skip whitespace.
        content = content.trim_start();

        // Test first char.
        let Some(c) = content.chars().next() else {
            break;
        };

        // Capture the content into the token
        let mut len: usize = 1;
        let kind: TokenKind = if c.is_alphabetic() {
            len = content
                .chars()
                .position(|c| !c.is_alphanumeric() && c != '_')
                .unwrap();
            TokenKind::Alphanumeric(content[..len].to_string())
        } else if c.is_numeric() {
            len = content
                .chars()
                .position(|c| !c.is_alphanumeric() && c != '_')
                .unwrap();
            TokenKind::Numeric(content[..len].to_string())
        } else {
            match c {
                ',' => TokenKind::Comma,
                ':' => TokenKind::Colon,
                ';' => TokenKind::Semicolon,
                '[' => TokenKind::OpenBracket,
                ']' => TokenKind::CloseBracket,
                '{' => TokenKind::OpenCurly,
                '}' => TokenKind::CloseCurly,
                _ => panic!("tokenizer hit an unhandled character: {:?}", c),
            }
        };

        tokens.push(Token {
            kind,
            start: offs,
            len,
        });

        content = &content[len..];
        offs += len;
    }

    Ok(tokens)
}
