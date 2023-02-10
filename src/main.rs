use std::fs::File;
use std::io::{
    self, BufRead, BufReader, Write
};
use std::process::exit;
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
fn main() -> io::Result<()>{
    let args = Args::parse();
    let mut lox = Lox::new();

    match args.run {
        None => {
            lox.run_prompt()
        },
        Some(program_name) => {
            lox.run_file(&program_name)
        }
    }
}

struct Lox {
    had_error: bool,
}

impl Lox {
    fn new() -> Self {
        Lox {
            had_error: false
        }
    }

    fn run_file(&self, path: &str) -> io::Result<()> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut source = String::from("");

        for line in reader.lines() {
            source.push_str(&line.unwrap());
            source.push_str("\n")
        }

        Self::run(&source);

        if self.had_error {
            exit(65);
        }

        Ok(())
    }

    fn run_prompt(&mut self) -> io::Result<()> {
        let stdin = io::stdin();
        let mut stdout = io::stdout();

        print!("> ");
        stdout.flush().unwrap();

        let mut source = String::from("");
        for line in stdin.lock().lines() {
            source.push_str(&line?);
            Self::run(&source);
            self.had_error = false;

            source.clear();
            print!("> ");
            stdout.flush().unwrap();
        }

        Ok(())
    }

    fn run(source: &str) {
        let scanner = Scanner::new(source);
        let tokens = scanner.scan_tokens();
        tokens.into_iter().map(|token| println!("{token}"))
    }

    #[allow(dead_code)]
    fn error(&mut self, line: u32, message: &str) {
        self.report(line, "", message);
    }

    #[allow(dead_code)]
    fn report(&mut self, line: u32, location: &str, message: &str) {
        println!("[line {} ] Error {}: {}", line, location, message);
        self.had_error = true;
    }
}
