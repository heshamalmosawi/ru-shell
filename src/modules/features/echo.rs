use std::ffi::CString;

use crate::shell::Shell;

/*
    In x86 asm, the syscall number for write is 1.
    write syscall in nasm is 1
    file descriptor for stdout is 1, and for stderr is 2, 0 for stdin.
    then we need to pass a pointer to the string to rsi register and its length to rdx register.
*/

impl Shell {
    pub fn handle_echo_command(args: Vec<String>) {
        echoln(args.join(" ").as_str());
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
