pub mod commands;
pub mod engine;
pub mod protocol;

use crate::uci::engine::UciEngine;
use std::io::{self, BufRead, BufReader, Write};
use std::sync::mpsc::{self, Receiver, TryRecvError};
use std::thread;

pub fn run_uci_loop() {
    let mut engine = UciEngine::new();

    // Create a channel for command processing
    let (tx, rx): (mpsc::Sender<String>, Receiver<String>) = mpsc::channel();

    // Spawn input reading thread
    thread::spawn(move || {
        let stdin = io::stdin();
        let reader = BufReader::new(stdin);

        for line in reader.lines() {
            match line {
                Ok(command) => {
                    if tx.send(command).is_err() {
                        break; // Main thread has quit
                    }
                }
                Err(_) => break,
            }
        }
    });

    // Main UCI processing loop - can handle commands even during search
    loop {
        match rx.try_recv() {
            Ok(command) => {
                let command = command.trim();
                if command.is_empty() {
                    continue;
                }

                if command == "quit" {
                    break;
                }

                // Handle uci command specially to send identification
                if command == "uci" {
                    // Send each response on a separate line with proper newlines
                    println!("id name Chess AI Rust");
                    io::stdout().flush().unwrap();

                    println!("id author Chess AI Rust Developer");
                    io::stdout().flush().unwrap();

                    // Send options
                    engine.send_options();

                    // Send uciok last
                    println!("uciok");
                    io::stdout().flush().unwrap();
                } else {
                    engine.handle_command(command);
                }
            }
            Err(TryRecvError::Empty) => {
                // No command available, sleep briefly to avoid busy waiting
                thread::sleep(std::time::Duration::from_millis(1));
            }
            Err(TryRecvError::Disconnected) => {
                // Input thread has quit
                break;
            }
        }
    }
}
