use std::sync::Arc;

use crate::base::semantic_analyzer::{FunctionTypeSymbol, Symbol, SymbolVariant, NativeFunctionSymbol};
use crate::exec::interpreter::Interpreter;
use crate::exec::value::{Value, ValueVariant, FunctionValue};

pub type NativeFn<'a> = dyn Fn(Vec<Value<'a>>) -> Option<Value<'a>> + Sync + 'a;

pub trait NativeFunctionBindable<'obj> {
    // Has to be able to be a closure, and the closure has to be able to be called.
    fn bind_void_function<'a, F>(&mut self, name: &str, f: F) -> anyhow::Result<()> where F: Fn(Vec<Value>) -> () + Sync + 'obj;
}

impl<'inter> NativeFunctionBindable<'inter> for Interpreter<'inter> {
    fn bind_void_function<F>(&mut self, name: &str, f: F) -> anyhow::Result<()> where F: Fn(Vec<Value>) -> () + Sync + 'inter, {
        // Construct the type of the function.
        let function_type_name = FunctionTypeSymbol::construct_type_name(
            None,
            vec![],
            &self.semantic_analyzer
        )?;

        let function_type = Symbol::new(
            function_type_name,
            SymbolVariant::FunctionType(FunctionTypeSymbol::new(None, vec![]))
        );

        let function_symbol = Symbol::new(
            name.to_string(),
            SymbolVariant::NativeFunction(NativeFunctionSymbol::new(function_type.symbol_id))
        );

        // Insert the type into the global scope
        {
            let global_scope = self.semantic_analyzer.global_scope_mut().expect("There's always a global scope");
            global_scope.insert(function_type.clone());
        }

        // Insert the symbol into the current scope.
        {
            let current_scope = self.semantic_analyzer.current_scope_mut().expect("There's always a scope");
            current_scope.insert(function_symbol.clone());
        }

        let native_fn = move |args: Vec<Value>| {
            f(args);
            None
        };

        let value = Value::new(ValueVariant::Function(FunctionValue::Native(Arc::new(native_fn))));
        self.value_table.insert(value.clone());

        self.bind_symbol_to_value(function_symbol.symbol_id, value.uuid);

        Ok(())
    }
}