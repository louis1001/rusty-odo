use clap::Parser as ClapParser;
use std::io::Write;
use anyhow;

use odo::lang::{lexer::Lexer, parser::Parser, semantic_analyzer::SemanticAnalyzer};

#[derive(ClapParser)]
#[command(author, version, about)]
struct Cli {
    source_file: Option<String>
}

fn repl() -> anyhow::Result<()> {
    // It keeps context through the repl, so it's just one for all loops.
    let mut semantic_analyzer = SemanticAnalyzer::new();
    loop {
        print!("> ");
        let mut input = String::new();

        std::io::stdout().flush()?;
        std::io::stdin().read_line(&mut input)?;

        let lexer = Lexer::new(input);
        let tokens: Vec<_> = lexer.collect();

        let mut parser = Parser::new(tokens);
        let ast = match parser.parse() {
            Ok(ast) => ast,
            Err(e) => {
                println!("{}", e);
                continue;
            }
        };

        println!("{:#?}", ast);

        let semantic_ast = match semantic_analyzer.analyze(ast) {
            Ok(ast) => ast,
            Err(e) => {
                println!("{}", e);
                continue;
            }
        };

        println!("{:#?}", semantic_ast);
    }
}

fn main() -> anyhow::Result<()> {
    let args = Cli::parse();

    if let Some(input_path) = args.source_file {
        // Execute the file
        todo!("Implement file execution with scoping and modularity");
    } else {
        // Execute the repl
        repl()?;
    }


    Ok(())
}
