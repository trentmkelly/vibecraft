#![allow(dead_code)]

use std::io::{self, BufRead};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::sync::mpsc::{self, Receiver, Sender};
use std::time::Duration;
use std::thread::{self, JoinHandle};

use crate::command_execution::CommandSourceStackModel;

#[derive(Debug, Clone, PartialEq)]
pub struct ConsoleInput {
    pub msg: String,
    pub source: CommandSourceStackModel,
}

impl ConsoleInput {
    pub fn new(msg: impl Into<String>, source: CommandSourceStackModel) -> Self {
        Self {
            msg: msg.into(),
            source,
        }
    }

    pub fn line(&self) -> &str {
        &self.msg
    }
}

pub fn spawn_console_input_thread() -> io::Result<(Receiver<ConsoleInput>, JoinHandle<()>)> {
    let (sender, receiver) = mpsc::channel();
    install_shutdown_signal_watcher(sender.clone())?;
    let handle = thread::Builder::new()
        .name("Server console handler".to_string())
        .spawn(move || {
            let stdin = io::stdin();
            let reader = stdin.lock();
            let _ = read_console_lines(reader, &sender);
        })?;
    Ok((receiver, handle))
}

/// Equivalent of the JVM shutdown hook Java's `DedicatedServer.initServer`
/// registers ("Server Shutdown Thread" calling `MinecraftServer.halt(true)`):
/// SIGINT/SIGTERM are translated into the console `stop` command so the live
/// accept loop takes the same graceful-stop path as typing `stop`.
fn install_shutdown_signal_watcher(sender: Sender<ConsoleInput>) -> io::Result<()> {
    let signalled = Arc::new(AtomicBool::new(false));
    for signal in [signal_hook::consts::SIGINT, signal_hook::consts::SIGTERM] {
        signal_hook::flag::register(signal, Arc::clone(&signalled))?;
    }
    thread::Builder::new()
        .name("Server shutdown signal watcher".to_string())
        .spawn(move || {
            while !signalled.load(Ordering::SeqCst) {
                thread::sleep(Duration::from_millis(50));
            }
            let _ = sender.send(ConsoleInput::new("stop", console_command_source()));
        })?;
    Ok(())
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
        if sender
            .send(ConsoleInput::new(command, console_command_source()))
            .is_err()
        {
            return Ok(());
        }
    }
}

fn console_command_source() -> CommandSourceStackModel {
    CommandSourceStackModel::new("Server", "overworld", 4)
}

#[cfg(test)]
mod tests {
    use super::{console_command_source, read_console_lines, ConsoleInput};
    use std::io::Cursor;
    use std::sync::mpsc;

    #[cfg(vibecraft_has_decompiled_sources)]
    const JAVA_SOURCE: &str = vibecraft_java_source!("/net/minecraft/server/ConsoleInput.java");

    #[cfg(vibecraft_has_decompiled_sources)]
    #[test]
    fn server_utility_console_input_matches_java_source_shape() {
        assert!(JAVA_SOURCE.contains("public class ConsoleInput"));
        assert!(JAVA_SOURCE.contains("public final String msg;"));
        assert!(JAVA_SOURCE.contains("public final CommandSourceStack source;"));
        assert!(JAVA_SOURCE.contains("public ConsoleInput(final String msg, final CommandSourceStack source)"));
        assert!(JAVA_SOURCE.contains("this.msg = msg;"));
        assert!(JAVA_SOURCE.contains("this.source = source;"));
    }

    #[test]
    fn server_utility_console_input_reader_queues_utf8_lines_with_source() -> std::io::Result<()> {
        let (sender, receiver) = mpsc::channel();
        read_console_lines(Cursor::new("list\nsay hello\r\nstop\n"), &sender)?;
        drop(sender);

        let lines = receiver.into_iter().collect::<Vec<_>>();
        let source = console_command_source();
        assert_eq!(
            lines,
            vec![
                ConsoleInput::new("list", source.clone()),
                ConsoleInput::new("say hello", source.clone()),
                ConsoleInput::new("stop", source),
            ]
        );
        Ok(())
    }

    #[test]
    fn sigterm_is_translated_into_console_stop_command() -> std::io::Result<()> {
        let (sender, receiver) = mpsc::channel();
        super::install_shutdown_signal_watcher(sender)?;
        signal_hook::low_level::raise(signal_hook::consts::SIGTERM)?;
        let input = receiver
            .recv_timeout(std::time::Duration::from_secs(5))
            .expect("stop command from signal");
        assert_eq!(input.line(), "stop");
        Ok(())
    }

    #[test]
    fn server_utility_console_input_reader_stops_cleanly_when_receiver_is_gone(
    ) -> std::io::Result<()> {
        let (sender, receiver) = mpsc::channel();
        drop(receiver);
        read_console_lines(Cursor::new("stop\n"), &sender)
    }

    #[test]
    fn server_utility_console_input_line_accessor_returns_java_msg_field() {
        let input = ConsoleInput::new("reload", console_command_source());

        assert_eq!(input.msg, "reload");
        assert_eq!(input.line(), "reload");
        assert_eq!(input.source.source, "Server");
        assert_eq!(input.source.permission_level, 4);
    }
}
