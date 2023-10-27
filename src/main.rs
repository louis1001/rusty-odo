use clap::Parser;
use anyhow;

#[derive(Parser)]
#[command(author, version)]
struct Cli {
    source_file: Option<String>,

    // About
    #[clap(short, long)]
    about: bool,
}

fn main() -> anyhow::Result<()> {
    let args = Cli::parse();

    if args.about {
        repl::print_logo();
        return Ok(());
    }

    if let Some(input_path) = args.source_file {
        // Execute the file
        todo!("Implement file execution with scoping and modularity");
    } else {
        // Execute the repl
        repl::repl()?;
    }


    Ok(())
}

mod repl {
    use odo::exec::interpreter::Interpreter;
    use std::io::Write;

    pub fn print_logo() {
        let logo = format!(
            r#"
              (((((((((((((((
           (((((((((((((((((((((
         (((((((           ******
         ((((((             ******
         ((((((             **   *
         ((((((             ******
         ((((((((         *******
           (((((((((((((((((((((
              (((((((((((((((
    
                odo(-lang)
                   {}
          Luis Gonzalez (louis1001)
                 2019-2023
    "#, 
            env!("CARGO_PKG_VERSION"));

        println!("{}", logo);
    }

    pub fn repl() -> anyhow::Result<()> {
        // It keeps context through the repl, so it's just one for all loops.
        let mut interpreter = Interpreter::new();

        loop {
            print!("> ");
            let mut input = String::new();
    
            std::io::stdout().flush()?;
            std::io::stdin().read_line(&mut input)?;

            if input == "exit" {
                break;
            }
    
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

        Ok(())
    }
}