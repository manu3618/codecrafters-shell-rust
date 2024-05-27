#[allow(unused_imports)]
use std::io::{self, Write};
use std::str::FromStr;

#[derive(Debug)]
enum Command {
    Exit(String),
    Echo(String),
}

#[derive(Debug, PartialEq, Eq)]
struct CommandParsingError;

impl FromStr for Command {
    type Err = CommandParsingError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts = s.trim().splitn(2, ' ').collect::<Vec<_>>();
        let args = match parts.get(1) {
            Some(s) => s.to_string(),
            None => String::new(),
        };
        match *parts.first().unwrap_or(&"") {
            "exit" => Ok(Command::Exit(args)),
            "echo" => Ok(Command::Echo(args)),
            _ => Err(CommandParsingError),
        }
    }
}

fn main() {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    // eprintln!("Logs from your program will appear here!");

    // REPL loop
    loop {
        print!("$ ");
        io::stdout().flush().unwrap();

        // Wait for user input
        let stdin = io::stdin();
        let mut input = String::new();
        stdin.read_line(&mut input).unwrap();
        if let Ok(c) = Command::from_str(&input) {
            match c {
                Command::Exit(_a) => return,
                Command::Echo(e) => println!("{}", &e),
            }
        } else {
            println!("{}: command not found", &input.trim())
        }
    }
}
