use std::fmt;
use std::fmt::Display;
#[allow(unused_imports)]
use std::io::{self, Write};
use std::str::FromStr;

#[derive(Debug)]
enum Command {
    Exit(String),
    Echo(String),
    Type(Type),
}

#[derive(Debug)]
enum Type {
    Builtin(Box<Command>),
    Unknown(String),
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
            "type" => match Command::from_str(&args) {
                Err(_) => Ok(Command::Type(Type::Unknown(args))),
                Ok(c) => Ok(Command::Type(Type::Builtin(Box::new(c)))),
            },
            _ => Err(CommandParsingError),
        }
    }
}

impl Display for Command {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Command::Exit(_) => write!(f, "exit"),
            Command::Echo(_) => write!(f, "echo"),
            Command::Type(_) => write!(f, "type"),
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
                Command::Type(Type::Builtin(c)) => println!("{} is a shell builtin", c),
                Command::Type(Type::Unknown(u)) => println!("{} not found", u),
            }
        } else {
            println!("{}: command not found", &input.trim())
        }
    }
}
