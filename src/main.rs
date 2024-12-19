use std::env;
use std::ffi::OsStr;
use std::fmt;
use std::fmt::Display;
use std::io::{self, Write};
use std::path::PathBuf;
use std::process;
use std::str::FromStr;

#[derive(Debug)]
struct PathEnv(Vec<PathBuf>);

#[derive(Debug)]
struct PathEnvParsingError;

impl FromStr for PathEnv {
    type Err = PathEnvParsingError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut paths = Vec::new();
        let parts = s.trim().split(':');
        for part in parts {
            paths.push(part.into());
        }
        Ok(Self(paths))
    }
}

impl PathEnv {
    /// Find first occurence of bin in path contained in Self
    fn find(&self, bin: &str) -> Option<PathBuf> {
        if bin.starts_with('/') {
            return Some(bin.into());
        }
        let bin = OsStr::new(bin);
        for path in &self.0 {
            let msg = format!("{} should exist", path.display());
            for entry in path.read_dir().expect(&msg) {
                let e = entry.unwrap().path().clone();
                if e.file_name().unwrap() == bin {
                    return Some(e);
                }
            }
        }
        None
    }
}

#[derive(Debug)]
enum Command {
    Exit(String),
    Echo(String),
    Type(Type),
    Pwd,
    /// Local command with the output
    Local(String),
}

#[derive(Debug)]
enum Type {
    Builtin(Box<Command>),
    Local(PathBuf),
    Unknown(String),
}

#[derive(Debug, PartialEq, Eq)]
struct CommandParsingError;

impl FromStr for Command {
    type Err = CommandParsingError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let pathenv: PathEnv =
            PathEnv::from_str(&env::var("PATH").unwrap()).expect("PATH should be set");
        let parts = s.trim().splitn(2, ' ').collect::<Vec<_>>();
        let args = match parts.get(1) {
            Some(s) => s.to_string(),
            None => String::new(),
        };
        match *parts.first().unwrap_or(&"") {
            "exit" => Ok(Command::Exit(args)),
            "echo" => Ok(Command::Echo(args)),
            "type" => match Command::from_str(&args) {
                Ok(Command::Local(_)) => {
                    Ok(Command::Type(Type::Local(pathenv.find(parts[1]).unwrap())))
                }
                Ok(c) => Ok(Command::Type(Type::Builtin(Box::new(c)))),
                Err(_) => {
                    if let Some(p) = pathenv.find(args.split(' ').next().unwrap_or("")) {
                        Ok(Command::Type(Type::Local(p)))
                    } else {
                        Ok(Command::Type(Type::Unknown(args)))
                    }
                }
            },
            "pwd" => Ok(Command::Pwd),
            cmd => {
                if let Some(p) = pathenv.find(cmd) {
                    let mut c = process::Command::new(p);
                    for arg in args.split(' ') {
                        c.arg(arg);
                    }
                    let out = std::str::from_utf8(&c.output().unwrap().stdout)
                        .unwrap()
                        .into();
                    Ok(Command::Local(out))
                } else {
                    Err(CommandParsingError)
                }
            }
        }
    }
}

impl Display for Command {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Command::Exit(_) => write!(f, "exit"),
            Command::Echo(_) => write!(f, "echo"),
            Command::Type(_) => write!(f, "type"),
            Command::Pwd => write!(f, "pwd"),
            Command::Local(_) => unimplemented!(),
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
                Command::Type(Type::Local(p)) => {
                    println!(
                        "{} is {}",
                        p.file_name().unwrap().to_str().unwrap(),
                        p.to_str().unwrap()
                    )
                }
                Command::Type(Type::Unknown(u)) => println!("{} not found", u),
                Command::Pwd => {
                    let path = env::current_dir().unwrap();
                    println!("{}", path.display())
                }
                Command::Local(o) => print!("{}", o),
            }
        } else {
            println!("{}: command not found", &input.trim())
        }
    }
}
