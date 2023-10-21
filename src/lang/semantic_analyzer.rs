use crate::lang::parser::Ast;

use std::{collections::HashMap, result};
use uuid::Uuid;
use lazy_static::lazy_static;

use super::{parser::Node, lexer::Token};

pub struct SemanticAnalyzer {
    pub symbol_table: SymbolTable
}

impl SemanticAnalyzer {
    pub fn new() -> SemanticAnalyzer {
        SemanticAnalyzer {
            symbol_table: SymbolTable {
                identifier: "global_scope".to_string(),
                symbols: {
                    let mut map = HashMap::new();
                    // Add the primitive types to the symbol table
                    map.insert(INT_TYPE.symbol_id, INT_TYPE.clone());
                    map.insert(DEC_TYPE.symbol_id, DEC_TYPE.clone());
                    map.insert(STRING_TYPE.symbol_id, STRING_TYPE.clone());
                    map.insert(BOOL_TYPE.symbol_id, BOOL_TYPE.clone());
                
                    map
                }
            }
        }
    }
}

lazy_static! {
    /// This stores the primitive types
    static ref INT_TYPE: Symbol = Symbol::new("int".to_string(), SymbolVariant::Primitive);
    static ref DEC_TYPE: Symbol = Symbol::new("dec".to_string(), SymbolVariant::Primitive); // Equivalent to float
    static ref STRING_TYPE: Symbol = Symbol::new("string".to_string(), SymbolVariant::Primitive);
    static ref BOOL_TYPE: Symbol = Symbol::new("bool".to_string(), SymbolVariant::Primitive);
}

pub type SemanticNode = Box<SemanticAst>;

#[derive(Debug)]
pub enum SemanticAst {
    Block(Vec<SemanticAst>),
    Number(Token),
    Variable(Token),
    // It should also store the infered type
    Declaration(Token, Uuid, Box<SemanticAst>),
    Assignment(Token, Box<SemanticAst>),
    DebugPrint(Box<SemanticAst>)
}

pub struct SymbolTable {
    identifier: String,
    symbols: HashMap<Uuid, Symbol>
}

impl SymbolTable {
    // Lookup by name
    pub fn lookup(&self, name: String) -> Option<&Symbol> {
        for symbol in self.symbols.values() {
            if symbol.name == name {
                return Some(symbol);
            }
        }

        None
    }

    // Lookup by id
    pub fn lookup_id(&self, id: Uuid) -> Option<&Symbol> {
        self.symbols.get(&id)
    }
}

#[derive(Clone)]
pub struct Symbol {
    name: String,
    pub symbol_id: Uuid,
    variant: SymbolVariant
}

impl Symbol {
    pub fn new(name: String, kind: SymbolVariant) -> Self {
        Symbol {
            name: name,
            symbol_id: Uuid::new_v4(),
            variant: kind
        }
    }
}

#[derive(Clone)]
pub enum SymbolVariant {
    Variable(Variable),
    Primitive // Primitives only need their name
}

// Symbol variants:
#[derive(Clone)]
pub struct Variable {
    type_id: Uuid
}

// Semantic analysis

/// This is what is returned when a grammatical Node is analyzed
#[derive(Debug)]
pub struct SemanticResult {
    pub node: SemanticNode,
    type_id: Option<Uuid>,
    // More context to be added later...
    // Does this node have side effects, for example.
}

impl SemanticAnalyzer {
    pub fn analyze(&mut self, ast: Node) -> anyhow::Result<SemanticResult> {
        let ast = ast.clone();
        let semantic_ast = self.analyze_node(ast)?;

        println!("{:#?}", semantic_ast.node);

        Ok(semantic_ast)
    }

    pub fn analyze_node(&mut self, ast: Node) -> anyhow::Result<SemanticResult> {
        match *ast {
            Ast::Block(nodes) => {
                let mut semantic_nodes = Vec::new();

                for node in nodes {
                    semantic_nodes.push(*self.analyze_node(node)?.node);
                }

                let node = SemanticAst::Block(semantic_nodes);

                Ok(SemanticResult {
                    node: Box::new(node),
                    type_id: None
                })
            },
            Ast::Number(token) => {
                let node = SemanticAst::Number(token);

                Ok(SemanticResult {
                    node: Box::new(node),
                    type_id: Some(INT_TYPE.symbol_id)
                })
            },
            Ast::Variable(token) => {
                let node = SemanticAst::Variable(token.clone());

                // lookup the variable and return it's type
                let symbol = self.symbol_table.lookup(token.value.clone())
                    .ok_or(anyhow::anyhow!("Variable {} not found", token.value))?;

                let type_id = match symbol.variant {
                    SymbolVariant::Variable(ref var) => var.type_id,
                    _ => panic!("Symbol is not a variable")
                };

                Ok(SemanticResult {
                    node: Box::new(node),
                    type_id: Some(type_id)
                })
            },
            Ast::Declaration(token, node) => {
                let result_node = self.analyze_node(node)?;

                // Analyze the initialization node and get its type
                let type_id = result_node.type_id
                    .ok_or(anyhow::anyhow!("Variable initialization must be a valid expression (Must return value)"))?;

                // Check if the variable has already been declared
                if self.symbol_table.lookup(token.value.clone()).is_some() {
                    return Err(anyhow::anyhow!("Variable called {} already exists.", token.value));
                }

                // Create a new symbol and insert it into the symbol table
                let symbol = Symbol::new(token.value.clone(), SymbolVariant::Variable(Variable {
                    type_id: type_id
                }));

                self.symbol_table.symbols.insert(symbol.symbol_id, symbol.clone());

                let node = SemanticAst::Declaration(token, symbol.symbol_id, result_node.node);

                Ok(SemanticResult {
                    node: Box::new(node),
                    type_id: None
                })
            },
            Ast::Assignment(token, node) => {
                let result_node = self.analyze_node(node)?;

                // Lookup the variable and get its type
                let symbol = self.symbol_table.lookup(token.value.clone()).ok_or(anyhow::anyhow!("Assignment to unknown variable {}.", token.value))?;

                // TODO: Expand the kinds of symbol that can be assigned to
                let type_id = match symbol.variant {
                    SymbolVariant::Variable(ref var) => var.type_id,
                    _ => panic!("Symbol is not a variable")
                };

                // Check if the type of the assignment is the same as the type of the variable
                if type_id != result_node.type_id.unwrap() {
                    return Err(anyhow::anyhow!("Type mismatch: Expected type {:?} but got type {:?}", type_id, result_node.type_id.unwrap()));
                }

                let node = SemanticAst::Assignment(token, result_node.node);

                Ok(SemanticResult {
                    node: Box::new(node),
                    type_id: None
                })
            },
            Ast::DebugPrint(node) => {
                let result_node = self.analyze_node(node)?;

                // This is not important. Just check that there's a value to print (type_id is some).
                let _ = result_node.type_id.ok_or(anyhow::anyhow!("DebugPrint must be a valid expression (Must return value)"))?;
                // Return nothing

                let node = SemanticAst::DebugPrint(result_node.node);

                Ok(SemanticResult {
                    node: Box::new(node),
                    type_id: None
                })
            }
        }
    }
}