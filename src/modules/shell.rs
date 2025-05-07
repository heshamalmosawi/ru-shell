use std::env;

#[allow(dead_code)]
pub struct Shell {
    pub history: Vec<String>,
    pub relative_cwd: String,   // represents the relative path to the current working directory
    pub abs_cwd: String,        // represents the absolute path to the current working directory
}

impl Shell {
    pub fn new() -> Self {

        let relative_cwd = match dirs::home_dir() {
            Some(path) => path.to_str().unwrap_or("/").to_string(),
            None => String::from("/"),
        };

        let dir = env::current_dir();
        let abs_cwd = match dir {
            Ok(path) => path.to_str().unwrap_or("/").to_string(),
            Err(_) => String::from("/"),
        };

        Self {
            history: Vec::new(),
            relative_cwd,
            abs_cwd,
        }
    }

    pub fn add_to_history(&mut self, command: String) {
        self.history.push(command);
    }

    pub fn get_history(&self) -> &Vec<String> {
        &self.history
    }

    
}