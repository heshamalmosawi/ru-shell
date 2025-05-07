use std::{
    io::{self, Write},
    process,
};

use super::{echo, handle_echo_command};

fn print_prompt() -> io::Result<()> {
    echo("ru-shell$ ");
    io::stdout().flush()
}

pub fn boot() -> io::Result<()> {
    ctrlc::set_handler(move || {
        echo("\nCtrl-C pressed, exiting...\n");
        process::exit(0);
    })
    .expect("Error setting Ctrl-C handler");

    let stdin = io::stdin();
    loop {
        if let Err(e) = print_prompt() {
            echo(format!("Failed to print prompt: {}", e).as_str());
            continue;
        }
        let mut buffer = String::new();

        if let Err(e) = stdin.read_line(&mut buffer) {
            echo(format!("Failed to read line: {}", e).as_str());
            continue;
        } else if buffer.is_empty() {
            // handling EOF (Ctrl-D)
            echo("\nEOF received, exiting...");
            break;
        }

        match buffer.trim() {
            "exit" => {
                break;
            }
            cmd if cmd.starts_with("echo ") || cmd == "echo" => {
                handle_echo_command(cmd);
            }
            "hello" => echo("Hello, world!"),
            _ => echo(format!("{}: command not found", buffer.trim()).as_str()),
        }
    }
    Ok(())
}
