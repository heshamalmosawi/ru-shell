use super::echo::echoln;
use crate::shell::Shell;
use std::path::{Path, PathBuf};
use std::{fs, io};

impl Shell {
    pub fn handle_copy_command(&mut self, args: Vec<String>) {
        // Parse arguments for flags and paths
        let mut recursive = false;
        let mut paths = Vec::new();

        for arg in args {
            if arg == "-r" || arg == "-R" || arg == "--recursive" {
                recursive = true;
            } else {
                paths.push(arg);
            }
        }

        // Check if we have enough paths
        if paths.len() < 2 {
            self.error("cp: missing file operand", true);
            echoln("Usage: cp [-r] SOURCE... DESTINATION");
            return;
        }

        // Last argument is the destination
        let dest_path = &paths[paths.len() - 1];
        let destination_str = if dest_path.starts_with("/") || dest_path.starts_with("~") {
            dest_path.to_string()
        } else {
            format!("{}/{}", self.abs_cwd, dest_path)
        };

        let destination = Path::new(&destination_str);
        let source_paths: Vec<String> = paths[0..paths.len() - 1]
            .iter()
            .map(|source| {
                if source.starts_with("/") || source.starts_with("~") {
                    source.to_string()
                } else {
                    format!("{}/{}", self.abs_cwd, source)
                }
            })
            .collect();

        // If multiple sources, destination must be a directory
        if source_paths.len() > 1 && !destination.is_dir() {
            self.error(
                &format!("cp: target '{}' is not a directory", destination.display()),
                false,
            );
            return;
        }

        // Process each source
        for source_str in &source_paths {
            let source_path = Path::new(source_str);

            if !source_path.exists() {
                self.error(
                    &format!(
                        "cp: cannot stat '{}': No such file or directory",
                        source_str
                    ),
                    false,
                );
                continue;
            }

            if source_path.is_dir() {
                if !recursive {
                    self.error(
                        &format!("cp: -r not specified; omitting directory '{}'", source_str),
                        false,
                    );
                    continue;
                }

                // Get the destination path for this source
                let dest_path = self.get_destination_path(source_path, destination);

                // Copy directory recursively
                if let Err(e) = self.copy_dir_recursive(source_path, &dest_path) {
                    self.error(&format!("cp: {}", e), false);
                }
            } else {
                // Get the destination path for this source
                let dest_path = self.get_destination_path(source_path, destination);

                // Copy file
                if let Err(e) = self.copy_file(source_path, &dest_path) {
                    self.error(&format!("cp: {}", e), false);
                }
            }
        }
    }

    fn get_destination_path(&self, source: &Path, destination: &Path) -> PathBuf {
        if destination.is_dir() {
            // If destination is a directory, append source filename to it
            destination.join(source.file_name().unwrap())
        } else {
            // Otherwise use the destination as is
            destination.to_path_buf()
        }
    }

    fn copy_file(&self, source: &Path, destination: &Path) -> io::Result<()> {
        // Create parent directories if they don't exist
        if let Some(parent) = destination.parent() {
            fs::create_dir_all(parent)?;
        }

        // Read source file and write to destination
        let content = fs::read(source)?;
        fs::write(destination, content)?;

        // Copy permissions
        if let Ok(metadata) = fs::metadata(source) {
            let permissions = metadata.permissions();
            fs::set_permissions(destination, permissions)?;
        }

        Ok(())
    }

    fn copy_dir_recursive(&self, source: &Path, destination: &Path) -> io::Result<()> {
        // Create destination directory with same permissions
        if !destination.exists() {
            fs::create_dir_all(destination)?;

            // Copy directory permissions
            if let Ok(metadata) = fs::metadata(source) {
                let permissions = metadata.permissions();
                fs::set_permissions(destination, permissions)?;
            }
        }

        // Copy directory contents
        for entry in fs::read_dir(source)? {
            let entry = entry?;
            let entry_path = entry.path();
            let dest_path = destination.join(entry.file_name());

            if entry_path.is_dir() {
                // Recursively copy subdirectory
                self.copy_dir_recursive(&entry_path, &dest_path)?;
            } else {
                // Copy file
                self.copy_file(&entry_path, &dest_path)?;
            }
        }

        Ok(())
    }

    pub fn handle_move_command(&mut self, args: Vec<String>) {
        // Check if we have enough arguments
        if args.len() < 2 {
            self.error("mv: missing file operand", true);
            echoln("Usage: mv SOURCE... DESTINATION");
            return;
        }

        // Last argument is the destination
        let dest_path = &args[args.len() - 1];
        let destination_str = if dest_path.starts_with("/") || dest_path.starts_with("~") {
            dest_path.to_string()
        } else {
            format!("{}/{}", self.abs_cwd, dest_path)
        };

        let destination = Path::new(&destination_str);

        // Process source paths
        let source_paths: Vec<String> = args[0..args.len() - 1]
            .iter()
            .map(|source| {
                if source.starts_with("/") || source.starts_with("~") {
                    source.to_string()
                } else {
                    format!("{}/{}", self.abs_cwd, source)
                }
            })
            .collect();

        // If multiple sources, destination must be a directory
        if source_paths.len() > 1 && !destination.is_dir() {
            self.error(
                &format!("mv: target '{}' is not a directory", destination.display()),
                false,
            );
            return;
        }

        // Process each source
        for source_str in &source_paths {
            let source_path = Path::new(source_str);

            if !source_path.exists() {
                self.error(
                    &format!(
                        "mv: cannot stat '{}': No such file or directory",
                        source_str
                    ),
                    false,
                );
                continue;
            }

            // Get the destination path for this source
            let dest_path = self.get_destination_path(source_path, destination);

            // Check if source and destination are the same
            if source_path == dest_path {
                self.error(
                    &format!(
                        "mv: '{}' and '{}' are the same file",
                        source_str,
                        dest_path.display()
                    ),
                    false,
                );
                continue;
            }

            // Try to rename (fast path - works if on same filesystem)
            if fs::rename(source_path, &dest_path).is_ok() {
                continue;
            }

            // If rename fails (e.g., across filesystems), fall back to copy and delete
            let result = if source_path.is_dir() {
                // Copy directory contents
                let copy_result = self.copy_dir_recursive(source_path, &dest_path);

                // Remove source directory if copy succeeds
                if copy_result.is_ok() {
                    fs::remove_dir_all(source_path)
                } else {
                    copy_result
                }
            } else {
                // Copy file and remove source
                let copy_result = self.copy_file(source_path, &dest_path);

                // Remove source file if copy succeeds
                if copy_result.is_ok() {
                    fs::remove_file(source_path)
                } else {
                    copy_result
                }
            };

            // Handle errors
            if let Err(e) = result {
                self.error(&format!("mv: {}", e), false);
            }
        }
    }
}
