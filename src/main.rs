use shell_starter_rust::parse_args;
use std::env;
use std::ffi::OsStr;
use std::fmt;
use std::fmt::Display;
use std::io::{self, Write};
use std::path::Path;
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
    Cd(String),
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
        let s = s.trim();
        let parts = parse_args(s);
        let empty = String::from("");
        let cmd = parts.first().unwrap_or(&empty).as_str();
        let mut args = String::new();
        if !parts.get(1).unwrap_or(&empty).is_empty() {
            args = s[cmd.len()..]
                .split_once(char::is_whitespace)
                .unwrap()
                .1
                .to_string();
        }
        match cmd {
            "exit" => Ok(Command::Exit(args)),
            "echo" => Ok(Command::Echo(args)),
            "type" => match Command::from_str(&args) {
                Ok(Command::Local(_)) => {
                    Ok(Command::Type(Type::Local(pathenv.find(&args).unwrap())))
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
            "cd" => Ok(Command::Cd(args)),
            cmd => {
                if let Some(p) = pathenv.find(cmd) {
                    let mut c = process::Command::new(p);
                    for arg in parse_args(&args) {
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
            Command::Cd(_) => write!(f, "cd"),
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
                Command::Echo(e) => println!("{}", &parse_args(&e).join(" ")),
                Command::Type(Type::Builtin(c)) => println!("{} is a shell builtin", c),
                Command::Type(Type::Local(p)) => {
                    println!(
                        "{} is {}",
                        p.file_name().unwrap().to_str().unwrap(),
                        p.to_str().unwrap()
                    )
                }
                Command::Type(Type::Unknown(u)) => println!("{}: not found", u),
                Command::Pwd => {
                    let path = env::current_dir().unwrap();
                    println!("{}", path.display())
                }
                Command::Cd(path) => {
                    let home = env::var("HOME").expect("HOME should be set");
                    let path = match path.as_str() {
                        "~" | "" => Path::new(&home),
                        _ => Path::new(&path),
                    };
                    let _ = env::set_current_dir(path).or_else(|_| {
                        println!("cd: {}: No such file or directory", path.display());
                        Ok::<(), String>(())
                    });
                }
                Command::Local(o) => print!("{}", o),
            }
        } else {
            println!("{}: command not found", &input.trim())
        }
    }
}
