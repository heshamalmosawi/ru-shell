use crate::shell::Shell;
use std::fs;
use std::path::Path;

impl Shell {
    pub fn handle_mkdir_command(&mut self, args: Vec<String>) {
        if args.is_empty() {
            self.error("mkdir: missing operand", true);
            return;
        }

        for dir in args {
            let path_str = if dir.starts_with('/') {
                dir.clone()
            } else if dir.starts_with("~") {
                dir.replace("~", &self.home_dir)
            } else {
                format!("{}/{}", self.abs_cwd, dir)
            };

            let path = Path::new(&path_str);

            if path.exists() {
                self.error(
                    &format!("mkdir: cannot create directory '{}': File exists", dir),
                    false,
                );
                continue;
            }

            // Try to create the directory
            if let Err(e) = fs::create_dir(path) {
                self.error(
                    &format!("mkdir: cannot create directory '{}': {}", dir, e),
                    false,
                );
            }
        }
    }

    pub fn handle_rmdir_command(&mut self, args: Vec<String>) {
        if args.is_empty() {
            self.error("rmdir: missing operand", true);
            return;
        }

        let mut recursive = false;
        let filtered_args: Vec<String> = args
            .iter()
            .filter(|arg| {
                if **arg == "-r" {
                    recursive = true;
                    false
                } else {
                    true
                }
            })
            .cloned()
            .collect();

        if filtered_args.is_empty() {
            self.error("rmdir: missing operand", true);
            return;
        }

        for dir in filtered_args {
            let path_str = if dir.starts_with('/') {
                dir.clone()
            } else if dir.starts_with("~") {
                dir.replace("~", &self.home_dir)
            } else {
                format!("{}/{}", self.abs_cwd, dir)
            };

            let path = Path::new(&path_str);

            if !path.exists() {
                self.error(
                    &format!("rmdir: cannot remove '{}': No such file or directory", dir),
                    false,
                );
                continue;
            }

            // Try to remove the directory
            let result = if recursive {
                fs::remove_dir_all(path)
            } else {
                fs::remove_dir(path)
            };

            if let Err(e) = result {
                self.error(&format!("rmdir: cannot remove '{}': {}", dir, e), false);
            }
        }
    }

    pub fn handle_rm_command(&mut self, args: Vec<String>) {
        if args.is_empty() {
            self.error("rm: missing operand", true);
            return;
        }

        // Parse for -r flag
        let mut recursive = false;
        let mut paths = Vec::new();

        for arg in args {
            if arg == "-r" || arg == "-R" || arg == "--recursive" {
                recursive = true;
            } else {
                paths.push(arg);
            }
        }

        if paths.is_empty() {
            self.error("rm: missing operand", true);
            return;
        }

        for path_arg in paths {
            let path_str = if path_arg.starts_with('/') {
                path_arg.clone()
            } else if path_arg.starts_with("~") {
                path_arg.replace("~", &self.home_dir)
            } else {
                format!("{}/{}", self.abs_cwd, path_arg)
            };

            let path = Path::new(&path_str);

            if !path.exists() {
                self.error(
                    &format!(
                        "rm: cannot remove '{}': No such file or directory",
                        path_arg
                    ),
                    false,
                );
                continue;
            }

            // Handle directories
            if path.is_dir() {
                if !recursive {
                    self.error(
                        &format!("rm: cannot remove '{}': Is a directory", path_arg),
                        false,
                    );
                    continue;
                }

                // Remove directory recursively
                if let Err(e) = fs::remove_dir_all(path) {
                    self.error(&format!("rm: cannot remove '{}': {}", path_arg, e), false);
                }
            } else {
                // Remove file
                if let Err(e) = fs::remove_file(path) {
                    self.error(&format!("rm: cannot remove '{}': {}", path_arg, e), false);
                }
            }
        }
    }
}
