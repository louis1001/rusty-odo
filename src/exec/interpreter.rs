use uuid::Uuid;
use std::collections::HashMap;
use super::value::{ValueTable, Value, NO_VALUE, PrimitiveValue, ValueVariant};

use crate::base::{semantic_analyzer::{SemanticAnalyzer, SemanticAst}, lexer::Lexer, parser::Parser};

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
                let value = Value::new(ValueVariant::Primitive(PrimitiveValue::Int(token.value.parse::<i64>()?)));

                Ok(ExecutionResult { value: value })
            },
            SemanticAst::Truth(token) => {
                let value = Value::new(ValueVariant::Primitive(PrimitiveValue::Bool(token.value.parse::<bool>()?)));

                Ok(ExecutionResult { value: value })
            },
            SemanticAst::Text(token) => {
                let value = Value::new(ValueVariant::Primitive(PrimitiveValue::Text(token.value)));

                Ok(ExecutionResult { value: value })
            },
            SemanticAst::Variable(id) => {
                let symbol = self.semantic_analyzer.current_scope().expect("There's always a scope")
                    .symbol_from_id(id, &self.semantic_analyzer)
                    .ok_or(anyhow::anyhow!("Symbol not found"))?;

                let value = self.value_table.get(self.symbol_to_value[&symbol.symbol_id]).ok_or(anyhow::anyhow!("Value not found"))?;

                Ok(ExecutionResult { value: value.clone() })
            },
            SemanticAst::Declaration(target, _, node) => {
                let result = self.interpret(*node)?;

                let symbol = self.semantic_analyzer.current_scope()
                    .expect("There's always a scope")
                    .lookup_id(target).ok_or(anyhow::anyhow!("Symbol not found"))?;

                self.symbol_to_value.insert(symbol.symbol_id, result.value.uuid);

                self.value_table.insert(result.value);

                Ok(ExecutionResult { value: NO_VALUE.clone() })
            },
            SemanticAst::Assignment(target_id, node) => {
                let result = self.interpret(*node)?;

                let symbol = self.semantic_analyzer.current_scope()
                    .expect("There's always a scope").symbol_from_id(target_id, &self.semantic_analyzer)
                    .ok_or(anyhow::anyhow!("Symbol not found"))?;

                self.symbol_to_value.insert(symbol.symbol_id, result.value.uuid);

                self.value_table.insert(result.value); // Updates if it already existed

                Ok(ExecutionResult { value: NO_VALUE.clone() })
            },
            SemanticAst::If(condition, body) => {
                let condition_result = self.interpret(*condition)?;

                if let ValueVariant::Primitive(PrimitiveValue::Bool(true)) = condition_result.value.content {
                    self.interpret(*body)?;
                }

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

