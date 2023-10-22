use clap::Parser;
use std::io::Write;
use anyhow;

use odo::exec::interpreter::Interpreter;

#[derive(Parser)]
#[command(author, version, about)]
struct Cli {
    source_file: Option<String>
}

fn repl() -> anyhow::Result<()> {
    // It keeps context through the repl, so it's just one for all loops.
    let mut interpreter = Interpreter::new();
    loop {
        print!("> ");
        let mut input = String::new();

        std::io::stdout().flush()?;
        std::io::stdin().read_line(&mut input)?;

        let result = match interpreter.eval(input) {
            Ok(result) => result,
            Err(e) => {
                println!("{}", e);
                continue;
            }
        };

        match result.value.content {
            odo::exec::value::ValueVariant::Nothing => {},
            _ => println!("{:#?}", result.value.content)
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
