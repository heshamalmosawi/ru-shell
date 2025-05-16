use std::io::Error;

/// Parse command line arguments
pub fn parse_args(input: &str) -> Result<Vec<String>, Error> {
    if input.trim().is_empty() {
        return Ok(Vec::new());
    }

    let mut args = Vec::new();
    let mut current_arg = String::new();
    let mut in_quotes = false;
    let mut escaped = false;

    for c in input.chars() {
        if escaped {
            current_arg.push(c);
            escaped = false;
        } else if c == '\\' {
            escaped = true;
        } else if c == '"' {
            in_quotes = !in_quotes;
        } else if c.is_whitespace() && !in_quotes {
            if !current_arg.is_empty() {
                args.push(current_arg);
                current_arg = String::new();
            }
        } else {
            current_arg.push(c);
        }
    }

    if in_quotes {
        return Err(Error::new(std::io::ErrorKind::InvalidInput, "unclosed quotes"));
    }

    if escaped {
        return Err(Error::new(std::io::ErrorKind::InvalidInput, "trailing backslash"));
    }

    if !current_arg.is_empty() {
        args.push(current_arg);
    }

    Ok(args)
}
