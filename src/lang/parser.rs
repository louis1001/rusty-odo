use anyhow::Context;

use crate::lang::lexer::{Token, TokenType};

pub struct Parser {
    // tokens is a peekable iterator on a collection of Tokens
    tokens: std::iter::Peekable<std::vec::IntoIter<Token>>
}

#[derive(Debug)]
enum Error {
    SuddenEndOfFile,
    UnexpectedToken(TokenType, Token), // Expected, got
}

impl Error {
    fn description(&self) -> String {
        match self {
            Error::UnexpectedToken(expected, got) => {
                format!("Expected token of type {:?} but got {:?}", expected, got)
            }
            Error::SuddenEndOfFile => "Unexpected end of file".to_string(),
        }
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
       write!(f, "{}", self.description())
    }
}

impl std::error::Error for Error {}

// The AST
pub type Node = Box<Ast>;

#[derive(Debug, Clone)]
pub enum Ast {
    Block(Vec<Node>),
    Number(Token),
    Variable(Token),
    Assignment(Token, Node),
    Declaration(Token, Node),
    DebugPrint(Node) // Temporary
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Parser {
        Parser {
            tokens: tokens.into_iter().peekable()
        }
    }

    fn consume(&mut self, kind: TokenType) -> anyhow::Result<Token> {
        // we don't unwrap, we use anyhow and context
        let current_token = match self.tokens.peek() {
            Some(token) => Ok(token),
            None => Err(Error::SuddenEndOfFile)
        }?;

        if kind == current_token.token_type {
            Ok(self.tokens.next().unwrap())
        } else {
            Err(Error::UnexpectedToken(kind.clone(), current_token.clone()))
                .context(format!("Expected token of type {:?} but got {:?}", kind, current_token))
        }
    }

    pub fn parse(&mut self) -> anyhow::Result<Node> {
        let mut ast: Vec<Node> = Vec::new();
        
        while let Some(_) = self.tokens.peek() {
            ast.push(self.parse_statement()?);
        }
        
        Ok(Box::new(Ast::Block(ast)))
    }

    pub fn parse_statement(&mut self) -> anyhow::Result<Node> {
        // Current Ast kinds of statement: 
        // - Assignment
        // - DebugPrint

        match self.tokens.peek().unwrap().token_type {
            TokenType::Var => self.parse_declaration(),
            TokenType::Name => self.parse_assignment(),
            TokenType::DebugPrint => {
                self.consume(TokenType::DebugPrint).unwrap();
                let expr = self.parse_expr()?;

                Ok(Box::new(Ast::DebugPrint(expr)))
            },
            _ => return Err(anyhow::anyhow!("Unexpected token {:?}", self.tokens.peek().unwrap().token_type))
        }
    }

    fn parse_declaration(&mut self) -> anyhow::Result<Node> {
        let _ = self.consume(TokenType::Var)?;
        let name = self.consume(TokenType::Name)?;
        let _ = self.consume(TokenType::Assign)
            .context("Expected an assignment statement ('=')")?;
        let expr = self.parse_expr()?;

        Ok(Box::new(Ast::Declaration(name, expr)))
    }

    fn parse_assignment(&mut self) -> anyhow::Result<Node> {
        let name = self.consume(TokenType::Name)?;
        self.consume(TokenType::Assign)
            .context("Expected an assignment statement ('=')")?;
        let expr = self.parse_expr()?;

        Ok(Box::new(Ast::Assignment(name, expr)))
    }

    fn parse_expr(&mut self) -> anyhow::Result<Node> {
        match self.tokens.peek().ok_or(Error::SuddenEndOfFile)?.token_type {
            TokenType::Number => {
                let token = self.tokens.next().ok_or(Error::SuddenEndOfFile)?;
                Ok(Box::new(Ast::Number(token)))
            },
            TokenType::Name => {
                Ok(Box::new(Ast::Variable(self.tokens.next().ok_or(Error::SuddenEndOfFile)?)))
            },
            _ => return Err(anyhow::anyhow!("Unexpected token {:?}", self.tokens.peek().unwrap().token_type))
        }
    }
}

#[cfg(test)]
mod tests {
    fn parser(input: &str) -> crate::lang::parser::Parser {
        use crate::lang::lexer::Lexer;

        let lexer = Lexer::new(input.to_string());
        let tokens: Vec<_> = lexer.collect();

        crate::lang::parser::Parser::new(tokens)
    }

    #[test]
    fn test_parse_declaration() {
        let mut parser = parser("var x = 1");
        let ast = parser.parse_statement().unwrap();

        assert_eq!(format!("{:?}", ast), "Declaration(Token { token_type: Name, value: \"x\", line: 1, column: 4 }, Number(Token { token_type: Number, value: \"1\", line: 1, column: 8 }))");
    }

    #[test]
    fn test_parse_assignment() {
        let mut parser = parser("x = 1");
        let ast = parser.parse_statement().unwrap();

        assert_eq!(format!("{:?}", ast), "Assignment(Token { token_type: Name, value: \"x\", line: 1, column: 0 }, Number(Token { token_type: Number, value: \"1\", line: 1, column: 4 }))");
    }
}