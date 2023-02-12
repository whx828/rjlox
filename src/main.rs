mod expr;
mod interpreter;
mod object;
mod parser;
mod scanner;
mod token;

use std::fs::File;
use std::io::{self, BufRead, BufReader, Write};
use std::process::exit;

use crate::interpreter::{Interpreter, RuntimeError};
use crate::token::{Token, TokenType};
use clap::Parser;

/// rjlox interpreter
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Name of the lox file to interpreter
    #[arg(short, long)]
    run: Option<String>,
}

// note1: 最后在整个程序的层面进行 Rust 错误处理
fn main() -> io::Result<()> {
    let args = Args::parse();
    let mut lox = Lox::new();

    match args.run {
        None => run_prompt(&mut lox),
        Some(program_name) => run_file(lox, &program_name),
    }
}

#[derive(Clone)]
pub struct Lox {
    had_error: bool,
    had_runtime_error: bool,
}

impl Lox {
    fn new() -> Self {
        Lox {
            had_error: false,
            had_runtime_error: false,
        }
    }

    pub fn error(&mut self, line: u32, message: &str) {
        self.report(line, "", message);
    }

    pub fn error_parse(&mut self, token: Token, message: &str) {
        if token.token_type == TokenType::EOF {
            self.report(token.line, "at end", message);
        } else {
            let x = format!("at '{}'", token.lexeme);
            self.report(token.line, &x, message);
        }
    }

    pub fn runtime_error(&mut self, err: RuntimeError) {
        println!("{} [line {} ]", err.message, err.token.line);
        self.had_runtime_error = true;
    }

    fn report(&mut self, line: u32, location: &str, message: &str) {
        println!("[line {line} ] Error {location}: {message}");
        self.had_error = true;
    }
}

fn run_file(mut lox: Lox, path: &str) -> io::Result<()> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut source = String::from("");

    for line in reader.lines() {
        source.push_str(&line.unwrap());
        source.push('\n');
    }

    run(&mut lox, &source); // 对于 file，run 只运行一次

    if lox.had_error {
        exit(65);
    }

    if lox.had_runtime_error {
        exit(70);
    }

    Ok(())
}

fn run_prompt(lox: &mut Lox) -> io::Result<()> {
    let stdin = io::stdin();
    let mut stdout = io::stdout();

    print!("> ");
    stdout.flush().unwrap();

    let mut source = String::from("");
    for line in stdin.lock().lines() {
        source.push_str(&line?);
        run(lox, &source); // 对于 prompt，run 运行若干次，有几次有效输入就运行几次
        lox.had_error = false;

        source.clear();
        print!("> ");
        stdout.flush().unwrap();
    }

    Ok(())
}

fn run(lox: &mut Lox, source: &str) {
    let mut scanner = scanner::Scanner::new(lox, source.to_string());
    let tokens = scanner.scan_tokens();
    let mut parser = parser::Parser::new(lox, tokens);
    let expression = parser.parse();

    // Stop if there was a syntax error.
    if lox.had_error {
        return;
    }

    if let Some(expr) = expression {
        let mut interpreter = Interpreter::new(lox);
        interpreter.interpret(expr);
    }
}
