use libc::{STDOUT_FILENO, write};
use std::ffi::CString;
use std::{env, fs::{File, OpenOptions}, io::{self, BufRead, BufReader, Write}, path::PathBuf};

use super::echo::echoln;

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

        let mut res = Self {
            history: Vec::new(),
            home_dir,
            current_dir: relative_cwd,
            abs_cwd,
        };

        res.load_history().unwrap_or_else(|e| {
            echoln(&format!("Error loading history: {}", e));
        });

        res
    }

    pub fn get_prompt(&self) -> String {
        // printing colors of prompt as inverse of pop-os terminal :)
        format!("\x1b[1;34mru-shell\x1b[0m:\x1b[1;32m{:#}\x1b[0m$ ", self.current_dir)        
    }

    pub fn history_file_path(&self) -> PathBuf {
        PathBuf::from(&self.home_dir).join(".rushistory")
    }
    
    pub fn load_history(&mut self) -> io::Result<()> {
        let history_path = self.history_file_path();
        
        if !history_path.exists() {
            // Create the file if it doesn't exist
            File::create(&history_path)?;
            return Ok(());
        }
        
        let file = File::open(history_path)?;
        let reader = BufReader::new(file);
        
        for line in reader.lines() {
            if let Ok(command) = line {
                if !command.trim().is_empty() {
                    self.history.push(command);
                }
            }
        }
        
        Ok(())
    }
    
    pub fn add_to_history(&mut self, command: String) {
        if command.trim().is_empty() {
            return;
        }
        
        self.history.push(command.clone());
        
        // append to history file
        if let Ok(mut file) = OpenOptions::new()
            .create(true)
            .append(true)
            .open(self.history_file_path())
        {
            let _ = writeln!(file, "{}", command);
        }
    }

    pub fn get_history(&self) -> &Vec<String> {
        &self.history
    }

    pub fn clear(&self) {
        let clear_screen = CString::new("\x1B[2J\x1B[H").expect("CString::new failed");
        unsafe {
            write(
                STDOUT_FILENO,
                clear_screen.as_ptr() as *const _,
                clear_screen.as_bytes().len(),
            );
        }
    }

    pub fn error(&self, msg: &str, show_name: bool) {
        let message = if show_name {
            format!("ru-shell: {}", msg)
        } else {
            msg.to_string()
        };
        echoln(message.as_str());
    }
}
