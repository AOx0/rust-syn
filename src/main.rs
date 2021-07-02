use std::fs::OpenOptions;
use std::process::{Command};
use std::io::ErrorKind as E;
use lazy_static::*;

pub struct Error(String);

impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Self {
        Error(format!("{}", error))
    }
}

impl std::fmt::Debug for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        write!(f, "{}", self.0)
    }
}

impl std::convert::From<std::string::FromUtf8Error> for Error {
    fn from(error: std::string::FromUtf8Error) -> Self {
        Error(format!("{}", error))
    }
}

fn load_file<'a >(name: &String) -> Result<String,Error> {
    let path = if name.starts_with("/") {
            name.to_string()
        } else {
            format!("{}/{}", *CURRENT_DIR, name)
    };

    let status = OpenOptions::new()
        .write(true)
        .read(true)
        .create(true)
        .open(&path);


    match status {
        Err(e) => Err(match  &e.kind() {
            E::PermissionDenied => Error(format!("Failed to load '{}'. Permission Denied", path)),
            _ => e.into()
        }),
        Ok(_) => Ok(path)
    }

}

lazy_static! {
    static ref CURRENT_DIR: String = {
        std::env::current_dir().unwrap().display().to_string()
    };
}

fn main() -> Result<(),Error> {
    let mut args: Vec<String> = std::env::args().collect::<Vec<String>>(); args.remove(0);
    lazy_static::initialize(&CURRENT_DIR);

    let mut paths: Vec<String> = Vec::new();
    for path in args {
        paths.push(load_file(&path)?)
    }

    loop {
        let state = Command::new("synalyze").args(&paths).output().ok().ok_or(Error("'synalyze' not found. Make sure it is installed and available in the $PATH".to_string()))?;
        if state.status.success() { break };
    }
    Ok(())
}