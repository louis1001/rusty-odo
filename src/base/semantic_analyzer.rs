use crate::base::parser::Ast;

use std::collections::HashMap;
use uuid::Uuid;
use lazy_static::lazy_static;

use super::{parser::Node, lexer::Token};

pub struct SemanticAnalyzer {
    scopes: HashMap<Uuid, SymbolTable>,
    pub current_scope_id: TableId,
    pub repl_scope_id: TableId,
    global_scope_id: TableId
}

impl SemanticAnalyzer {
    pub fn new() -> SemanticAnalyzer {
        let mut global_table = SymbolTable::new("global_table".to_string());
        // Primitive types
        global_table.symbols.insert(INT_TYPE.symbol_id, INT_TYPE.clone());
        global_table.symbols.insert(DEC_TYPE.symbol_id, DEC_TYPE.clone());
        global_table.symbols.insert(TEXT_TYPE.symbol_id, TEXT_TYPE.clone());
        global_table.symbols.insert(TRUTH_TYPE.symbol_id, TRUTH_TYPE.clone());

        let id = global_table.table_id;

        let mut repl_scope = SymbolTable::new("repl_scope".to_string());
        let repl_scope_id = repl_scope.table_id;
        repl_scope.parent = Some(id);
        
        SemanticAnalyzer {
            scopes: {
                let mut map = HashMap::new();
                map.insert(global_table.table_id, global_table);
                map.insert(repl_scope.table_id, repl_scope);
                map
            },
            current_scope_id: id,
            repl_scope_id,
            global_scope_id: id
        }
    }

    pub fn global_scope(&self) -> anyhow::Result<&SymbolTable> {
        self.scopes.get(&self.global_scope_id)
            .ok_or(anyhow::anyhow!("There should always be a global scope"))
    }

    pub fn global_scope_mut(&mut self) -> anyhow::Result<&mut SymbolTable> {
        self.scopes.get_mut(&self.global_scope_id)
            .ok_or(anyhow::anyhow!("There should always be a global scope"))
    }

    pub fn current_scope(&self) -> anyhow::Result<&SymbolTable> {
        self.scopes.get(&self.current_scope_id)
            .ok_or(anyhow::anyhow!("There should always be a scope"))
    }

    pub fn current_scope_mut(&mut self) -> anyhow::Result<&mut SymbolTable> {
        self.scopes.get_mut(&self.current_scope_id)
        .ok_or(anyhow::anyhow!("There should always be a scope"))
    }
}

lazy_static! {
    /// This stores the primitive types
    static ref INT_TYPE: Symbol = Symbol::new("int".to_string(), SymbolVariant::Primitive);
    static ref DEC_TYPE: Symbol = Symbol::new("dec".to_string(), SymbolVariant::Primitive); // Equivalent to float
    static ref TEXT_TYPE: Symbol = Symbol::new("string".to_string(), SymbolVariant::Primitive);
    static ref TRUTH_TYPE: Symbol = Symbol::new("truth".to_string(), SymbolVariant::Primitive);
}

pub type SemanticNode = Box<SemanticAst>;

#[derive(Debug)]
pub enum SemanticAst {
    Block(Vec<SemanticAst>, TableId),
    Number(Token),
    Truth(Token),
    Text(Token),
    Variable(SymbolId),
    // It should also store the infered type
    Declaration(SymbolId, Uuid, SemanticNode),
    Assignment(SymbolId, SemanticNode),
    FunctionCall(SemanticNode, Vec<SemanticNode>),
    If(SemanticNode, SemanticNode),
    DebugPrint(SemanticNode)
}

type TableId = Uuid;

pub struct SymbolTable {
    #[allow(dead_code)]
    name: String,
    table_id: TableId,
    parent: Option<TableId>,
    symbols: HashMap<TableId, Symbol>
}

impl SymbolTable {
    pub fn new(name: String) -> Self {
        SymbolTable {
            name,
            table_id: TableId::new_v4(),
            parent: None,
            symbols: HashMap::new()
        }
    }

    pub fn insert(&mut self, symbol: Symbol) {
        self.symbols.insert(symbol.symbol_id, symbol);
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
    pub fn lookup_id(&self, id: SymbolId) -> Option<&Symbol> {
        self.symbols.get(&id)
    }
}

pub type SymbolId = Uuid;

#[derive(Clone, Debug)]
pub struct Symbol {
    name: String,
    pub symbol_id: SymbolId,
    pub variant: SymbolVariant
}

impl Symbol {
    pub fn new(name: String, kind: SymbolVariant) -> Self {
        Symbol {
            name: name,
            symbol_id: SymbolId::new_v4(),
            variant: kind
        }
    }
}

#[derive(Clone, Debug)]
pub enum SymbolVariant {
    Variable(VariableSymbol),
    Primitive, // Primitives only need their name
    FunctionType(FunctionTypeSymbol),
    NativeFunction(NativeFunctionSymbol)
}

// Symbol variants:
#[derive(Clone, Debug)]
pub struct VariableSymbol {
    type_id: SymbolId
}

#[derive(Clone, Debug)]
pub struct FunctionTypeSymbol {
    return_id: Option<SymbolId>,
    argument_ids: Vec<SymbolId>
}

impl FunctionTypeSymbol {
    pub fn new(return_id: Option<SymbolId>, argument_ids: Vec<SymbolId>) -> Self {
        FunctionTypeSymbol {
            return_id,
            argument_ids
        }
    }

    pub fn construct_type_name(return_id: Option<SymbolId>, argument_ids: Vec<SymbolId>, semantic_analyzer: &SemanticAnalyzer) -> anyhow::Result<String> {
        // Format for a function type name:
        // <arg1,arg2,...,argn:return>
        // <arg1:return>
        // <:return>
        // <arg1:>
        // <:>

        let mut name = "<".to_string();

        let mut counter = 0;
        for arg_id in &argument_ids {
            let arg_name = semantic_analyzer.name_of_type(*arg_id)?.unwrap_or("<unknown>".to_string());
            name.push_str(&arg_name);

            counter += 1;
            if counter < argument_ids.len() {
                name.push(',');
            }
        }

        name.push('>');

        Ok(name)
    }
}

#[derive(Clone, Debug)]
pub struct NativeFunctionSymbol {
    type_id: SymbolId
}

impl NativeFunctionSymbol {
    pub fn new(type_id: SymbolId) -> Self {
        NativeFunctionSymbol {
            type_id
        }
    }
}

// Semantic analysis

/// This is what is returned when a grammatical Node is analyzed
#[derive(Debug)]
pub struct SemanticResult {
    pub node: SemanticNode,
    type_id: Option<SymbolId>,
    // More context to be added later...
    // Does this node have side effects, for example.
}

impl SemanticAnalyzer {
    pub fn analyze(&mut self, ast: Node) -> anyhow::Result<SemanticResult> {
        let ast = ast.clone();
        Ok(self.analyze_node(ast)?)
    }

    pub fn analyze_node(&mut self, ast: Node) -> anyhow::Result<SemanticResult> {
        match *ast {
            Ast::Block(nodes) => {
                // Create a scope and set it as the current scope
                let mut scope = SymbolTable::new("block".to_string());
                let id = scope.table_id;

                scope.parent = Some(self.current_scope_id);

                self.scopes.insert(id, scope);
                self.push_scope(id);
                
                let mut semantic_nodes = Vec::new();

                for node in nodes {
                    semantic_nodes.push(*self.analyze_node(node)?.node);
                }

                let node = SemanticAst::Block(semantic_nodes, id);

                // Set the current scope to the parent scope
                self.pop_scope()?;

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
                    type_id: Some(TRUTH_TYPE.symbol_id)
                })
            },
            Ast::Text(token) => {
                let node = SemanticAst::Text(token);

                Ok(SemanticResult {
                    node: Box::new(node),
                    type_id: Some(TEXT_TYPE.symbol_id)
                })
            },
            Ast::Variable(token) => {
                // lookup the variable and return it's type
                let name_node = Ast::Variable(token.clone());
                let symbol = self.current_scope()?.symbol_from_node(&name_node, self)?
                    .ok_or(anyhow::anyhow!("Variable {} not found", token.value))?;

                let type_id = match symbol.variant {
                    SymbolVariant::Variable(ref var) => var.type_id,
                    SymbolVariant::NativeFunction(ref func) => func.type_id,
                    _ => panic!("Symbol does not contain a value")
                };

                let node = SemanticAst::Variable(symbol.symbol_id);

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
                if self.current_scope()?
                    .symbol_from_node(&Ast::Variable(token.clone()), &self)?
                    .is_some()
                {
                    return Err(anyhow::anyhow!("Variable called {} already exists.", token.value));
                }

                // Create a new symbol and insert it into the symbol table
                let symbol = Symbol::new(token.value.clone(), SymbolVariant::Variable(VariableSymbol {
                    type_id: type_id
                }));

                self.current_scope_mut()?
                    .symbols.insert(symbol.symbol_id, symbol.clone());

                let node = SemanticAst::Declaration(symbol.symbol_id, symbol.symbol_id, result_node.node);

                Ok(SemanticResult {
                    node: Box::new(node),
                    type_id: None
                })
            },
            Ast::Assignment(target, node) => {
                let result_node = self.analyze_node(node)?;

                let target_symbol = self.symbol_from_node(&*target)?
                .ok_or(anyhow::anyhow!("Symbol not found"))?;

                // Get the type of the target
                // TODO: Expand the kinds of symbol that can be assigned to
                let type_id = match target_symbol.variant {
                    SymbolVariant::Variable(ref var) => var.type_id,
                    _ => panic!("Symbol is not a variable")
                };

                // Check if the type of the assignment is the same as the type of the variable
                if result_node.type_id.ok_or(anyhow::anyhow!("Assignment must be a valid expression (Must return value)"))? != type_id {
                    let expected_name = self.name_of_type(type_id)?.unwrap_or("<unknown>".to_string());
                    let got_name = self.name_of_type(
                        result_node.type_id
                            .ok_or(anyhow::anyhow!("Assignment must be a valid expression (Must return value)"))?
                        )?
                        .unwrap_or("<unknown>".to_string());

                    return Err(
                        anyhow::anyhow!(
                            "Type mismatch: Expected type {:?} but got type {:?}",
                            expected_name,
                            got_name
                        )
                    );
                }

                let node = SemanticAst::Assignment(target_symbol.symbol_id, result_node.node);

                Ok(SemanticResult {
                    node: Box::new(node),
                    type_id: None
                })
            },
            Ast::FunctionCall(callee, args) => {
                let callee_result = self.analyze_node(callee)?;
                let callee_variant = &self.current_scope()?
                    .symbol_from_id(callee_result.type_id.ok_or(anyhow::anyhow!(""))?, &self)
                    .ok_or(anyhow::anyhow!("Symbol not found"))?
                    .variant;

                let callee_type = match callee_variant {
                    SymbolVariant::FunctionType(ref func) => func.clone(),
                    _ => panic!("Symbol is not a function")
                };

                // Check that the number of arguments is correct
                if args.len() != callee_type.argument_ids.len() {
                    return Err(anyhow::anyhow!("Incorrect number of arguments"));
                }

                let mut arg_nodes = vec![];

                // Check that the types of the arguments are correct
                for (i, arg) in args.clone().iter().enumerate() {
                    let arg_result = self.analyze_node(arg.clone())?;
                    arg_nodes.push(arg_result.node);
                    let arg_type_id = arg_result.type_id
                        .ok_or(anyhow::anyhow!("Function argument must be a valid expression (Must return value)"))?;

                    if arg_type_id != callee_type.argument_ids[i] {
                        let expected_name = self.name_of_type(callee_type.argument_ids[i])?.unwrap_or("<unknown>".to_string());
                        let got_name = self.name_of_type(arg_type_id)?.unwrap_or("<unknown>".to_string());

                        return Err(
                            anyhow::anyhow!(
                                "Type mismatch: Expected type {:?} but got type {:?}",
                                expected_name,
                                got_name
                            )
                        );
                    }
                }

                let node = SemanticAst::FunctionCall(
                    callee_result.node,
                    arg_nodes
                );

                Ok(SemanticResult {
                    node: Box::new(node),
                    type_id: callee_type.return_id
                })
            },
            Ast::If(condition, body) => {
                let condition = self.analyze_node(condition)?;
                let body = self.analyze_node(body)?;

                // Check that the condition is a truth
                let condition_type = condition.type_id
                    .ok_or(anyhow::anyhow!("If condition must be a valid expression (Must return value)"))?;

                if condition_type != TRUTH_TYPE.symbol_id {
                    return Err(anyhow::anyhow!("If condition must be a truth"));
                }

                let node = SemanticAst::If(condition.node, body.node);

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

    pub fn push_scope(&mut self, scope_id: TableId) {
        self.current_scope_id = scope_id;
    }

    pub fn pop_scope(&mut self) -> anyhow::Result<()> {
        self.current_scope_id = self.current_scope()?.parent
            .expect("If you're popping a scope, it must have a parent");

        Ok(())
    }
}

// To recursively handle symbols in scopes
impl SemanticAnalyzer {
    // Find symbol from node
    fn symbol_from_node(&self, node: &Ast) -> anyhow::Result<Option<&Symbol>> {
        self.current_scope()?
            .symbol_from_node(node, &self)
    }
}

impl SymbolTable {
    fn symbol_from_node<'a>(&'a self, node: &Ast, semantic_analyzer: &'a SemanticAnalyzer) -> anyhow::Result<Option<&'a Symbol>> {
        let result = match node {
            Ast::Variable(token) => {
                self.lookup(token.value.clone())
            }
            _ => return Err(anyhow::anyhow!("Expected a variable"))
        };

        match result {
            Some(symbol) => Ok(Some(symbol)),
            None => {
                if let Some(parent) = self.parent_scope(semantic_analyzer) {
                    parent.symbol_from_node(node, semantic_analyzer)
                } else {
                    Ok(None)
                }
            }
        }
    }

    pub fn symbol_from_id<'a>(&'a self, id: SymbolId, semantic_analyzer: &'a SemanticAnalyzer) -> Option<&'a Symbol> {
        let symbol = self.lookup_id(id);

        if let Some(symbol) = symbol {
            Some(symbol)
        } else if let Some(parent) = self.parent_scope(semantic_analyzer) {
            parent.symbol_from_id(id, semantic_analyzer)
        } else {
            None
        }
    }
}

// For report purposes
impl SemanticAnalyzer {
    fn name_of_type(&self, id: SymbolId) -> anyhow::Result<Option<String>> {
        Ok(self.current_scope()?
        .name_of_type(id, &self))
    }
}

impl SymbolTable {
    fn parent_scope<'a>(&self, semantic_analyzer: &'a SemanticAnalyzer) -> Option<&'a SymbolTable> {
        if let Some(parent_id) = self.parent {
            return semantic_analyzer.scopes.get(&parent_id);
        }

        None
    }

    fn name_of_type(&self, id: SymbolId, semantic_analyzer: &SemanticAnalyzer) -> Option<String> {
        let symbol = self.lookup_id(id);

        if let Some(symbol) = symbol {
            Some(symbol.name.clone())
        } else if let Some(parent) = self.parent_scope(semantic_analyzer) {
            parent.name_of_type(id, semantic_analyzer)
        } else {
            None
        }
    }
}