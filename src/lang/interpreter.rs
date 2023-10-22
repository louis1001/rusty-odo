use lazy_static::lazy_static;
use uuid::Uuid;
use std::collections::HashMap;

use super::{semantic_analyzer::{SemanticAnalyzer, SemanticAst}, lexer::Lexer, parser::Parser};

pub struct Interpreter {
    pub value_table: ValueTable,
    semantic_analyzer: SemanticAnalyzer,
    symbol_to_value: HashMap<Uuid, Uuid>
}

impl Interpreter {
    pub fn new() -> Interpreter {
        Interpreter {
            value_table: ValueTable::new(),
            semantic_analyzer: SemanticAnalyzer::new(),
            symbol_to_value: HashMap::new()
        }
    }

    fn interpret(&mut self, semantic_ast: SemanticAst) -> anyhow::Result<ExecutionResult> {
        match semantic_ast {
            SemanticAst::Block(nodes, scope_id) => {
                self.semantic_analyzer.push_scope(scope_id);
                for node in nodes {
                    self.interpret(node)?;
                }
                self.semantic_analyzer.pop_scope()?;

                Ok(ExecutionResult { value: NO_VALUE.clone() })
            },
            SemanticAst::Number(token) => {
                let value = Value {
                    content: ValueVariant::Primitive(PrimitiveValue::Int(token.value.parse::<i64>()?)),
                    uuid: Uuid::new_v4()
                };

                Ok(ExecutionResult { value: value })
            },
            SemanticAst::Truth(token) => {
                let value = Value {
                    content: ValueVariant::Primitive(PrimitiveValue::Bool(token.value.parse::<bool>()?)),
                    uuid: Uuid::new_v4()
                };

                Ok(ExecutionResult { value: value })
            },
            SemanticAst::Variable(token) => {
                let symbol = self.semantic_analyzer.current_scope().expect("There's always a scope").lookup(token.value).ok_or(anyhow::anyhow!("Symbol not found"))?;

                let value = self.value_table.get(self.symbol_to_value[&symbol.symbol_id]).ok_or(anyhow::anyhow!("Value not found"))?;

                Ok(ExecutionResult { value: value.clone() })
            },
            SemanticAst::Declaration(token, _, node) => {
                let result = self.interpret(*node)?;

                let symbol = self.semantic_analyzer.current_scope().expect("There's always a scope").lookup(token.value).ok_or(anyhow::anyhow!("Symbol not found"))?;

                self.symbol_to_value.insert(symbol.symbol_id, result.value.uuid);

                self.value_table.insert(result.value);

                Ok(ExecutionResult { value: NO_VALUE.clone() })
            },
            SemanticAst::Assignment(target_id, node) => {
                let result = self.interpret(*node)?;

                let symbol = self.semantic_analyzer.current_scope()
                    .expect("There's always a scope").lookup_id(target_id)
                    .ok_or(anyhow::anyhow!("Symbol not found"))?;

                self.symbol_to_value.insert(symbol.symbol_id, result.value.uuid);

                self.value_table.insert(result.value); // Updates if it already existed

                Ok(ExecutionResult { value: NO_VALUE.clone() })
            },
            SemanticAst::DebugPrint(node) => {
                let result = self.interpret(*node)?;

                println!("DebugPrint -> {:?}", result.value);

                Ok(ExecutionResult { value: NO_VALUE.clone() })
            }
        }
    }

    /* This is a translation of this old C++ code:
    value_t Interpreter::eval(std::string code) {

        call_stack.push_back({"global", 1, 1});
        parser.set_text(std::move(code));

        auto statements = parser.program_content();

    //    try{
        currentScope = &replScope;

        auto result = null;
        for (const auto& node : statements) {
            analyzer->from_repl(node);
            result = visit(node);
        }

        currentScope = &globalTable;
    //    }
        call_stack.pop_back();

        return result;
    }
     */
    pub fn eval(&mut self, code: String) -> anyhow::Result<ExecutionResult> {
        let lexer = Lexer::new(code);
        let tokens: Vec<_> = lexer.collect();

        let mut parser = Parser::new(tokens);
        let statements = parser.statement_list()?;

        let repl_id = self.semantic_analyzer.repl_scope_id;
        self.semantic_analyzer.push_scope(repl_id);

        let mut result = NO_VALUE.clone();
        for node in statements {
            let semantic_result = self.semantic_analyzer.analyze(node)?;
            result = self.interpret(*semantic_result.node)?.value;
        }

        self.semantic_analyzer.pop_scope()?;

        Ok(ExecutionResult { value: result })
    }
}

pub struct ExecutionResult {
    pub value: Value
}

#[derive(Debug)]
pub struct ValueTable {
    values: HashMap<Uuid, Value>,
}

#[derive(Clone, Debug)]
pub struct Value {
    pub content: ValueVariant,
    uuid: Uuid,
}

lazy_static! {
    pub static ref NO_VALUE: Value = Value { content: ValueVariant::Nothing, uuid: Uuid::new_v4() };
}

#[derive(Clone, Debug)]
pub enum ValueVariant {
    Nothing,
    Primitive(PrimitiveValue),
}

#[derive(Clone, Debug)]
pub enum PrimitiveValue {
    Int(i64),
    Dec(f64),
    String(String),
    Bool(bool),
}

impl ValueTable {
    pub fn new() -> ValueTable {
        ValueTable {
            values: HashMap::new(),
        }
    }

    pub fn insert(&mut self, value: Value) {
        self.values.insert(value.uuid, value);
    }

    pub fn get(&self, uuid: Uuid) -> Option<&Value> {
        self.values.get(&uuid)
    }
}
