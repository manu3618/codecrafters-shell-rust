use anyhow::Result;
use shell_starter_rust::parse_args;
use std::env;
use std::ffi::OsStr;
use std::fmt;
use std::fmt::Display;
use std::fs::File;
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
    /// Local command with the output and error
    Local(CmdOutput, CmdError),
}

#[derive(Debug)]
struct CmdOutput(String);

impl Display for CmdOutput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug)]
struct CmdError(String);

impl Display for CmdError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
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
        match extract_command(s) {
            Ok((c, _, _)) => Ok(c),
            Err(e) => Err(e),
        }
    }
}

/// Extract command, stdout and stderr
fn extract_command(s: &str) -> Result<(Command, Option<File>, Option<File>), CommandParsingError> {
    let pathenv: PathEnv =
        PathEnv::from_str(&env::var("PATH").unwrap()).expect("PATH should be set");
    let s = s.trim();
    let parts = &parse_args(s);
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

    let mut to_remove = Vec::new();
    let sout = match parts.iter().position(|elt| elt == "1>" || elt == ">") {
        Some(idx) => {
            let filename = parts[idx + 1].clone();
            to_remove.extend([idx, idx + 1]);
            Some(File::create(filename).unwrap())
        }
        None => None,
    };
    let serr = match parts.iter().position(|elt| elt == "2>") {
        Some(idx) => {
            let filename = parts[idx + 1].clone();
            to_remove.extend([idx, idx + 1]);
            Some(File::create(filename).unwrap())
        }
        None => None,
    };
    to_remove.sort_by(|a, b| b.cmp(a));
    let mut parts = parts.clone();
    for idx in to_remove {
        let _ = parts.remove(idx);
    }
    let _ = parts.remove(0); // cmd

    let command = match cmd {
        "exit" => Command::Exit(args),
        "echo" => Command::Echo(parts.join(" ")),
        "type" => match extract_command(&args) {
            Ok((Command::Local(_, _), _, _)) => {
                Command::Type(Type::Local(pathenv.find(&args).unwrap()))
            }
            Ok((c, _, _)) => Command::Type(Type::Builtin(Box::new(c))),
            Err(_) => {
                if let Some(p) = pathenv.find(args.split(' ').next().unwrap_or("")) {
                    Command::Type(Type::Local(p))
                } else {
                    Command::Type(Type::Unknown(args))
                }
            }
        },
        "pwd" => Command::Pwd,
        "cd" => Command::Cd(args),
        cmd => {
            if let Some(p) = pathenv.find(cmd) {
                let mut c = process::Command::new(p);
                for arg in parts {
                    c.arg(arg);
                }
                if let Some(f) = sout {
                    c.stdout(f);
                }
                if let Some(f) = serr {
                    c.stderr(f);
                }
                let out = &c.output().unwrap();
                let out_out = String::from_utf8(out.stdout.clone()).unwrap();
                let out_err = String::from_utf8(out.stderr.clone()).unwrap();
                return Ok((
                    Command::Local(CmdOutput(out_out), CmdError(out_err)),
                    None,
                    None,
                ));
            } else {
                return Err(CommandParsingError);
            }
        }
    };
    Ok((command, sout, serr))
}

impl Display for Command {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Command::Exit(_) => write!(f, "exit"),
            Command::Echo(_) => write!(f, "echo"),
            Command::Type(_) => write!(f, "type"),
            Command::Pwd => write!(f, "pwd"),
            Command::Cd(_) => write!(f, "cd"),
            Command::Local(_, _) => unimplemented!(),
        }
    }
}

fn main() -> Result<()> {
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
        if let Ok((c, sout, serr)) = extract_command(&input) {
            let (out, err) = match c {
                Command::Exit(_a) => return Ok(()),
                Command::Echo(e) => (Some(format!("{}\n", &parse_args(&e).join(" "))), None),
                Command::Type(Type::Builtin(c)) => {
                    (Some(format!("{} is a shell builtin", c)), None)
                }
                Command::Type(Type::Local(p)) => (
                    Some(format!(
                        "{} is {}",
                        p.file_name().unwrap().to_str().unwrap(),
                        p.to_str().unwrap()
                    )),
                    None,
                ),
                Command::Type(Type::Unknown(u)) => (None, Some(format!("{}: not found", u))),
                Command::Pwd => {
                    let path = env::current_dir().unwrap();
                    (Some(format!("{}", path.display())), None)
                }
                Command::Cd(path) => {
                    let home = env::var("HOME").expect("HOME should be set");
                    let path = match path.as_str() {
                        "~" | "" => Path::new(&home),
                        _ => Path::new(&path),
                    };
                    match env::set_current_dir(path) {
                        Ok(_) => (None, None),
                        Err(_) => (
                            None,
                            Some(format!(
                                "cd: {}: No such file or directory\n",
                                path.display()
                            )),
                        ),
                    }
                }
                Command::Local(o, e) => (Some(o.0), Some(e.0)),
            };

            if let Some(content) = out {
                if !content.is_empty() {
                    if let Some(mut f) = sout {
                        write!(f, "{}", content)?;
                    } else {
                        print!("{}", content);
                    }
                }
            }
            if let Some(content) = err {
                if !content.is_empty() {
                    if let Some(mut f) = serr {
                        write!(f, "{}", content)?;
                    } else {
                        print!("{}", content);
                    }
                }
            }
        } else {
            println!("{}: command not found", &input.trim())
        }
    }
}
