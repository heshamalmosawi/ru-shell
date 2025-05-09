use std::fs::{self, Metadata};

use crate::shell::Shell;

use super::echo::echoln;

impl Shell {
    pub fn pwd(&self) {
        echoln(self.abs_cwd.as_str());
    }

    pub fn cd(&mut self, path: &str) -> Result<(), String> {
        let path = path.trim();

        // if path empty, set to home dir
        if path.is_empty() {
            self.current_dir = "~".to_string();
            self.abs_cwd = self.home_dir.clone();
            println!("current dir: {:#}", self.current_dir);
            println!("abs cwd: {:#}", self.abs_cwd);
        }

        // setting the path as a string
        let mut abs_dir: String;

        if path.starts_with('/') {
            // meaning user entered absolute path
            abs_dir = path.to_string();
        } else if path.starts_with('~') {
            // meaning user entered path starting with home dir
            abs_dir = path.replace('~', &self.home_dir).to_string();
        } else if path.starts_with("../") && !path[3..].starts_with(' ') {
            //  entering directory from parent
            let mut parts: Vec<&str> = self.abs_cwd.split('/').collect();
            parts.pop();
            abs_dir = parts.join("/") + "/" + &path[3..];
        } else if path.starts_with("..") {
            // entering parent
            let mut parts: Vec<&str> = self.abs_cwd.split('/').collect();
            parts.pop();
            abs_dir = parts.join("/");
        } else {
            abs_dir = if self.abs_cwd == "/" {
                format!("/{}", path)
            } else {
                format!("{}/{}", self.abs_cwd, path.strip_prefix("./").unwrap_or(path))
            };
        }

        if abs_dir == "" {
            // if abs dir is empty, the root has entered root
            abs_dir = "/".to_string();
        }

        // checking path validity
        path_exists_dir(&abs_dir)?;

        // if previous abs path is the home,
        self.current_dir = if abs_dir.starts_with(&self.home_dir) {
            abs_dir.replace(&self.home_dir, "~")
        } else {
            abs_dir.clone()
        };
        self.abs_cwd = abs_dir;

        Ok(())
    }
}

pub fn path_exists_dir(path: &str) -> Result<Metadata, String> {
    let metadata = fs::metadata(path).map_err(|_| format!("ru-shell: cd: {}: No such file or directory", path))?;
    if metadata.is_dir() {
        Ok(metadata)
    } else {
        Err(format!("ru-shell: cd: {}: Not a directory", path))
    }
}
