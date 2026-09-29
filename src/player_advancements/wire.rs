//! Network encoding of an advancement definition for
//! `ClientboundUpdateAdvancementsPacket` (`AdvancementHolder.STREAM_CODEC`).

use std::io::{self, Write};

use crate::advancement_system::{AdvancementDefinition, AdvancementDisplay, AdvancementFrame};
use crate::item_catalog::item_protocol_id;
use crate::network::codec::{
    write_identifier, write_optional, write_trusted_component, ComponentJson,
};
use crate::network::play::{AdvancementData, AdvancementHolderData};
use crate::network::varint::write_var_i32;

/// `DisplayInfo` flag bits (`DisplayInfo.STREAM_CODEC`): background present.
const FLAG_HAS_BACKGROUND: i32 = 1;
/// `DisplayInfo` flag bit: show the toast.
const FLAG_SHOW_TOAST: i32 = 2;
/// `DisplayInfo` flag bit: hidden until the parent is done.
const FLAG_HIDDEN: i32 = 4;

/// `AdvancementHolder` as sent to the client: the id plus the `Advancement` stream
/// codec (parent, display, requirements, telemetry flag; rewards and criteria are not
/// networked).
pub fn holder_data(definition: &AdvancementDefinition) -> io::Result<AdvancementHolderData> {
    let display_payload = definition
        .display
        .as_ref()
        .map(display_info_payload)
        .transpose()?;
    Ok(AdvancementHolderData {
        id: definition.id.clone(),
        value: AdvancementData {
            parent: definition.parent.clone(),
            display_payload,
            requirements: definition.requirements.clone(),
            sends_telemetry_event: definition.sends_telemetry_event,
        },
    })
}

/// `DisplayInfo.STREAM_CODEC` encode (the bytes after the optional's presence flag).
fn display_info_payload(display: &AdvancementDisplay) -> io::Result<Vec<u8>> {
    let mut out = Vec::new();
    write_trusted_component(&mut out, &ComponentJson(display.title_json.to_string()))?;
    write_trusted_component(
        &mut out,
        &ComponentJson(display.description_json.to_string()),
    )?;
    write_icon(&mut out, display)?;
    write_var_i32(&mut out, frame_ordinal(display.frame))?;
    out.write_all(&display_flags(display).to_be_bytes())?;
    write_optional(&mut out, display.background.as_ref(), write_identifier)?;
    out.write_all(&display.x.to_be_bytes())?;
    out.write_all(&display.y.to_be_bytes())?;
    Ok(out)
}

/// `ItemStackTemplate.STREAM_CODEC`: item holder id, count and an empty component patch.
// TODO(advancement-icon-components): three vanilla icons (ominous banner and decorated
// pot) carry a `components` patch, which the server has no JSON component codec to
// encode; they are sent as the plain item.
fn write_icon(out: &mut Vec<u8>, display: &AdvancementDisplay) -> io::Result<()> {
    let item = display.icon.to_string();
    let id = item_protocol_id(&item).ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("unknown advancement icon item {item}"),
        )
    })?;
    write_var_i32(out, id)?;
    write_var_i32(out, 1)?;
    write_var_i32(out, 0)?;
    write_var_i32(out, 0)
}

/// `AdvancementType` ordinal (`FriendlyByteBuf.writeEnum`): task, challenge, goal.
fn frame_ordinal(frame: AdvancementFrame) -> i32 {
    match frame {
        AdvancementFrame::Task => 0,
        AdvancementFrame::Challenge => 1,
        AdvancementFrame::Goal => 2,
    }
}

/// The `flags` int of `DisplayInfo.STREAM_CODEC`.
fn display_flags(display: &AdvancementDisplay) -> i32 {
    let mut flags = 0;
    if display.background.is_some() {
        flags |= FLAG_HAS_BACKGROUND;
    }
    if display.show_toast {
        flags |= FLAG_SHOW_TOAST;
    }
    if display.hidden {
        flags |= FLAG_HIDDEN;
    }
    flags
}
