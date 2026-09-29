//! Tests for console/RCON command execution ([`super::ServerCommandRunner`]) including an
//! end-to-end RCON socket round trip on an ephemeral port.

use super::*;
use super::server_command_runner::format_translation;
use crate::game_rules::{GameRules, LiveGameRules};
use crate::network::rcon::{
    read_packet, spawn_rcon_server, write_packet, RconPacket, SERVERDATA_AUTH,
    SERVERDATA_AUTH_RESPONSE, SERVERDATA_EXECCOMMAND, SERVERDATA_RESPONSE_VALUE,
};
use std::net::TcpStream;
use std::sync::mpsc;

fn runner_with(properties: ServerProperties) -> ServerCommandRunner {
    ServerCommandRunner::new(
        &properties,
        1234,
        ActiveLoginRegistry::default(),
        LiveGameRules::shared(GameRules::new(false)),
        Arc::new(Mutex::new(PlayerAccess::default())),
    )
}

fn default_runner() -> ServerCommandRunner {
    let properties = ServerProperties::load_or_default(Path::new("/nonexistent/server.properties"))
        .expect("defaults");
    runner_with(properties)
}

#[test]
fn rcon_response_is_the_plain_feedback_text() {
    let runner = default_runner();
    assert_eq!(runner.run_rcon("seed"), "Seed: 1234");
    // Java `performPrefixedCommand` also accepts a leading slash.
    assert_eq!(runner.run_rcon("/seed"), "Seed: 1234");
}

#[test]
fn rcon_list_reports_the_live_roster() {
    let runner = default_runner();
    assert!(runner
        .run_rcon("list")
        .starts_with("There are 0 of a max of "));
}

#[test]
fn rcon_stop_halts_the_server() {
    let runner = default_runner();
    assert!(!runner.halt_requested());
    let response = runner.run_rcon("stop");
    assert!(runner.halt_requested());
    #[cfg(vibecraft_has_decompiled_sources)]
    assert_eq!(response, "Stopping the server");
    let _ = response;
}

#[cfg(vibecraft_has_decompiled_sources)]
#[test]
fn rcon_unknown_command_reports_parse_error_with_context() {
    let runner = default_runner();
    assert_eq!(
        runner.run_rcon("frobnicate now"),
        "Unknown or incomplete command. See below for error\
         frobnicate now<--[HERE]"
    );
}

#[test]
fn console_input_queue_is_drained_and_stop_is_detected() {
    let runner = default_runner();
    let (sender, receiver) = mpsc::channel();
    let source = crate::command_execution::CommandSourceStackModel::new("Server", "overworld", 4);
    sender
        .send(ConsoleInput::new("seed", source.clone()))
        .expect("queue");
    assert!(!handle_console_inputs(&receiver, &runner));
    sender.send(ConsoleInput::new("stop", source)).expect("queue");
    assert!(handle_console_inputs(&receiver, &runner));
}

#[test]
fn translation_formatting_matches_java_placeholders() {
    let args = ["a".to_string(), "b".to_string()];
    assert_eq!(format_translation("[%s: %s]", &args), "[a: b]");
    assert_eq!(format_translation("%2$s then %1$s", &args), "b then a");
    assert_eq!(format_translation("100%% sure", &args), "100% sure");
}

fn send(stream: &mut TcpStream, request_id: i32, packet_type: i32, payload: &str) {
    write_packet(
        stream,
        &RconPacket {
            request_id,
            packet_type,
            payload: payload.to_string(),
        },
    )
    .expect("send");
}

fn receive(stream: &mut TcpStream) -> RconPacket {
    read_packet(stream).expect("read").expect("packet")
}

#[test]
fn rcon_socket_round_trip_authenticates_and_executes_commands() {
    let runner = Arc::new(default_runner());
    let handler = Arc::clone(&runner);
    let (_thread, address) = spawn_rcon_server("127.0.0.1", 0, "hunter2".to_string(), false, move |command| {
        handler.run_rcon(command)
    })
    .expect("rcon listener");
    let mut client = TcpStream::connect(address).expect("connect");
    client
        .set_read_timeout(Some(Duration::from_secs(10)))
        .expect("timeout");

    // Commands before authentication are rejected with request id -1.
    send(&mut client, 5, SERVERDATA_EXECCOMMAND, "seed");
    let reply = receive(&mut client);
    assert_eq!((reply.request_id, reply.packet_type), (-1, SERVERDATA_AUTH_RESPONSE));

    // Wrong password: auth failure.
    send(&mut client, 6, SERVERDATA_AUTH, "wrong");
    let reply = receive(&mut client);
    assert_eq!((reply.request_id, reply.packet_type), (-1, SERVERDATA_AUTH_RESPONSE));

    // Correct password: echo of the request id, empty payload.
    send(&mut client, 7, SERVERDATA_AUTH, "hunter2");
    let reply = receive(&mut client);
    assert_eq!(
        reply,
        RconPacket {
            request_id: 7,
            packet_type: SERVERDATA_AUTH_RESPONSE,
            payload: String::new()
        }
    );

    // Authenticated command: response type 0 with the buffered output and the same request id.
    send(&mut client, 8, SERVERDATA_EXECCOMMAND, "seed");
    let reply = receive(&mut client);
    assert_eq!(
        reply,
        RconPacket {
            request_id: 8,
            packet_type: SERVERDATA_RESPONSE_VALUE,
            payload: "Seed: 1234".to_string()
        }
    );

    // Unknown request types answer "Unknown request <hex>".
    send(&mut client, 9, 0x2a, "");
    assert_eq!(receive(&mut client).payload, "Unknown request 2a");

    send(&mut client, 10, SERVERDATA_EXECCOMMAND, "stop");
    assert_eq!(receive(&mut client).request_id, 10);
    assert!(runner.halt_requested());
}
