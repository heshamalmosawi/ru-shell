use std::ffi::CString;

use crate::shell::Shell;

/*
    In x86 asm, the syscall number for write is 1.
    write syscall in nasm is 1
    file descriptor for stdout is 1, and for stderr is 2, 0 for stdin.
    then we need to pass a pointer to the string to rsi register and its length to rdx register.
*/

impl Shell {
    pub fn handle_echo_command(arg: &str) {
        // performing sanitization on the input
        let binding = arg
            .trim()
            .split_whitespace()
            .skip(1)
            .collect::<Vec<&str>>()
            .join(" ");
        let mut arg = binding.as_str();

        if arg.starts_with('"') && arg.ends_with('"') {
            arg = &arg[1..arg.len() - 1];
        } else if arg.starts_with('\'') && arg.ends_with('\'') {
            arg = &arg[1..arg.len() - 1];
        }

        let sanitized = if (arg.starts_with('"') && arg.ends_with('"'))
            || (arg.starts_with('\'') && arg.ends_with('\''))
        {
            format!("{}\n", &arg[1..arg.len() - 1])
        } else {
            format!("{}\n", arg)
        };
        echo(sanitized.as_str());
    }
}

pub fn echo(arg: &str) {
    // removing the quotes from the string and
    let c_string = CString::new(arg).expect("CString::new failed");

    unsafe {
        std::arch::asm!(
            "syscall",
            in("rax") 1, // syscall number for write
            in("rdi") 1, // file descriptor for stdout
            in("rsi") c_string.as_ptr(), // pointer to the string
            in("rdx") c_string.as_bytes().len(), // length of the string
        )
    }
}

pub fn echoln(arg: &str) {
    let arg = format!("{}\n", arg);
    echo(arg.as_str());
}
