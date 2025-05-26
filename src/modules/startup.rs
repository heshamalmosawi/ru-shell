use std::io::{self};
use rustyline::Editor;

use super::{echo::echoln, parse_args, shell::Shell};

pub fn boot() -> io::Result<()> {

    let mut rl = Editor::<(), _>::new().unwrap();
    let mut inst = Shell::new();
    // load history if it exists
    if let Err(e) = rl.load_history(&inst.history_file_path()) {
        echoln(&format!("Error loading history: {}", e));
    }

    loop {
        match rl.readline(inst.get_prompt().as_str()) {
            Ok(line) => {
                if line.trim().is_empty() {
                    continue;
                }
                let _ = rl.add_history_entry(line.clone());
                inst.add_to_history(line.clone());
                // splitting the line into command and arguments
                let parts: Vec<&str> = line.split_whitespace().collect();
                let command : &str = parts[0];
                let args = match parse_args(&line[command.len()..]) {
                    Ok(parsed_args) => parsed_args,
                    Err(e) => {
                        inst.error(&e.to_string(), true);
                        continue;
                    }
                };


                match command {
                    "exit" => {
                        break;
                    }
                   "echo" => Shell::handle_echo_command(args),
                    "cd" => {
                        if args.len() > 1 {
                            inst.error("cd: too many arguments", true);
                            continue;
                        }
                        let path = if !args.is_empty() { args[0].trim() } else { "" };
                        let res = inst.cd(path);
                        if let Err(e) = res {
                            echoln(e.as_str());
                        }
                    }
                    "pwd" => inst.pwd(),
                    "ls" => inst.handle_ls_command(args),
                    "cat" => inst.handle_cat_command(args),
                    "cp" => inst.handle_copy_command(args),
                    "mv" => inst.handle_move_command(args),
                    "mkdir" => inst.handle_mkdir_command(args),
                    "rm" => inst.handle_rm_command(args),
                    "clear" => inst.clear(),
                    _ => inst.error(format!("{}: command not found", line.trim()).as_str(), false),
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
                inst.error(&e.to_string(), false);
                continue;
            }
        }
    }
    Ok(())
}
