// temporary allow
#![allow(dead_code)]

#[macro_use]
extern crate enum_display_derive;

mod tokens;
mod scanner;

use std::{error, path::Path};

fn main() -> Result<(), Box<dyn error::Error>> {
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

fn run_file<P>(file: &P) -> Result<(), Box<dyn error::Error>>
where
    P: AsRef<Path>,
{
    let src = std::fs::read_to_string(file)?;
    run(&src)?;

    Ok(())
}

fn run_prompt() -> Result<(), Box<dyn error::Error>> {
    use rustyline::{error, Editor};
    let mut reader = Editor::<()>::new();
    loop {
        let line = reader.readline(">> ");
        match line {
            Ok(line) => run(&line)?,
            Err(error::ReadlineError::Interrupted) => break,
            Err(error::ReadlineError::Eof) => break,
            Err(err) => return Err(err.into()),
        }
    }

    Ok(())
}
fn run(src: &str) -> Result<(), Box<dyn error::Error>> {
    let tokens = src.split_whitespace();
    for token in tokens {
        println!("{:?}", token);
    }
    Ok(())
}
