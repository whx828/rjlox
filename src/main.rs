mod callable;
mod environment;
mod error;
mod expr;
mod interpreter;
mod object;
mod parser;
mod scanner;
mod stmt;
mod token;

use std::fs::File;
use std::io::{self, BufRead, BufReader, Write};
use std::process::exit;

use crate::environment::Environment;
use crate::error::Error;
use crate::error::Result;
use crate::interpreter::Interpreter;
use clap::Parser;

/// rjlox interpreter
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Name of the lox file to interpreter
    #[arg(short, long)]
    run: Option<String>,
}

fn main() -> io::Result<()> {
    let args = Args::parse();

    match args.run {
        None => run_prompt(),
        Some(program_name) => run_file(&program_name),
    }
}

fn run_file(path: &str) -> io::Result<()> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut source = String::from("");
    let env = Environment::new(None);
    let mut interpreter = Interpreter::new(env);

    for line in reader.lines() {
        source.push_str(&line.unwrap());
        source.push('\n');
    }

    if run(&source, &mut interpreter).is_err() {
        exit(70);
    };

    Ok(())
}

fn run_prompt() -> io::Result<()> {
    let stdin = io::stdin();
    let mut stdout = io::stdout();
    let env = Environment::new(None);
    let mut interpreter = Interpreter::new(env);

    print!("> ");
    stdout.flush().unwrap();

    let mut source = String::from("");
    for line in stdin.lock().lines() {
        source.push_str(&line?);

        if run(&source, &mut interpreter).is_err() {}

        source.clear();
        print!("> ");
        stdout.flush().unwrap();
    }

    Ok(())
}

fn run(source: &str, interpreter: &mut Interpreter) -> Result<()> {
    let mut scanner = scanner::Scanner::new(source.to_string());
    let tokens = scanner.scan_tokens();
    let mut parser = parser::Parser::new(tokens);
    let statements = match parser.parse() {
        Ok(result) => result,
        _ => return Err(Error::ParseError(String::from("parse error"))),
    };

    interpreter.interpret(statements)
}
