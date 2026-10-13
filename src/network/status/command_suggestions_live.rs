//! Live command completion responses, including provider tooltip components.

use super::*;

/// Java `ServerGamePacketListenerImpl.handleCustomCommandSuggestions`: parses the
/// requested command against the dispatcher as the player's source and answers
/// with `CommandDispatcher.getCompletionSuggestions` (at most 1000 entries).
pub fn write_command_suggestions_response<W: Write, R: Read>(
    stream: &mut W,
    compression: CompressionState,
    input: &mut R,
    permission_level: u8,
) -> io::Result<()> {
    let packet = ServerboundCommandSuggestionPacket::read(input)?;
    let suggestions = crate::brigadier::suggest::command_suggestions(
        crate::brigadier::CommandGraph::served(),
        permission_level,
        &packet.command,
    );

    write_framed_packet_with_compression(
        stream,
        compression,
        CLIENTBOUND_COMMAND_SUGGESTIONS_PACKET_ID,
        |payload| {
            // Java preserves Suggestion.getTooltip through ComponentUtils.fromMessage.
            // Registered entity suggestions carry EntityType's translated description.
            crate::network::play::ClientboundCommandSuggestionsPacket {
                transaction_id: packet.id,
                start: suggestions.start as i32,
                length: (suggestions.end - suggestions.start) as i32,
                suggestions: suggestions
                    .texts
                    .iter()
                    .zip(&suggestions.tooltips)
                    .map(
                        |(text, tooltip)| crate::network::play::CommandSuggestionEntry {
                            text: text.clone(),
                            tooltip: tooltip.as_ref().map(|key| {
                                Tag::Compound(vec![(
                                    "translate".to_owned(),
                                    Tag::String(key.clone()),
                                )])
                            }),
                        },
                    )
                    .collect(),
            }
            .write(payload)
        },
    )
}
