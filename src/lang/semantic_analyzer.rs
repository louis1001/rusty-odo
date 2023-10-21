use crate::lang::parser::Ast;

use std::collections::HashMap;
use uuid::Uuid;
use lazy_static::lazy_static;

use super::{parser::Node, lexer::Token};

pub struct SemanticAnalyzer {
    scopes: HashMap<Uuid, SymbolTable>,
    current_scope_id: Uuid
}

impl SemanticAnalyzer {
    pub fn new() -> SemanticAnalyzer {
        let global_table = SymbolTable::new("global_table".to_string());
        let id = global_table.table_id;
        
        SemanticAnalyzer {
            scopes: {
                let mut map = HashMap::new();
                map.insert(global_table.table_id, global_table);
                map
            },
            current_scope_id: id
        }
    }

    pub fn current_scope(&self) -> Option<&SymbolTable> {
        self.scopes.get(&self.current_scope_id)
    }

    fn current_scope_mut(&mut self) -> Option<&mut SymbolTable> {
        self.scopes.get_mut(&self.current_scope_id)
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
    Truth(Token),
    Variable(Token),
    // It should also store the infered type
    Declaration(Token, Uuid, Box<SemanticAst>),
    Assignment(Token, Box<SemanticAst>),
    DebugPrint(Box<SemanticAst>)
}

pub struct SymbolTable {
    name: String,
    table_id: Uuid,
    parent: Option<Uuid>,
    symbols: HashMap<Uuid, Symbol>
}

impl SymbolTable {
    pub fn new(name: String) -> Self {
        SymbolTable {
            name,
            table_id: Uuid::new_v4(),
            parent: None,
            symbols: HashMap::new()
        }
    }

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
    pub variant: SymbolVariant
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
            Ast::Truth(token) => {
                let node = SemanticAst::Truth(token);

                Ok(SemanticResult {
                    node: Box::new(node),
                    type_id: Some(BOOL_TYPE.symbol_id)
                })
            },
            Ast::Variable(token) => {
                let node = SemanticAst::Variable(token.clone());

                // lookup the variable and return it's type
                let symbol = self.current_scope().expect("There's always a scope").lookup(token.value.clone())
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
                if self.current_scope().expect("There's always a scope").lookup(token.value.clone()).is_some() {
                    return Err(anyhow::anyhow!("Variable called {} already exists.", token.value));
                }

                // Create a new symbol and insert it into the symbol table
                let symbol = Symbol::new(token.value.clone(), SymbolVariant::Variable(Variable {
                    type_id: type_id
                }));

                self.current_scope_mut()
                    .expect("There's always a scope")
                    .symbols.insert(symbol.symbol_id, symbol.clone());

                let node = SemanticAst::Declaration(token, symbol.symbol_id, result_node.node);

                Ok(SemanticResult {
                    node: Box::new(node),
                    type_id: None
                })
            },
            Ast::Assignment(token, node) => {
                let result_node = self.analyze_node(node)?;

                // Lookup the variable and get its type
                let symbol = self.current_scope().expect("There's always a scope")
                    .lookup(token.value.clone())
                    .ok_or(anyhow::anyhow!("Assignment to unknown variable {}.", token.value))?;

                // TODO: Expand the kinds of symbol that can be assigned to
                let type_id = match symbol.variant {
                    SymbolVariant::Variable(ref var) => var.type_id,
                    _ => panic!("Symbol is not a variable")
                };

                // Check if the type of the assignment is the same as the type of the variable
                if result_node.type_id.ok_or(anyhow::anyhow!("Assignment must be a valid expression (Must return value)"))? != type_id {
                    return Err(
                        anyhow::anyhow!(
                            "Type mismatch: Expected type {:?} but got type {:?}",
                            self.name_of_type(type_id).unwrap_or("<unknown>"),
                            result_node.type_id.map(|id| self.name_of_type(id)).unwrap_or(Some("<unknown>")).unwrap_or("<unknown>") // FIXME: This is ugly
                        )
                    );
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

// For report purposes
impl SemanticAnalyzer {
    fn name_of_type(&self, id: Uuid) -> Option<&str> {
        let symbol = self.current_scope().expect("There's always a scope").lookup_id(id);

        symbol.map(|symbol| symbol.name.as_str())
    }
}