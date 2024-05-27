#[allow(unused_imports)]
use std::io::{self, Write};
use std::str::FromStr;

#[derive(Debug)]
enum Command {}

#[derive(Debug, PartialEq, Eq)]
struct CommandParsingError;

impl FromStr for Command {
    type Err = CommandParsingError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            _ => Err(CommandParsingError),
        }
    }
}

fn main() {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    // eprintln!("Logs from your program will appear here!");

    // Uncomment this block to pass the first stage
    print!("$ ");
    io::stdout().flush().unwrap();

    // Wait for user input
    let stdin = io::stdin();
    let mut input = String::new();
    stdin.read_line(&mut input).unwrap();
    if let Ok(_c) = Command::from_str(&input) {
    } else {
        println!("{}: command not found", &input)
    }
}
