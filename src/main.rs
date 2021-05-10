// temporary allow
#![allow(dead_code)]

mod env;
mod error;
mod expr;
mod functions;
mod interpreter;
mod object;
mod parser;
mod scanner;
mod stmt;
mod tokens;

use std::path::Path;

use interpreter::Interpreter;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = std::env::args().collect::<Vec<_>>();
    match args.len() {
        1 => run_prompt()?,
        2 => run_file(&args[1])?,
        _ => {
            eprintln!("Usage: rlox [script]");
            std::process::exit(64);
        }
    }
    Ok(())
}

fn run_file<P>(file: &P) -> Result<(), Box<dyn std::error::Error>>
where
    P: AsRef<Path>,
{
    let src = std::fs::read_to_string(file)?;
    let mut i = Interpreter::new(false);
    run(&src, &mut i)?;

    Ok(())
}

fn run_prompt() -> Result<(), Box<dyn std::error::Error>> {
    use rustyline::{error, Editor};
    let mut reader = Editor::<()>::new();
    let mut i = Interpreter::new(true);

    loop {
        let line = reader.readline(">> ");
        match line {
            Ok(line) => run(&line, &mut i)?,
            Err(error::ReadlineError::Interrupted) => break,
            Err(error::ReadlineError::Eof) => break,
            Err(err) => return Err(err.into()),
        }
    }

    Ok(())
}
fn run(src: &str, i: &mut Interpreter) -> Result<(), Box<dyn std::error::Error>> {
    let scan = scanner::Scanner::new(src.chars());
    //let tokens_vec: Vec<tokens::Token> = scan.map(|x| x.unwrap()).collect();
    //println!(
    //    "[{}]",
    //    tokens_vec
    //        .iter()
    //        .fold(String::new(), |acc, token| acc + &token.to_string() + ", ")
    //);

    let parser = parser::Parser::new(scan);

    for stmt in parser {
        match stmt {
            Err(e) => eprintln!("{}", e),
            Ok(stmt) => i.interpret(&stmt).unwrap(),
        }
    }

    Ok(())
}
