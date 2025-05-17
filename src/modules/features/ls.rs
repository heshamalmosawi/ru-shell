use crate::shell::Shell;
use libc;
use std::{
    ffi::{CStr, CString, OsString},
    os::unix::ffi::OsStringExt,
    time::{Duration, UNIX_EPOCH},
};

use super::echo::{echo, echoln};

#[derive(Debug, Clone, Copy, PartialEq)]
enum LsFlag {
    LongFormat,
    All,
    Classify,
}

impl LsFlag {
    fn from_char(c: char) -> Option<Self> {
        match c {
            'l' => Some(Self::LongFormat),
            'a' => Some(Self::All),
            'F' => Some(Self::Classify),
            _ => None,
        }
    }

    fn from_long_option(option: &str) -> Option<Self> {
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
        let (flags, paths) = parse_ls_args(args);

        for (i, path) in paths.iter().enumerate() {
            // print the path name as a header  if multiple paths
            if paths.len() > 1 {
                echoln(&format!("{}/:", path));
            }
            let path_to_list = if path == "." {
                self.abs_cwd.clone()
            } else {
                path.to_string()
            };

            let listings = match list_directory(&path_to_list, flags.clone()) {
                Ok(entries) => entries,
                Err(e) => {
                    if e == path_to_list {
                        vec![path_to_list.clone()]
                    } else {
                        self.error(&format!("ls: {}", e), false);
                        continue;
                    }
                }
            };

            for entry in listings {
                if flags.contains(&LsFlag::LongFormat) {
                    // print the long format entry
                    echoln(&entry);
                } else {
                    echo(&(entry + "\t"));
                }
            }

            // print a new line only if not the last path
            if i != paths.len() - 1 {
                echoln("\n");
            }
        }
        echoln("");
    }
}

fn list_directory(dir_path: &str, flags: Vec<LsFlag>) -> Result<Vec<String>, String> {
    let all = flags.contains(&LsFlag::All);
    let long_format = flags.contains(&LsFlag::LongFormat);
    let classify = flags.contains(&LsFlag::Classify);

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
                if name_str.starts_with('.') && !all {
                    continue;
                }

                let full_path = format!("{}/{}", dir_path, name_str);
                let c_full_path = CString::new(full_path.clone()).unwrap();
                let mut stat_buf: libc::stat = std::mem::zeroed();

                if libc::lstat(c_full_path.as_ptr(), &mut stat_buf) != 0 {
                    continue; // skip on error
                }

                let mut display_name = name_str.to_string();
                if classify {
                    let mode = stat_buf.st_mode;
                    let ftype = mode & libc::S_IFMT;
                    match ftype {
                        libc::S_IFDIR => display_name.push('/'),
                        libc::S_IFLNK => display_name.push('@'),
                        libc::S_IFIFO => display_name.push('|'),
                        libc::S_IFSOCK => display_name.push('='),
                        libc::S_IFREG => {
                            if (mode & 0o111) != 0 {
                                display_name.push('*');
                            }
                        }
                        _ => {}
                    }
                }

                let f_type = get_file_type(stat_buf.st_mode);
                if f_type == 'd' {
                    display_name = format!("\x1b[1;34m{}\x1b[0m", display_name);
                } else if f_type == 'l' {
                    display_name = format!("\x1b[1;36m{}\x1b[0m", display_name);
                }
                if long_format {
                    let perms = file_mode_string(stat_buf.st_mode);
                    let nlink = stat_buf.st_nlink;
                    // get user and group names from UIDs and GIDs
                    let uid = stat_buf.st_uid;
                    let gid = stat_buf.st_gid;

                    // Look up and format username
                    let pwd = libc::getpwuid(uid);
                    let uid_name = if !pwd.is_null() {
                        CStr::from_ptr((*pwd).pw_name).to_string_lossy().to_string()
                    } else {
                        uid.to_string() // Fallback to numeric ID if lookup fails
                    };

                    // Look up and format group name
                    let grp = libc::getgrgid(gid);
                    let gid_name = if !grp.is_null() {
                        CStr::from_ptr((*grp).gr_name).to_string_lossy().to_string()
                    } else {
                        gid.to_string() // Fallback to numeric ID if lookup fails
                    };

                    let size = stat_buf.st_size;
                    let mtime = UNIX_EPOCH + Duration::from_secs(stat_buf.st_mtime as u64);
                    let datetime = chrono::DateTime::<chrono::Local>::from(mtime);

                    let formatted = format!(
                        "{} {:>3} {} {} {:>8} {} {}",
                        perms,
                        nlink,
                        uid_name,
                        gid_name,
                        size,
                        datetime.format("%b %d %H:%M"),
                        display_name
                    );
                    entry_names.push(formatted);
                } else {
                    entry_names.push(display_name);
                }
            } else {
                return Err(format!("ls: error converting entry name to string"));
            }
        }
        libc::closedir(dir_ptr);
    }

    // Sort entries
    if long_format {
        // Sort by name when in long format
        entry_names.sort_by(|a, b| {
            let a_parts: Vec<&str> = a.split_whitespace().collect();
            let b_parts: Vec<&str> = b.split_whitespace().collect();

            if a_parts.len() > 0 && b_parts.len() > 0 {
                // Get the filename (last part) when in long format
                let a_name = a_parts.last().unwrap();
                let b_name = b_parts.last().unwrap();
                // strip ANSI color codes for comparison
                let a_name = a_name.replace("\x1b[1;34m", "").replace("\x1b[1;36m", "").replace("\x1b[0m", "");
                let b_name = b_name.replace("\x1b[1;34m", "").replace("\x1b[1;36m", "").replace("\x1b[0m", "");

                a_name.to_lowercase().cmp(&b_name.to_lowercase())
            } else {
                a.cmp(b) // Fallback
            }
        });
    } else {
        entry_names.sort_by(|a, b| {
            // Strip ANSI color codes for comparison
            // not ideal but dont feel like changing alot :)
            let a_clean = a.replace("\x1b[1;34m", "").replace("\x1b[1;36m", "").replace("\x1b[0m", "");
            let b_clean = b.replace("\x1b[1;34m", "").replace("\x1b[1;36m", "").replace("\x1b[0m", "");
            a_clean.to_lowercase().cmp(&b_clean.to_lowercase())
        });
    }

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

fn file_mode_string(mode: u32) -> String {
    let file_type = get_file_type(mode);

    let mut perms = String::new();
    perms.push(file_type);
    let flags = [
        (libc::S_IRUSR, 'r'),
        (libc::S_IWUSR, 'w'),
        (libc::S_IXUSR, 'x'),
        (libc::S_IRGRP, 'r'),
        (libc::S_IWGRP, 'w'),
        (libc::S_IXGRP, 'x'),
        (libc::S_IROTH, 'r'),
        (libc::S_IWOTH, 'w'),
        (libc::S_IXOTH, 'x'),
    ];
    for (bit, chr) in flags.iter() {
        perms.push(if (mode & *bit) != 0 { *chr } else { '-' });
    }
    perms
}

fn get_file_type(mode: u32) -> char {
    match mode & libc::S_IFMT {
        libc::S_IFDIR => 'd',
        libc::S_IFLNK => 'l',
        libc::S_IFCHR => 'c',
        libc::S_IFBLK => 'b',
        libc::S_IFIFO => 'p',
        libc::S_IFSOCK => 's',
        _ => '-',
    }
}
