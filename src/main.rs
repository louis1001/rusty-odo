use clap::Parser as ClapParser;
use std::io::Write;
use anyhow;

use odo::lang::{lexer::Lexer, parser::Parser};

#[derive(ClapParser)]
#[command(author, version, about)]
struct Cli {
    source_file: Option<String>
}

fn repl() -> anyhow::Result<()> {
    loop {
        print!("> ");
        let mut input = String::new();

        std::io::stdout().flush()?;
        std::io::stdin().read_line(&mut input)?;

        let lexer = Lexer::new(input);
        let tokens: Vec<_> = lexer.collect();

        let mut parser = Parser::new(tokens);
        let ast = parser.parse();

        match ast {
            Ok(ast) => println!("{:#?}", ast),
            Err(e) => println!("{}", e)
        }
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
