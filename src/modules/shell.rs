use libc::{STDOUT_FILENO, write};
use std::ffi::CString;
use std::{env, path::PathBuf};

#[allow(dead_code)]
pub struct Shell {
    pub history: Vec<String>,
    pub home_dir: String,
    pub current_dir: String,    // represents the current path that will be used for stdout
    pub abs_cwd: String,        // represents the absolute path to the current working directory
}

impl Shell {
    pub fn new() -> Self {
        let home_path = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/"));
        let home_dir = home_path.to_str().unwrap_or("/").to_string();

        let abs_path = env::current_dir().unwrap_or_else(|_| PathBuf::from("/"));
        let abs_cwd = abs_path.to_str().unwrap_or("/").to_string();

        // Replace home_dir in abs_cwd with ~ for relative_cwd
        let relative_cwd = if abs_cwd.starts_with(&home_dir) {
            abs_cwd.replacen(&home_dir, "~", 1)
        } else {
            abs_cwd.clone()
        };

        Self {
            history: Vec::new(),
            home_dir,
            current_dir: relative_cwd,
            abs_cwd,
        }
    }

    pub fn get_prompt(&self) -> String {
        // printing colors of prompt as inverse of pop-os terminal :)
        format!("\x1b[1;34mru-shell\x1b[0m: \x1b[1;32m{:#}\x1b[0m$ ", self.current_dir)        
    }

    pub fn add_to_history(&mut self, command: String) {
        self.history.push(command);
    }

    pub fn get_history(&self) -> &Vec<String> {
        &self.history
    }

    pub fn clear(&self) {
        let clear_screen = CString::new("\x1B[2J\x1B[H").unwrap();
        unsafe {
            write(
                STDOUT_FILENO,
                clear_screen.as_ptr() as *const _,
                clear_screen.as_bytes().len(),
            );
        }
    }
}
