#![allow(dead_code)]

use serde_json::Value;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextWebSocketFrame {
    text: String,
}

pub struct JsonToWebSocketEncoder;
pub struct WebSocketToJsonCodec;

impl TextWebSocketFrame {
    pub fn new(text: String) -> Self {
        Self { text }
    }

    pub fn text(&self) -> &str {
        &self.text
    }
}

impl JsonToWebSocketEncoder {
    pub fn encode(msg: &Value) -> TextWebSocketFrame {
        TextWebSocketFrame::new(msg.to_string())
    }
}

impl WebSocketToJsonCodec {
    pub fn decode(msg: &TextWebSocketFrame) -> serde_json::Result<Value> {
        serde_json::from_str(msg.text())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn json_to_websocket_encoder_writes_compact_json_text_frame() {
        let value = json!({
            "jsonrpc": "2.0",
            "method": "server/status",
            "params": {
                "running": true,
                "players": [ "Steve", "Alex" ]
            }
        });

        let frame = JsonToWebSocketEncoder::encode(&value);

        assert_eq!(
            frame,
            TextWebSocketFrame::new(
                "{\"jsonrpc\":\"2.0\",\"method\":\"server/status\",\"params\":{\"players\":[\"Steve\",\"Alex\"],\"running\":true}}"
                    .to_string()
            )
        );
    }

    #[test]
    fn websocket_to_json_codec_parses_text_frame_json() {
        let frame = TextWebSocketFrame::new(
            "{\"jsonrpc\":\"2.0\",\"id\":7,\"params\":[1,true,null]}".to_string(),
        );

        let parsed = match WebSocketToJsonCodec::decode(&frame) {
            Ok(value) => value,
            Err(error) => panic!("failed to parse websocket JSON frame: {error}"),
        };

        assert_eq!(
            parsed,
            json!({
                "jsonrpc": "2.0",
                "id": 7,
                "params": [1, true, null]
            })
        );
    }

    #[test]
    fn websocket_to_json_codec_surfaces_parse_errors() {
        let frame = TextWebSocketFrame::new("{not-json".to_string());

        assert!(WebSocketToJsonCodec::decode(&frame).is_err());
    }

    #[test]
    #[cfg(vibecraft_has_decompiled_sources)]
    fn websocket_codec_sources_match_java_26_1_2() {
        const JSON_TO_WEBSOCKET_ENCODER: &str = vibecraft_java_source!(
            "/net/minecraft/server/jsonrpc/websocket/JsonToWebSocketEncoder.java"
        );
        const WEBSOCKET_TO_JSON_CODEC: &str = vibecraft_java_source!(
            "/net/minecraft/server/jsonrpc/websocket/WebSocketToJsonCodec.java"
        );

        for sentinel in [
            "public class JsonToWebSocketEncoder extends MessageToMessageEncoder<JsonElement>",
            "protected void encode(final ChannelHandlerContext ctx, final JsonElement msg, final List<Object> out)",
            "out.add(new TextWebSocketFrame(msg.toString()));",
        ] {
            assert!(
                JSON_TO_WEBSOCKET_ENCODER.contains(sentinel),
                "JsonToWebSocketEncoder.java is missing sentinel: {sentinel}"
            );
        }

        for sentinel in [
            "public class WebSocketToJsonCodec extends MessageToMessageDecoder<TextWebSocketFrame>",
            "protected void decode(final ChannelHandlerContext ctx, final TextWebSocketFrame msg, final List<Object> out)",
            "JsonElement json = JsonParser.parseString(msg.text());",
            "out.add(json);",
        ] {
            assert!(
                WEBSOCKET_TO_JSON_CODEC.contains(sentinel),
                "WebSocketToJsonCodec.java is missing sentinel: {sentinel}"
            );
        }
    }
}
