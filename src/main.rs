use std::fs::OpenOptions;
use std::process::{Command};

const VERSION: &str = "1.0.0";

struct Error(String);

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

fn load_file<'a >(name: &str, current_dir: &str, strict: bool, soft: bool, compare: bool) -> Result<String,Error> {
    let path = if name.starts_with("/") {
        name.to_string()
    } else {
        format!("{}/{}", current_dir, name)
    };

    let status = OpenOptions::new()
        .write(true).read(true)
        .create((!(strict || soft) && !compare) || (soft && compare)).open(&path);

    if let (Err(e), true)  = (&status, strict||compare && !soft) { return Err(Error(format!("Failed to load '{}'. {}", path, e.to_string()))) }
    if let (Err(_), true) = (&status, soft && !compare) { return Ok("".to_string()) }
    Ok(path)
}

macro_rules! e_error {
    ($msg:expr) => {{
        eprintln!("Error: {}", $msg); panic!()
    }};
}

fn print_help() {
    println!(
"Two in one command line tool. 'hexf' and 'synalyze' in one place.

SYNOPSIS:
    syn [-x|-xo] [-s|-ss] [file1 file2 ...] 
    syn -c [-ss] file1 file2

INFO:
    Both 'hexf' (intalled by 'Hex Fiend') and 'synalyze' (installed by 'Synalyze It!') must be installed and available within the $PATH.
    By default, all files are only opened with 'Synalyze It!'. You can open files only with 'Hex Fiend' by using `-xo` or `-x` to open files with both 'Synalyze It!' (synalyze) and 'Hex Fiend' (hexf).
    By default, when a file does not exists the program creates it and opens it. You can disable this by using strict mode `-s` to finish the program when a file or directory does not exist or use soft-strict mode `-ss` to ignore nonexistent files and do not panic the program.
    When syn has no arguments it will open the specified app (which you can specify using `-x`, `-xo` or nothing) with a new empty file Untitled (disable startup welcome panel in 'Synalyze It!' preferences).
    Due to 'compare' `-c` strict default mode, `-ss` can be used with it to enable missing file(s) creation.

OPTIONS:
    -c,  --compare          Opens two given files with 'Hex Fiend' compare mode. Panics with nonexistent files, you can use `-ss` to create them.

    -s,  --strict           Strict mode, panics when a nonexistent file is given.
    -ss, --soft-strict      Soft strict mode, ignores nonexistent files and opens all valid ones.

    -x,  --hexf             Opens all files with 'Hex Fiend' along with 'Synalyze It!'.
    -xo, --hexf-only        Opens all files only with 'Hex Fiend'.

    -h,  --help             Prints this message.
    -v,  --version          Print syn version.
    
EXAMPLES:
    syn file.txt                    -- Opens 'file.txt' with 'Synalyze It!'
    syn -x file.txt                 -- Opens 'file.txt' with both 'Synalyze It!' and 'Hex Fiend'
    syn -xo file.txt                -- Opens 'file.txt' with 'Hex Fiend'
    syn -c -ss file1 file2          -- Compares file1 and file2 with 'Hex Fiend'. If a file does not exist, it creates it.
    syn -c file1 file2              -- Compares file1 and file2 with 'Hex Fiend'. If a file does not exist, panics... fails.
    syn -x -ss file1 file2 file3    -- Opens 'file1', 'file2' and 'file3' with both 'Synalyze It!' and 'Hex Fiend'. If a file does not exists it just ignores it and opens all other files.
    "
        
    );
    panic!()
}

macro_rules! hexf_open_imp {
    ($paths: expr) => {
        let state = Command::new("hexf").args($paths).output();
        if let Err(e) = state {
            e_error!(format!("Maybe 'hexf' is not installed? Make sure it is installed and available in the $PATH. {}", e.to_string()));
        }
    };
}

macro_rules! hexf_open {
    ($paths: expr, $files_num: expr) => {
        if $files_num > 0 {
            hexf_open_imp!($paths);
        } else {
            hexf_open_imp!(&["Untitled"]);
        }
    };
}

trait MkTrue {
    fn mk_true(&mut self);
}

impl MkTrue for bool {
    fn mk_true(&mut self) {
        *self = true
    }
}

fn main() -> Result<(),Error> {
    std::panic::set_hook(Box::new(|_info| {}));

    let mut args: Vec<String> = std::env::args().collect::<Vec<String>>(); args.remove(0);
    let current_dir = std::env::current_dir()?.display().to_string();

    let mut paths: Vec<String> = Vec::new();

    let mut compare = false;
    let mut strict_mode = false;
    let mut strict_soft_mode = false;
    let mut hexf = false;
    let mut hexf_only = false;
    
    args.retain( |x: &String| {
        if x.starts_with("-") {
            match &x[..] {
                "--help" | "-h" => print_help(),
                "--version" | "-v" => {println!("syn {}", VERSION); panic!()},
                "-s" | "--strict" => strict_mode.mk_true(),
                "-ss"| "--soft-strict" => strict_soft_mode.mk_true(),
                "-c" |  "--compare" => compare.mk_true(),
                "-x" |  "--hexf"  =>  hexf.mk_true(),
                "-xo"| "--hexf-only" => hexf_only.mk_true(),
                _ => ()
            };
            return false;
        }
        true
    });

    if compare && (hexf_only || hexf || strict_mode) {
        e_error!("Invalid input: '-c', '--compare' flag works only with '-ss' flag.")
    }
    if hexf && hexf_only { e_error!("Invalid input: Can't use both flags '-x' and '-xo'") }
    if strict_mode && strict_soft_mode { e_error!("Invalid input: Can't use both flags '-s' and '-ss'") }

    for arg in args {
        paths.push(
            match load_file(&arg, &current_dir, strict_mode, strict_soft_mode, compare) {
                Err(e) => { e_error!(e.0); },
                Ok(v)  => if v != ""  { v } else { continue }
            }
        );
    }

    let files_num = paths.len();

    if compare {
        if files_num == 2 {
            let state = Command::new("hexf").arg("-d").args(&paths).output();
            if let Err(e) = state {
                e_error!(format!("Maybe 'hexf' is not installed? Make sure it is installed and available in the $PATH. {}", e.to_string()));
            }
            panic!()
        } else {
            e_error!("Invalid input: '-c', '--compare' requires two files.")
        }
    }
    
    
    if hexf_only {
        hexf_open!(&paths, files_num);
    } else {
        loop {
            let state = Command::new("synalyze").args(&paths).output();

            let state = if let Err(e) = state {
                e_error!(format!("Maybe 'synalyze' is not installed? Make sure it is installed and available in the $PATH. {}", e.to_string()));
            } else { state.unwrap() };

            if state.status.success() { break };
        }

        if hexf {
            hexf_open!(&paths, files_num);
        }
    }


    Ok(())
}