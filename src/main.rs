// SPDX-License-Identifier: MIT

use std::env;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Read, Write};
use std::process::{Command, Stdio, exit};

fn write_tty(text: &str) {
    let mut tty = match File::options().write(true).open("/dev/tty") {
        Ok(file) => file,
        Err(e) => {
            eprintln!("Error opening /dev/tty for writing: {e}");
            exit(1);
        }
    };

    if let Err(e) = tty.write(text.as_bytes()) {
        eprintln!("Error writing to /dev/tty: {e}");
        exit(1);
    }

    if let Err(e) = tty.flush() {
        eprintln!("Error flushing /dev/tty: {e}");
        exit(1);
    }
}

fn main() {
    // Read all data from stdin
    let mut stdin_data = Vec::new();
    if let Err(e) = io::stdin().read_to_end(&mut stdin_data) {
        eprintln!("Error reading from stdin: {e}");
        exit(1);
    }

    // Get command line arguments (skip program name)
    let args: Vec<String> = env::args().skip(1).collect();

    loop {
        // Execute msmtp with the provided arguments
        let mut child = match Command::new("msmtp")
            .args(&args)
            .stdin(Stdio::piped())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .spawn()
        {
            Ok(child) => child,
            Err(e) => {
                eprintln!("Error starting msmtp: {e}");
                exit(1);
            }
        };

        // Send the stdin data to msmtp
        if let Some(mut stdin) = child.stdin.take()
            && let Err(e) = stdin.write_all(&stdin_data)
        {
            eprintln!("Error writing to msmtp stdin: {e}");
            exit(1);
        }

        // Wait for msmtp to finish and get exit code
        let exit_status = match child.wait() {
            Ok(status) => status,
            Err(e) => {
                eprintln!("Error waiting for msmtp: {e}");
                exit(1);
            }
        };

        // If msmtp succeeded (exit code 0), we're done
        if exit_status.success() {
            exit(0);
        }

        // Get the actual exit code
        let exit_code = exit_status.code().unwrap_or(-1);

        // Ask user if they want to retry
        loop {
            let retry_request = format!("msmtp failed with exit code {exit_code}. Retry? (y/n): ");
            write_tty(&retry_request);

            // Read answer from from /dev/tty
            let tty = match File::open("/dev/tty") {
                Ok(file) => file,
                Err(e) => {
                    eprintln!("Error opening /dev/tty: {e}");
                    exit(1);
                }
            };

            let mut reader = BufReader::new(tty);
            let mut input = String::new();

            match reader.read_line(&mut input) {
                Ok(_) => {
                    let input = input.trim().to_lowercase();
                    match input.as_str() {
                        "y" => break,
                        "n" => exit(exit_code),
                        _ => continue,
                    }
                }
                Err(e) => {
                    eprintln!("Error reading from /dev/tty: {e}");
                    exit(1);
                }
            }
        }
    }
}
