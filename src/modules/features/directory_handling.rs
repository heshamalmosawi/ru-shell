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
        let mut current_dir: String;

        if path.starts_with('/') {
            // meaning user entered absolute path
            abs_dir = path.to_string();
            current_dir = path.replace(&self.home_dir, "~").to_string();
        } else if path.starts_with('~') {
            // meaning user entered path starting with home dir
            abs_dir = path.replace('~', &self.home_dir).to_string();
            current_dir = path.replace(&self.home_dir, "~").to_string();
        } else if path.starts_with("../") && !path[3..].starts_with(' ') {
            //  entering directory from parent
            let mut parts: Vec<&str> = self.abs_cwd.split('/').collect();
            parts.pop();
            abs_dir = parts.join("/") + "/" + &path[3..];
            current_dir = if self.abs_cwd == self.home_dir {
                // if current path is home, rely on absolute to get the parent
                let mut parts: Vec<&str> =
                    self.abs_cwd.split('/').filter(|s| !s.is_empty()).collect();
                parts.pop();
                format!("/{:#}", parts.join("/") + "/" + &path[3..])
            } else {
                let mut current_parts: Vec<&str> = self.current_dir.split('/').collect();
                current_parts.pop();
                current_parts.join("/") + "/" + &path[3..]
            };
        } else if path.starts_with("..") {
            // entering parent
            let mut parts: Vec<&str> = self.abs_cwd.split('/').collect();
            parts.pop();
            abs_dir = parts.join("/");

            current_dir = if self.abs_cwd == self.home_dir {
                // if current path is home, rely on absolute to get the parent
                let mut parts: Vec<&str> =
                    self.abs_cwd.split('/').filter(|s| !s.is_empty()).collect();
                parts.pop();
                format!("/{:#}", parts.join("/"))
            } else {
                let mut current_parts: Vec<&str> = self.current_dir.split('/').collect();
                current_parts.pop();
                current_parts.join("/")
            };
        } else {
            abs_dir = if self.abs_cwd == "/" {
                format!("/{}", path)
            } else {
                format!("{}/{}", self.abs_cwd, path)
            };

            current_dir = if self.current_dir == "/" {
                format!("/{}", path)
            } else {
                format!("{}/{}", self.current_dir, path)
            };
        }

        if abs_dir == "" {
            // if abs dir is empty, the root has entered root
            abs_dir = "/".to_string();
            current_dir = "/".to_string();
        }

        // checking path validity
        path_exists(&abs_dir)
            .map_err(|_| format!("ru-shell: cd: {}: No such file or directory", abs_dir))?;

        // if previous abs path is the home,
        self.abs_cwd = abs_dir;
        self.current_dir = current_dir;

        Ok(())
    }
}

pub fn path_exists(path: &str) -> Result<Metadata, std::io::Error> {
    fs::metadata(path)
}
