use std::io::{self};
use rustyline::Editor;

use super::{echo::echoln, shell::Shell};

pub fn boot() -> io::Result<()> {

    let mut rl = Editor::<(), _>::new().unwrap();
    let mut inst = Shell::new();
    loop {
        match rl.readline(inst.get_prompt().as_str()) {
            Ok(line) => {
                if line.trim().is_empty() {
                    continue;
                }
                let _ = rl.add_history_entry(line.clone());

                match line.trim() {
                    "exit" => {
                        break;
                    }
                    cmd if cmd.starts_with("echo ") || cmd == "echo" => {
                        Shell::handle_echo_command(cmd);
                    }
                    cmd if cmd.starts_with("cd ") || cmd == "cd" => {
                        let path = if cmd == "cd" { "" } else { &cmd[3..] };
                        let res = inst.cd(path);
                        if let Err(e) = res {
                            echoln(e.as_str());
                        }
                    }
                    "pwd" => inst.pwd(),
                    "clear" => inst.clear(),
                    _ => echoln(format!("{}: command not found", line.trim()).as_str()),
                }
            }
            Err(rustyline::error::ReadlineError::Interrupted) => {
                echoln("\nCtrl-C pressed, exiting...");
                break;
            }
            Err(rustyline::error::ReadlineError::Eof) => {
                echoln("\nEOF received, exiting...");
                break;
            }
            Err(e) => {
                echoln(format!("Error reading line: {}", e).as_str());
                continue;
            }
        }
    }
    Ok(())
}
