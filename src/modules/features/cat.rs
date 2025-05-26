use std::{fs, path::Path};

use crate::shell::Shell;

use super::echo::echoln;

impl Shell {
    pub fn handle_cat_command(&self, args: Vec<String>) {
        if args.is_empty() {
            self.error("cat: missing file", false);
            return;
        }

        for arg in args {
            let file_name_str = if arg.starts_with('/') {
                arg.clone()
            } else if arg.starts_with("~") {
                arg.replace("~", &self.home_dir)
            } else {
                format!("{}/{}", self.abs_cwd, arg)
            };

            let path = Path::new(&file_name_str);
            if !path.exists() {
                self.error(format!("cat: {}: No such file or directory", arg).as_str(), false);
                continue;
            }

            match fs::read_to_string(path) {
                Ok(content) => echoln(content.as_str()),
                Err(e) => self.error(format!("cat: {}: {}", arg, e).as_str(), false),
            }
        }
    }
}