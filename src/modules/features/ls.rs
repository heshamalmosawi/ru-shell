use crate::shell::Shell;
use libc;
use std::{
    ffi::{CStr, CString, OsString},
    os::unix::ffi::OsStringExt,
};

use super::echo::{echo, echoln};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LsFlag {
    LongFormat,
    All,
    Classify,
}

impl LsFlag {
    pub fn from_char(c: char) -> Option<Self> {
        match c {
            'l' => Some(Self::LongFormat),
            'a' => Some(Self::All),
            'F' => Some(Self::Classify),
            _ => None,
        }
    }

    pub fn from_long_option(option: &str) -> Option<Self> {
        match option {
            "long" => Some(Self::LongFormat),
            "all" => Some(Self::All),
            "classify" => Some(Self::Classify),
            _ => None,
        }
    }
}

impl Shell {
    pub fn handle_ls_command(&self, args: Vec<String>) {
        let (_, paths) = parse_ls_args(args);

        for (i, path) in paths.iter().enumerate() {
            // print the path name as a header  if multiple paths
            if paths.len() > 1 {
                echoln(&format!("{}/:", path));
            }

            let listings = match list_directory(path) {
                Ok(entries) => entries,
                Err(e) => {
                    if e == *path {
                        vec![path.clone()]
                    } else {
                        self.error(&format!("ls: {}", e), false);
                        continue;
                    }
                }
            };

            for entry in listings {
                echo(&(entry + " "));
            }

            // print a new line only if not the last path
            if i != paths.len() - 1 {
                echoln("\n");
            }
        }
        echoln("");
    }
}

fn list_directory(dir_path: &str) -> Result<Vec<String>, String> {
    // converting the path to a Clang string to use with LibC
    let c_dir_path: CString = match CString::new(dir_path) {
        Ok(path) => path,
        Err(e) => return Err(format!("ls: error converting to CString '{e}'")),
    };

    let dir_ptr = unsafe { libc::opendir(c_dir_path.as_ptr()) };
    if dir_ptr.is_null() {
        // SAFELY read errno
        let err = unsafe { *libc::__errno_location() };
        let err_msg = match err {
            libc::ENOENT => format!("cannot access '{}': No such file or directory", dir_path),
            libc::ENOTDIR => dir_path.to_string(),
            _ => format!(
                "cannot open directory '{}': {}",
                dir_path,
                std::io::Error::from_raw_os_error(err)
            ),
        };
        return Err(err_msg);
    }

    let mut entry_names: Vec<String> = Vec::new();
    unsafe {
        loop {
            let entry = libc::readdir(dir_ptr);
            if entry.is_null() {
                break;
            }

            // Extract entry name
            let d_name = (*entry).d_name.as_ptr();
            let c_name = CStr::from_ptr(d_name);
            let name = OsString::from_vec(c_name.to_bytes().to_vec());
            if let Some(name_str) = name.to_str() {
                entry_names.push(name_str.to_string());
            } else {
                return Err(format!("ls: error converting entry name to string"));
            }
        }
        libc::closedir(dir_ptr);
    }

    entry_names.sort();
    Ok(entry_names)
}

// parsing command line arguments for the ls command
fn parse_ls_args(args: Vec<String>) -> (Vec<LsFlag>, Vec<String>) {
    let mut flags = Vec::new();
    let mut paths = Vec::new();

    for arg in args {
        if arg.starts_with("--") {
            // if not longer than 2, we mimick the behavoir of ls, ignoring it.
            if arg.len() > 2 {
                // handling long format options (--long, --all, etc.)
                let option = &arg[2..]; // remove the leading "--"
                if let Some(flag) = LsFlag::from_long_option(option) {
                    if !flags.contains(&flag) {
                        flags.push(flag);
                    }
                }
            }
        } else if arg.starts_with('-') {
            if arg.len() > 1 {
                for c in arg.chars().skip(1) {
                    if let Some(flag) = LsFlag::from_char(c) {
                        if !flags.contains(&flag) {
                            flags.push(flag);
                        }
                    }
                }
            }
        } else {
            paths.push(arg);
        }
    }

    // if no paths, then use the current directory
    if paths.is_empty() {
        paths.push(".".to_string());
    }

    (flags, paths)
}
