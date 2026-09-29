#!/usr/bin/env python3
"""Captures the configuration-phase registry sync of the official 26.1.2 server.

The registry pipeline tests (`src/registry_pipeline/tests/official_transcript.rs`)
compare VibeCraft's generated `registry_data` and `update_tags` packets against
the fixtures this script records from an unmodified vanilla `server.jar`.

Usage:
    1. Start the official server offline, keeping stdin open:
         (sleep 600 | java -jar "$VIBECRAFT_OFFICIAL_SERVER_JAR" --nogui)   # online-mode=false, server-port=25599
    2. python3 tools/capture_official_registry_sync.py OUT_DIR

The client answers `select_known_packs` with an empty list so the server sends the
full contents of every registry entry. Outputs:
    registry_data_full.bin  - repeated (VarInt length, registry_data packet body)
    tags.bin                - the update_tags packet body
"""
import socket
import struct
import sys
import uuid
import zlib

PORT = 25599
PROTOCOL = 775  # 26.1.2


def varint(value):
    value &= 0xFFFFFFFF
    out = b""
    while True:
        byte = value & 0x7F
        value >>= 7
        if value:
            out += bytes([byte | 0x80])
        else:
            return out + bytes([byte])


def read_varint(data, index):
    value = shift = 0
    while True:
        byte = data[index]
        index += 1
        value |= (byte & 0x7F) << shift
        if not byte & 0x80:
            return value, index
        shift += 7


def mc_string(text):
    raw = text.encode()
    return varint(len(raw)) + raw


class Connection:
    def __init__(self):
        self.sock = socket.create_connection(("127.0.0.1", PORT))
        self.compression = None

    def _read_exact(self, count):
        buffer = b""
        while len(buffer) < count:
            chunk = self.sock.recv(count - len(buffer))
            if not chunk:
                raise EOFError("server closed the connection")
            buffer += chunk
        return buffer

    def send(self, packet_id, body=b""):
        data = varint(packet_id) + body
        if self.compression is not None:
            data = varint(0) + data
        self.sock.sendall(varint(len(data)) + data)

    def receive(self):
        length, shift, value = 0, 0, 0
        while True:
            byte = self._read_exact(1)[0]
            value |= (byte & 0x7F) << shift
            if not byte & 0x80:
                break
            shift += 7
        data = self._read_exact(value)
        if self.compression is not None:
            uncompressed, index = read_varint(data, 0)
            data = data[index:] if uncompressed == 0 else zlib.decompress(data[index:])
        packet_id, index = read_varint(data, 0)
        return packet_id, data[index:]


def main(out_dir):
    conn = Connection()
    conn.send(0, varint(PROTOCOL) + mc_string("localhost") + struct.pack(">H", PORT) + varint(2))
    conn.send(0, mc_string("OracleBot") + uuid.uuid3(uuid.NAMESPACE_DNS, "OracleBot").bytes)
    while True:
        packet_id, body = conn.receive()
        if packet_id == 3:  # set compression
            conn.compression, _ = read_varint(body, 0)
        elif packet_id == 2:  # login finished
            break
        elif packet_id == 0:
            raise SystemExit(f"login disconnect: {body!r}")
    conn.send(3)  # login acknowledged

    registry_packets, tags = [], None
    while True:
        packet_id, body = conn.receive()
        if packet_id == 14:  # select known packs: answer with none
            conn.send(7, varint(0))
        elif packet_id == 7:
            registry_packets.append(body)
        elif packet_id == 13:
            tags = body
        elif packet_id == 3:  # finish configuration
            conn.send(3)
            break
    with open(f"{out_dir}/registry_data_full.bin", "wb") as out:
        out.write(b"".join(varint(len(body)) + body for body in registry_packets))
    with open(f"{out_dir}/tags.bin", "wb") as out:
        out.write(tags)
    print(f"captured {len(registry_packets)} registries")


if __name__ == "__main__":
    main(sys.argv[1] if len(sys.argv) > 1 else ".")
