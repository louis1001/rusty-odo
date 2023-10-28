use anyhow::Context;

use crate::base::lexer::{Token, TokenType};

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
    Truth(Token),
    Text(Token),
    Variable(Token),
    Assignment(Node, Node),
    Declaration(Token, Node),

    FunctionCall(Node, Vec<Node>),

    // Control flow
    If(Node, Node /*, Option<Node> */),

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

    fn next_is(&mut self, kind: TokenType) -> bool {
        match self.tokens.peek() {
            Some(token) => token.token_type == kind,
            None => false
        }
    }

    fn ignore_newline(&mut self) {
        while let Some(token) = self.tokens.peek() {
            if token.token_type == TokenType::NewLine {
                let _ = self.consume(TokenType::NewLine);
            } else {
                break;
            }
        }
    }

    pub fn parse(&mut self) -> anyhow::Result<Node> {
        let mut ast: Vec<Node> = Vec::new();
        
        while let Some(_) = self.tokens.peek() {
            ast.push(self.parse_statement()?);
        }
        
        Ok(Box::new(Ast::Block(ast)))
    }

    pub fn statement_list(&mut self) -> anyhow::Result<Vec<Node>> {
        let mut ast: Vec<Node> = Vec::new();

        self.ignore_newline();

        if let Ok(_) = self.consume(TokenType::RightCurly) {
            return Ok(ast);
        }
        
        while let Some(_) = self.tokens.peek() {
            // check terminators
            if self.tokens.peek().unwrap().token_type == TokenType::RightCurly {
                break;
            }
            
            ast.push(self.parse_statement()?);
        }
        
        Ok(ast)
    }

    fn check_statement_terminator(&mut self) -> anyhow::Result<()> {
        // Consume statement terminators
        let token = match self.tokens.peek() {
            Some(token) => token,
            None => return Ok(()) // End of file is a valid statement terminator
        };

        let block_terminators = vec![TokenType::RightCurly]; // Anything that would work as termination in a wrap block

        if token.token_type == TokenType::SemiColon {
            let _ = self.consume(TokenType::SemiColon)?;

            // Consume all new lines after this
            while let Some(token) = self.tokens.peek() {
                if token.token_type == TokenType::NewLine {
                    let _ = self.consume(TokenType::NewLine)?;
                } else {
                    break;
                }
            }
            // if it's not one of the block terminators
        } else if !block_terminators.contains(&token.token_type) {
            // Expect at least one new line, and consume all others
            let _ = self.consume(TokenType::NewLine)?;

            while let Some(token) = self.tokens.peek() {
                if token.token_type == TokenType::NewLine {
                    let _ = self.consume(TokenType::NewLine)?;
                } else {
                    break;
                }
            }
        }

        Ok(())
    }

    pub fn parse_statement(&mut self) -> anyhow::Result<Node> {
        let result = self.parse_statement_without_terminator()?;

        self.check_statement_terminator()?;

        Ok(result)
    }

    pub fn parse_statement_without_terminator(&mut self) -> anyhow::Result<Node> {
        // Current Ast kinds of statement: 
        // - Assignment
        // - Block
        // - DebugPrint

        self.ignore_newline();

        match self.tokens.peek().unwrap().token_type {
            TokenType::Var => self.parse_declaration(),
            TokenType::LeftCurly => self.parse_block(),
            TokenType::If => self.parse_if(),
            TokenType::DebugPrint => {
                self.consume(TokenType::DebugPrint).unwrap();
                let expr = self.parse_postfix()?;

                Ok(Box::new(Ast::DebugPrint(expr)))
            },
            _ => self.parse_postfix()
        }
    }

    fn parse_block(&mut self) -> anyhow::Result<Node> {
        let _ = self.consume(TokenType::LeftCurly)?;
        self.ignore_newline();
        let mut nodes = Vec::new();

        while let Some(token) = self.tokens.peek().cloned() {
            if token.token_type == TokenType::RightCurly {
                break;
            }

            nodes.push(self.parse_statement()?);
        }

        let _ = self.consume(TokenType::RightCurly)?;

        Ok(Box::new(Ast::Block(nodes)))
    }

    fn parse_declaration(&mut self) -> anyhow::Result<Node> {
        let _ = self.consume(TokenType::Var)?;
        self.ignore_newline();

        let name = self.consume(TokenType::Name)?;
        let _ = self.consume(TokenType::Assign)
            .context("Expected an assignment statement ('=')")?;
        let expr = self.parse_postfix()?;

        Ok(Box::new(Ast::Declaration(name, expr)))
    }

    fn parse_assignment(&mut self, target_node: Node) -> anyhow::Result<Node> {
        // TODO: Make sure the assignment target is valid
        self.ignore_newline();

        self.consume(TokenType::Assign)
            .context("Expected an assignment statement ('=')")?;
        let expr = self.parse_postfix()?;

        Ok(Box::new(Ast::Assignment(target_node, expr)))
    }

    fn parse_function_call(&mut self, callee: Node) -> anyhow::Result<Node> {
        self.ignore_newline();
        let _ = self.consume(TokenType::LeftParen)?;
        self.ignore_newline();

        let mut args = Vec::new();

        loop {
            match self.tokens.peek() {
                Some(token) if token.token_type == TokenType::RightParen => break,
                Some(_) => {},
                None => break,
            };
            
            args.push(self.parse_postfix()?);

            self.ignore_newline();

            if self.next_is(TokenType ::Comma) { self.consume(TokenType::Comma)?; }
            else { break; }
        }

        let _ = self.consume(TokenType::RightParen)?;

        Ok(Box::new(Ast::FunctionCall(callee, args)))
    }

    fn parse_postfix(&mut self) -> anyhow::Result<Node> {
        let mut expr = self.parse_factor()?;

        self.ignore_newline();

        while let Some(token) = self.tokens.peek().cloned() {
            match token.token_type {
                TokenType::Assign => {
                    expr = self.parse_assignment(expr)?;
                },
                TokenType::LeftParen => {
                    expr = self.parse_function_call(expr)?;
                }
                _ => break
            }
        }

        Ok(expr)
    }

    fn parse_factor(&mut self) -> anyhow::Result<Node> {
        self.ignore_newline();

        match self.tokens.peek().ok_or(Error::SuddenEndOfFile)?.token_type {
            TokenType::Number => {
                let token = self.tokens.next().expect("We just peeked");
                Ok(Box::new(Ast::Number(token)))
            },
            TokenType::Truth => {
                let token = self.tokens.next().expect("We just peeked");
                Ok(Box::new(Ast::Truth(token)))
            },
            TokenType::Text => {
                let token = self.tokens.next().expect("We just peeked");
                Ok(Box::new(Ast::Text(token)))
            },
            TokenType::Name => {
                Ok(Box::new(Ast::Variable(self.tokens.next().expect("We just peeked"))))
            },
            _ => return Err(anyhow::anyhow!("Unexpected token {:?}", self.tokens.peek().expect("We just peeked").token_type))
        }
    }
}

// Control flow implementations
impl Parser {
    fn parse_if(&mut self) -> anyhow::Result<Node> {
        let _ = self.consume(TokenType::If)?;
        let condition = self.parse_postfix()?;
        let body = self.parse_statement()?;

        Ok(Box::new(Ast::If(condition, body)))
    }
}

#[cfg(test)]
mod tests {
    fn parser(input: &str) -> crate::base::parser::Parser {
        use crate::base::lexer::Lexer;

        let lexer = Lexer::new(input.to_string());
        let tokens: Vec<_> = lexer.collect();

        crate::base::parser::Parser::new(tokens)
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