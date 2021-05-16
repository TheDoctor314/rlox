// temporary allow
//#![allow(dead_code)]

mod class;
mod env;
mod error;
mod expr;
mod functions;
mod interpreter;
mod object;
mod parser;
mod resolver;
mod runner;
mod scanner;
mod stmt;
mod tokens;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut run = runner::Runner {};

    let args = std::env::args().collect::<Vec<_>>();
    let res = {
        match args.len() {
            1 => run.prompt(),
            2 => run.file(&args[1]),
            _ => {
                eprintln!("Usage: rlox [script]");
                std::process::exit(64);
            }
        }
    };

    if let Err(e) = res {
        eprintln!("{}", e);
        std::process::exit(1);
    }
    Ok(())
}
