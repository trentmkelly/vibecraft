#![allow(dead_code)]

use std::io::{self, BufRead};
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread::{self, JoinHandle};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConsoleInput {
    pub line: String,
}

pub fn spawn_console_input_thread() -> (Receiver<ConsoleInput>, JoinHandle<()>) {
    let (sender, receiver) = mpsc::channel();
    let handle = thread::Builder::new()
        .name("Server console handler".to_string())
        .spawn(move || {
            let stdin = io::stdin();
            let reader = stdin.lock();
            let _ = read_console_lines(reader, &sender);
        })
        .expect("failed to spawn console input thread");
    (receiver, handle)
}

pub fn read_console_lines<R: BufRead>(
    mut reader: R,
    sender: &Sender<ConsoleInput>,
) -> io::Result<()> {
    let mut line = String::new();
    loop {
        line.clear();
        let read = reader.read_line(&mut line)?;
        if read == 0 {
            return Ok(());
        }
        let command = line.trim_end_matches(['\r', '\n']).to_string();
        if sender.send(ConsoleInput { line: command }).is_err() {
            return Ok(());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{read_console_lines, ConsoleInput};
    use std::io::Cursor;
    use std::sync::mpsc;

    #[test]
    fn console_reader_queues_utf8_lines_without_newline_markers() {
        let (sender, receiver) = mpsc::channel();
        read_console_lines(Cursor::new("list\nsay hello\r\nstop\n"), &sender).unwrap();
        drop(sender);

        let lines = receiver.into_iter().collect::<Vec<_>>();
        assert_eq!(
            lines,
            vec![
                ConsoleInput {
                    line: "list".to_string()
                },
                ConsoleInput {
                    line: "say hello".to_string()
                },
                ConsoleInput {
                    line: "stop".to_string()
                },
            ]
        );
    }

    #[test]
    fn console_reader_stops_cleanly_when_receiver_is_gone() {
        let (sender, receiver) = mpsc::channel();
        drop(receiver);
        read_console_lines(Cursor::new("stop\n"), &sender).unwrap();
    }
}
