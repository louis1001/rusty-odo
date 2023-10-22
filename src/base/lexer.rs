use std::collections::HashMap;

use lazy_static::lazy_static;

#[derive(Debug, Clone)]
pub struct Token {
    pub token_type: TokenType,
    pub value: String,
    line: usize,
    column: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TokenType {
    // for now, just a variable assignment and number type
    Var, // var a = 10.0
    Name, // a
    Number, // 10.0
    Truth,
    Assign, // =

    NewLine, // \n

    SemiColon, // ;

    LeftCurly, // {
    RightCurly, // }

    // Control flow
    If,

    DebugPrint // ':' - Temporary
}

pub struct Lexer {
    code: String,
    position: usize,
    current_line: usize,
    current_column: usize,
}

impl Lexer {
    pub fn new(code: String) -> Lexer {
        Lexer {
            code,
            position: 0,
            current_line: 1,
            current_column: 0,
        }
    }

    fn current_char(&self) -> Option<char> {
        self.code.chars().nth(self.position)
    }

    fn advance(&mut self) {
        self.position += 1;
        self.current_column += 1;

        if let Some('\n') = self.current_char() {
            self.current_line += 1;
            self.position += 1;
            self.current_column = 0;
        }
    }

    fn ignore_whitespace(&mut self) {
        while let Some(c) = self.current_char() {
            if !c.is_whitespace() || c == '\n' {
                break;
            }

            self.advance();
        }
    }
}

lazy_static! {
    static ref KEYWORDS: HashMap<String, TokenType> = {
        let mut map = HashMap::new();
        map.insert("var".to_string(), TokenType::Var);
        map.insert("true".to_string(), TokenType::Truth);
        map.insert("false".to_string(), TokenType::Truth);
        map.insert("if".to_string(), TokenType::If);
        map
    };
}

// Implement the Iterator trait for Lexer
impl Iterator for Lexer {
    type Item = Token;

    fn next(&mut self) -> Option<Token> {
        self.ignore_whitespace();
        if let Some(curr) = self.current_char() {
            let mut token = Token {
                token_type: TokenType::Var,
                value: String::new(),
                line: self.current_line,
                column: self.current_column,
            };

            if curr.is_alphabetic() {
                token.token_type = TokenType::Name;
                while let Some(c) = self.current_char() {
                    if !c.is_alphabetic() {
                        break;
                    }

                    token.value.push(c);
                    self.advance();
                }

                if let Some(token_type) = KEYWORDS.get(&token.value) {
                    token.token_type = token_type.clone();
                } else {
                    token.token_type = TokenType::Name;
                }
            } else if curr.is_numeric() {
                token.token_type = TokenType::Number;
                while let Some(c) = self.current_char() {
                    if !c.is_numeric() {
                        break;
                    }

                    token.value.push(c);
                    self.advance();
                }
            } else if curr == '\n' {
                token.token_type = TokenType::NewLine;
                token.value.push(curr);
                self.advance();
            } else if curr == '=' {
                token.token_type = TokenType::Assign;
                token.value.push(curr);
                self.advance();
            } else if curr == '{' {
                token.token_type = TokenType::LeftCurly;
                token.value.push(curr);
                self.advance();
            } else if curr == '}' {
                token.token_type = TokenType::RightCurly;
                token.value.push(curr);
                self.advance();
            } else if curr == ';' {
                token.token_type = TokenType::SemiColon;
                token.value.push(curr);
                self.advance();
            } else if curr == ':' {
                token.token_type = TokenType::DebugPrint;
                token.value.push(curr);
                self.advance();
            } else {
                panic!("Unexpected character: {}", curr);
            }

            Some(token)
        } else {
            None
        }
    }
}