use std::{
    io::{self},
    process,
};

use super::{echo::echoln, shell::Shell};


pub fn boot() -> io::Result<()> {
    ctrlc::set_handler(move || {
        echoln("\nCtrl-C pressed, exiting...");
        process::exit(0);
    })
    .expect("Error setting Ctrl-C handler");

    let stdin = io::stdin();
    let mut inst = Shell::new();
    loop {
        if let Err(e) = inst.print_prompt() {
            echoln(format!("Failed to print prompt: {}", e).as_str());
            continue;
        }
        let mut buffer = String::new();

        if let Err(e) = stdin.read_line(&mut buffer) {
            echoln(format!("Failed to read line: {}", e).as_str());
            continue;
        } else if buffer.is_empty() {
            // handling EOF (Ctrl-D)
            echoln("\nEOF received, exiting...");
            break;
        }

        match buffer.trim() {
            "exit" => {
                break;
            }
            cmd if cmd.starts_with("echo ") || cmd == "echo" => {
                Shell::handle_echo_command(cmd);
            },
            cmd if cmd.starts_with("cd ") || cmd == "cd" => {
                let path = if cmd == "cd" {
                    ""
                } else {
                    &cmd[3..]
                };
                let res = inst.cd(path);
                if let Err(e) = res {
                    echoln(e.as_str());
                }
            }
            "pwd" => inst.pwd(),
            _ => echoln(format!("{}: command not found", buffer.trim()).as_str()),
        }
    }
    Ok(())
}
