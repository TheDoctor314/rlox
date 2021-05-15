use std::path::Path;

use crate::{
    error::Result, interpreter::Interpreter, parser::StmtIterator, resolver::Resolver,
    scanner::TokenIterator,
};

pub struct Runner;

impl Runner {
    pub(crate) fn file<P>(&mut self, f: &P) -> Result<()>
    where
        P: AsRef<Path>,
    {
        let src = std::fs::read_to_string(f)?;

        let mut i = Interpreter::new(false);

        self.run(&mut i, &src)
    }

    pub(crate) fn prompt(&mut self) -> Result<()> {
        use rustyline::error::ReadlineError;
        use rustyline::Editor;

        let mut reader = Editor::<()>::new();
        let mut i = Interpreter::new(true);

        loop {
            let line = reader.readline(">> ");
            match line {
                Err(ReadlineError::Interrupted) => break,
                Err(ReadlineError::Eof) => break,
                Err(e) => {
                    eprintln!("{}", e);
                }
                Ok(line) => {
                    if let Err(e) = self.run(&mut i, &line) {
                        eprintln!("{}", e);
                    }
                }
            }
        }

        Ok(())
    }

    pub(crate) fn run(&mut self, i: &mut Interpreter, src: &str) -> Result<()> {
        for res in src.chars().tokens().statements() {
            match res {
                Err(e) => eprintln!("{}", e),
                Ok(stmt) => {
                    let i = Resolver::resolve(i, &stmt)?;
                    stmt.accept(i)?;
                }
            }
        }

        Ok(())
    }
}
