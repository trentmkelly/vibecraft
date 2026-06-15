use super::*;
use crate::command_synchronization::{
    argument_type_bootstrap_order, ArgumentInfoKind, NumericArgumentKind,
};

impl SlotDisplayData {
    const EMPTY_TYPE_ID: i32 = 0;
    const ANY_FUEL_TYPE_ID: i32 = 1;
    const WITH_ANY_POTION_TYPE_ID: i32 = 2;
    const ONLY_WITH_COMPONENT_TYPE_ID: i32 = 3;
    const ITEM_TYPE_ID: i32 = 4;
    const ITEM_STACK_TYPE_ID: i32 = 5;
    const TAG_TYPE_ID: i32 = 6;
    const DYED_TYPE_ID: i32 = 7;
    const SMITHING_TRIM_TYPE_ID: i32 = 8;
    const WITH_REMAINDER_TYPE_ID: i32 = 9;
    const COMPOSITE_TYPE_ID: i32 = 10;

    pub(super) fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        match self {
            Self::Empty => write_var_i32(writer, Self::EMPTY_TYPE_ID),
            Self::AnyFuel => write_var_i32(writer, Self::ANY_FUEL_TYPE_ID),
            Self::WithAnyPotion(display) => {
                write_var_i32(writer, Self::WITH_ANY_POTION_TYPE_ID)?;
                display.write(writer)
            }
            Self::OnlyWithComponent {
                contents,
                component_type_id,
            } => {
                write_var_i32(writer, Self::ONLY_WITH_COMPONENT_TYPE_ID)?;
                contents.write(writer)?;
                write_var_i32(writer, *component_type_id)
            }
            Self::Item { item_id } => {
                write_var_i32(writer, Self::ITEM_TYPE_ID)?;
                write_var_i32(writer, *item_id)
            }
            Self::ItemStack { stack } => {
                write_var_i32(writer, Self::ITEM_STACK_TYPE_ID)?;
                stack.write_template(writer)
            }
            Self::Tag { tag } => {
                write_var_i32(writer, Self::TAG_TYPE_ID)?;
                write_identifier(writer, tag)
            }
            Self::Dyed { dye, target } => {
                write_var_i32(writer, Self::DYED_TYPE_ID)?;
                dye.write(writer)?;
                target.write(writer)
            }
            Self::SmithingTrim {
                base,
                material,
                pattern_id,
            } => {
                write_var_i32(writer, Self::SMITHING_TRIM_TYPE_ID)?;
                base.write(writer)?;
                material.write(writer)?;
                write_var_i32(writer, *pattern_id)
            }
            Self::WithRemainder { input, remainder } => {
                write_var_i32(writer, Self::WITH_REMAINDER_TYPE_ID)?;
                input.write(writer)?;
                remainder.write(writer)
            }
            Self::Composite(contents) => {
                write_var_i32(writer, Self::COMPOSITE_TYPE_ID)?;
                write_collection(writer, contents, |writer, display| display.write(writer))
            }
        }
    }
}

impl RecipeIngredientData {
    pub(super) fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        match self {
            Self::DirectItems(item_ids) => {
                write_var_i32(writer, item_ids.len() as i32 + 1)?;
                for item_id in item_ids {
                    write_var_i32(writer, *item_id)?;
                }
                Ok(())
            }
            Self::Tag(tag) => {
                write_var_i32(writer, 0)?;
                write_identifier(writer, tag)
            }
        }
    }
}

pub(super) fn write_optional_var_i32<W: Write>(
    writer: &mut W,
    value: Option<i32>,
) -> io::Result<()> {
    match value {
        Some(value) => write_var_i32(writer, value + 1),
        None => write_var_i32(writer, 0),
    }
}

impl ClientboundRecipeBookRemovePacket {
    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_collection(
            writer,
            &self.recipe_display_ids,
            |writer, recipe_display_id| write_var_i32(writer, *recipe_display_id),
        )
    }
}

impl RecipeBookTypeSettings {
    pub const CLOSED_UNFILTERED: Self = Self {
        open: false,
        filtering: false,
    };

    pub(super) fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_bool(writer, self.open)?;
        write_bool(writer, self.filtering)
    }
}

impl ClientboundRecipeBookSettingsPacket {
    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        self.crafting.write(writer)?;
        self.furnace.write(writer)?;
        self.blast_furnace.write(writer)?;
        self.smoker.write(writer)
    }
}

impl ClientboundAdvancementsPacket {
    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_bool(writer, self.reset)?;
        write_collection(writer, &self.added, |writer, advancement| {
            advancement.write(writer)
        })?;
        write_collection(writer, &self.removed, write_identifier)?;
        write_collection(writer, &self.progress, |writer, (id, progress)| {
            write_identifier(writer, id)?;
            progress.write(writer)
        })?;
        write_bool(writer, self.show_advancements)
    }
}

impl ClientboundSelectAdvancementsTabPacket {
    pub fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        let packet = Self {
            tab: read_optional(reader, read_identifier)?,
        };
        expect_empty_payload(reader)?;
        Ok(packet)
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_optional(writer, self.tab.as_ref(), write_identifier)
    }
}

impl ClientboundServerDataPacket {
    pub fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        let packet = Self {
            motd: read_trusted_component(reader)?,
            icon_bytes: read_optional(reader, read_byte_array)?,
        };
        expect_empty_payload(reader)?;
        Ok(packet)
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_trusted_component(writer, &self.motd)?;
        write_optional(writer, self.icon_bytes.as_ref(), |writer, bytes| {
            write_byte_array(writer, bytes)
        })
    }
}

fn read_byte_array<R: Read>(reader: &mut R) -> io::Result<Vec<u8>> {
    let length = read_var_i32(reader)?;
    if length < 0 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "negative byte array length",
        ));
    }
    let mut bytes = vec![0; length as usize];
    reader.read_exact(&mut bytes)?;
    Ok(bytes)
}

fn write_byte_array<W: Write>(writer: &mut W, bytes: &[u8]) -> io::Result<()> {
    write_var_i32(writer, bytes.len() as i32)?;
    writer.write_all(bytes)
}

impl ClientboundPlayerInfoUpdatePacket {
    pub fn player_initializing(entries: Vec<PlayerInfoUpdateEntry>) -> Self {
        Self {
            actions: vec![
                PlayerInfoUpdateAction::AddPlayer,
                PlayerInfoUpdateAction::InitializeChat,
                PlayerInfoUpdateAction::UpdateGameMode,
                PlayerInfoUpdateAction::UpdateListed,
                PlayerInfoUpdateAction::UpdateLatency,
                PlayerInfoUpdateAction::UpdateDisplayName,
                PlayerInfoUpdateAction::UpdateListOrder,
                PlayerInfoUpdateAction::UpdateHat,
            ],
            entries,
        }
    }

    pub fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        let mask = read_u8(reader)?;
        let actions = player_info_actions_from_mask(mask);
        let entries = read_collection(reader, |reader| {
            let mut entry = PlayerInfoUpdateEntry {
                profile_id: read_uuid(reader)?,
                profile: None,
                chat_session_payload: None,
                game_mode: 0,
                listed: false,
                latency: 0,
                display_name_payload: None,
                list_order: 0,
                show_hat: false,
            };
            for action in &actions {
                action.read_entry(reader, &mut entry)?;
            }
            Ok(entry)
        })?;
        expect_empty_payload(reader)?;
        Ok(Self { actions, entries })
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        writer.write_all(&[player_info_action_mask(&self.actions)?])?;
        let actions = player_info_actions_in_java_order(&self.actions);
        write_collection(writer, &self.entries, |writer, entry| {
            write_uuid(writer, entry.profile_id)?;
            for action in &actions {
                action.write_entry(writer, entry)?;
            }
            Ok(())
        })
    }
}

impl ClientboundPlayerChatPacket {
    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_var_i32(writer, self.global_index)?;
        write_uuid(writer, self.sender)?;
        write_var_i32(writer, self.index)?;
        write_optional(writer, self.signature.as_ref(), |writer, signature| {
            write_message_signature(writer, signature)
        })?;
        self.body.write(writer)?;
        write_optional(
            writer,
            self.unsigned_content_payload.as_ref(),
            |writer, payload| writer.write_all(payload),
        )?;
        self.filter_mask.write(writer)?;
        self.chat_type.write(writer)
    }
}

impl SignedMessageBodyPacked {
    pub(super) fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_string(writer, &self.content, 256)?;
        write_i64(writer, self.timestamp_epoch_millis)?;
        write_i64(writer, self.salt)?;
        write_collection(writer, &self.last_seen, |writer, signature| {
            signature.write(writer)
        })
    }
}

impl MessageSignaturePackedData {
    pub(super) fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        match self {
            Self::Full(signature) => {
                write_var_i32(writer, 0)?;
                write_message_signature(writer, signature)
            }
            Self::Id(id) => write_var_i32(writer, id + 1),
        }
    }
}

impl FilterMaskData {
    pub(super) fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        match self {
            Self::PassThrough => write_var_i32(writer, 0),
            Self::FullyFiltered => write_var_i32(writer, 1),
            Self::PartiallyFiltered(mask) => {
                write_var_i32(writer, 2)?;
                write_bitset(writer, mask)
            }
        }
    }
}

impl BoundChatTypeData {
    pub(super) fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_var_i32(writer, self.chat_type_id + 1)?;
        writer.write_all(&self.name_payload)?;
        write_optional(
            writer,
            self.target_name_payload.as_ref(),
            |writer, payload| writer.write_all(payload),
        )
    }
}

pub(super) fn write_message_signature<W: Write>(
    writer: &mut W,
    signature: &[u8],
) -> io::Result<()> {
    if signature.len() != 256 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "message signature must be exactly 256 bytes",
        ));
    }
    writer.write_all(signature)
}

impl PlayerInfoUpdateAction {
    pub(super) fn ordinal(self) -> u8 {
        match self {
            Self::AddPlayer => 0,
            Self::InitializeChat => 1,
            Self::UpdateGameMode => 2,
            Self::UpdateListed => 3,
            Self::UpdateLatency => 4,
            Self::UpdateDisplayName => 5,
            Self::UpdateListOrder => 6,
            Self::UpdateHat => 7,
        }
    }

    pub(super) fn from_ordinal(ordinal: u8) -> Self {
        match ordinal {
            0 => Self::AddPlayer,
            1 => Self::InitializeChat,
            2 => Self::UpdateGameMode,
            3 => Self::UpdateListed,
            4 => Self::UpdateLatency,
            5 => Self::UpdateDisplayName,
            6 => Self::UpdateListOrder,
            7 => Self::UpdateHat,
            _ => unreachable!("player-info action mask only exposes 8 bits"),
        }
    }

    pub(super) fn read_entry<R: Read>(
        &self,
        reader: &mut R,
        entry: &mut PlayerInfoUpdateEntry,
    ) -> io::Result<()> {
        match self {
            Self::AddPlayer => entry.profile = Some(PlayerInfoProfile::read(reader)?),
            Self::InitializeChat => {
                entry.chat_session_payload = read_optional(reader, read_chat_session_data_payload)?
            }
            Self::UpdateGameMode => entry.game_mode = read_var_i32(reader)?,
            Self::UpdateListed => entry.listed = read_bool(reader)?,
            Self::UpdateLatency => entry.latency = read_var_i32(reader)?,
            Self::UpdateDisplayName => {
                entry.display_name_payload =
                    read_optional(reader, read_trusted_component_payload)?
            }
            Self::UpdateListOrder => entry.list_order = read_var_i32(reader)?,
            Self::UpdateHat => entry.show_hat = read_bool(reader)?,
        }
        Ok(())
    }

    pub(super) fn write_entry<W: Write>(
        &self,
        writer: &mut W,
        entry: &PlayerInfoUpdateEntry,
    ) -> io::Result<()> {
        match self {
            Self::AddPlayer => entry
                .profile
                .as_ref()
                .ok_or_else(|| {
                    io::Error::new(
                        io::ErrorKind::InvalidInput,
                        "ADD_PLAYER action requires a profile",
                    )
                })?
                .write(writer),
            Self::InitializeChat => write_optional(
                writer,
                entry.chat_session_payload.as_ref(),
                |writer, payload| writer.write_all(payload),
            ),
            Self::UpdateGameMode => write_var_i32(writer, entry.game_mode),
            Self::UpdateListed => write_bool(writer, entry.listed),
            Self::UpdateLatency => write_var_i32(writer, entry.latency),
            Self::UpdateDisplayName => write_optional(
                writer,
                entry.display_name_payload.as_ref(),
                |writer, payload| writer.write_all(payload),
            ),
            Self::UpdateListOrder => write_var_i32(writer, entry.list_order),
            Self::UpdateHat => write_bool(writer, entry.show_hat),
        }
    }
}

impl PlayerInfoProfile {
    pub(super) fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        let name = read_string(reader, 16)?;
        let properties = read_limited_len(reader, 16, "game profile property count")?;
        let mut parsed = Vec::with_capacity(properties);
        for _ in 0..properties {
            parsed.push(GameProfileProperty::read(reader)?);
        }
        Ok(Self {
            name,
            properties: parsed,
        })
    }

    pub(super) fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_string(writer, &self.name, 16)?;
        if self.properties.len() > 16 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "game profile property count exceeds vanilla limit",
            ));
        }
        write_var_i32(writer, self.properties.len() as i32)?;
        for property in &self.properties {
            property.write(writer)?;
        }
        Ok(())
    }
}

impl GameProfileProperty {
    pub(super) fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        Ok(Self {
            name: read_string(reader, 64)?,
            value: read_string(reader, 32767)?,
            signature: read_optional(reader, |reader| read_string(reader, 1024))?,
        })
    }

    pub(super) fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_string(writer, &self.name, 64)?;
        write_string(writer, &self.value, 32767)?;
        write_optional(writer, self.signature.as_ref(), |writer, signature| {
            write_string(writer, signature, 1024)
        })
    }
}

pub(super) fn player_info_action_mask(actions: &[PlayerInfoUpdateAction]) -> io::Result<u8> {
    let mut mask = 0u8;
    for action in actions {
        let bit = 1u8.checked_shl(action.ordinal() as u32).ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                "player info action ordinal exceeds fixed bitset size",
            )
        })?;
        mask |= bit;
    }
    Ok(mask)
}

pub(super) fn player_info_actions_from_mask(mask: u8) -> Vec<PlayerInfoUpdateAction> {
    (0..8)
        .filter(|ordinal| mask & (1 << ordinal) != 0)
        .map(PlayerInfoUpdateAction::from_ordinal)
        .collect()
}

pub(super) fn player_info_actions_in_java_order(
    actions: &[PlayerInfoUpdateAction],
) -> Vec<PlayerInfoUpdateAction> {
    const JAVA_ORDER: [PlayerInfoUpdateAction; 8] = [
        PlayerInfoUpdateAction::AddPlayer,
        PlayerInfoUpdateAction::InitializeChat,
        PlayerInfoUpdateAction::UpdateGameMode,
        PlayerInfoUpdateAction::UpdateListed,
        PlayerInfoUpdateAction::UpdateLatency,
        PlayerInfoUpdateAction::UpdateDisplayName,
        PlayerInfoUpdateAction::UpdateListOrder,
        PlayerInfoUpdateAction::UpdateHat,
    ];
    JAVA_ORDER
        .iter()
        .copied()
        .filter(|action| actions.contains(action))
        .collect()
}

fn read_chat_session_data_payload<R: Read>(reader: &mut R) -> io::Result<Vec<u8>> {
    let session_id = read_uuid(reader)?;
    let expires_at = read_i64(reader)?;
    let public_key = read_byte_array(reader)?;
    let key_signature = read_byte_array(reader)?;
    let mut payload = Vec::new();
    write_uuid(&mut payload, session_id)?;
    write_i64(&mut payload, expires_at)?;
    write_byte_array(&mut payload, &public_key)?;
    write_byte_array(&mut payload, &key_signature)?;
    Ok(payload)
}

fn read_trusted_component_payload<R: Read>(reader: &mut R) -> io::Result<Vec<u8>> {
    let component = read_trusted_component(reader)?;
    let mut payload = Vec::new();
    write_trusted_component(&mut payload, &component)?;
    Ok(payload)
}

impl AdvancementHolderData {
    pub fn minimal(
        id: Identifier,
        parent: Option<Identifier>,
        requirements: Vec<Vec<String>>,
        sends_telemetry_event: bool,
    ) -> Self {
        Self {
            id,
            value: AdvancementData {
                parent,
                display_payload: None,
                requirements,
                sends_telemetry_event,
            },
        }
    }

    pub(super) fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_identifier(writer, &self.id)?;
        self.value.write(writer)
    }
}

impl AdvancementData {
    pub(super) fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_optional(writer, self.parent.as_ref(), write_identifier)?;
        write_optional(writer, self.display_payload.as_ref(), |writer, payload| {
            writer.write_all(payload)
        })?;
        write_collection(writer, &self.requirements, |writer, requirement_group| {
            write_collection(writer, requirement_group, |writer, criterion| {
                write_string(writer, criterion, 32767)
            })
        })?;
        write_bool(writer, self.sends_telemetry_event)
    }
}

impl AdvancementProgressData {
    pub(super) fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_collection(writer, &self.criteria, |writer, (criterion, progress)| {
            write_string(writer, criterion, 32767)?;
            progress.write(writer)
        })
    }
}

impl CriterionProgressData {
    pub(super) fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        match self.obtained_epoch_millis {
            Some(epoch_millis) => {
                write_bool(writer, true)?;
                write_i64(writer, epoch_millis)
            }
            None => write_bool(writer, false),
        }
    }
}

impl ClientboundCommandsPacket {
    pub fn root_only() -> Self {
        Self {
            root_index: 0,
            entries: vec![CommandNodeEntryData {
                stub: CommandNodeStubData::Root,
                executable: false,
                restricted: false,
                redirect: None,
                children: Vec::new(),
            }],
        }
    }

    pub fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        let entries = read_collection(reader, CommandNodeEntryData::read)?;
        let root_index = read_var_i32(reader)?;
        validate_command_node_entries(&entries)?;
        expect_empty_payload(reader)?;
        Ok(Self {
            root_index,
            entries,
        })
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_collection(writer, &self.entries, |writer, entry| entry.write(writer))?;
        write_var_i32(writer, self.root_index)
    }
}

impl CommandNodeEntryData {
    const FLAG_EXECUTABLE: u8 = 4;
    const FLAG_REDIRECT: u8 = 8;
    const FLAG_CUSTOM_SUGGESTIONS: u8 = 16;
    const FLAG_RESTRICTED: u8 = 32;

    pub(super) fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        let flags = read_u8(reader)?;
        let child_count = read_var_i32(reader)?;
        if child_count < 0 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "negative command-node child count",
            ));
        }
        let mut children = Vec::with_capacity(child_count as usize);
        for _ in 0..child_count {
            children.push(read_var_i32(reader)?);
        }

        let redirect = if flags & Self::FLAG_REDIRECT != 0 {
            Some(read_var_i32(reader)?)
        } else {
            None
        };
        let stub = CommandNodeStubData::read(reader, flags)?;
        Ok(Self {
            stub,
            executable: flags & Self::FLAG_EXECUTABLE != 0,
            restricted: flags & Self::FLAG_RESTRICTED != 0,
            redirect,
            children,
        })
    }

    pub(super) fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        let mut flags = self.stub.node_type();
        if self.executable {
            flags |= Self::FLAG_EXECUTABLE;
        }
        if self.redirect.is_some() {
            flags |= Self::FLAG_REDIRECT;
        }
        if self.restricted {
            flags |= Self::FLAG_RESTRICTED;
        }
        if self.stub.has_custom_suggestions() {
            flags |= Self::FLAG_CUSTOM_SUGGESTIONS;
        }
        writer.write_all(&[flags])?;
        write_var_i32(writer, self.children.len() as i32)?;
        for child in &self.children {
            write_var_i32(writer, *child)?;
        }
        if let Some(redirect) = self.redirect {
            write_var_i32(writer, redirect)?;
        }
        self.stub.write(writer)
    }

    fn can_build(&self, unbuilt_nodes: &[bool]) -> bool {
        self.redirect
            .is_none_or(|redirect| !index_is_pending(unbuilt_nodes, redirect))
    }

    fn can_resolve(&self, unresolved_nodes: &[bool]) -> bool {
        self.children
            .iter()
            .all(|child| !index_is_pending(unresolved_nodes, *child))
    }
}

impl CommandNodeStubData {
    const MASK_TYPE: u8 = 3;

    pub(super) fn read<R: Read>(reader: &mut R, flags: u8) -> io::Result<Self> {
        match flags & Self::MASK_TYPE {
            0 => Ok(Self::Root),
            1 => Ok(Self::Literal {
                name: read_string(reader, 32767)?,
            }),
            2 => {
                let name = read_string(reader, 32767)?;
                let parser_type_id = read_var_i32(reader)?;
                let parser_payload = read_command_argument_payload(reader, parser_type_id)?;
                let suggestion_id = if flags & CommandNodeEntryData::FLAG_CUSTOM_SUGGESTIONS != 0 {
                    Some(read_identifier(reader)?)
                } else {
                    None
                };
                Ok(Self::Argument {
                    name,
                    parser_type_id,
                    parser_payload,
                    suggestion_id,
                })
            }
            _ => Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "unknown command node type",
            )),
        }
    }

    pub(super) fn node_type(&self) -> u8 {
        match self {
            Self::Root => 0,
            Self::Literal { .. } => 1,
            Self::Argument { .. } => 2,
        }
    }

    pub(super) fn has_custom_suggestions(&self) -> bool {
        matches!(
            self,
            Self::Argument {
                suggestion_id: Some(_),
                ..
            }
        )
    }

    pub(super) fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        match self {
            Self::Root => Ok(()),
            Self::Literal { name } => write_string(writer, name, 32767),
            Self::Argument {
                name,
                parser_type_id,
                parser_payload,
                suggestion_id,
            } => {
                write_string(writer, name, 32767)?;
                write_var_i32(writer, *parser_type_id)?;
                writer.write_all(parser_payload)?;
                if let Some(suggestion_id) = suggestion_id {
                    write_identifier(writer, suggestion_id)?;
                }
                Ok(())
            }
        }
    }
}

fn validate_command_node_entries(entries: &[CommandNodeEntryData]) -> io::Result<()> {
    validate_command_node_entries_with(entries, CommandNodeEntryData::can_build)?;
    validate_command_node_entries_with(entries, CommandNodeEntryData::can_resolve)
}

fn validate_command_node_entries_with(
    entries: &[CommandNodeEntryData],
    validator: fn(&CommandNodeEntryData, &[bool]) -> bool,
) -> io::Result<()> {
    let mut pending = vec![true; entries.len()];
    let mut pending_count = entries.len();
    while pending_count > 0 {
        let mut removed_any = false;
        for index in 0..entries.len() {
            if pending[index] && validator(&entries[index], &pending) {
                pending[index] = false;
                pending_count -= 1;
                removed_any = true;
            }
        }
        if !removed_any {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Server sent an impossible command tree",
            ));
        }
    }
    Ok(())
}

fn index_is_pending(pending: &[bool], index: i32) -> bool {
    usize::try_from(index)
        .ok()
        .and_then(|index| pending.get(index))
        .copied()
        .unwrap_or(false)
}

fn read_command_argument_payload<R: Read>(reader: &mut R, parser_type_id: i32) -> io::Result<Vec<u8>> {
    let registration = usize::try_from(parser_type_id)
        .ok()
        .and_then(|index| argument_type_bootstrap_order().get(index))
        .ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("unknown command argument type id {parser_type_id}"),
            )
        })?;

    match registration.info {
        ArgumentInfoKind::Singleton { .. } => Ok(Vec::new()),
        ArgumentInfoKind::Numeric(kind) => read_numeric_argument_payload(reader, kind),
        ArgumentInfoKind::String => read_string_argument_payload(reader),
        ArgumentInfoKind::Entity | ArgumentInfoKind::ScoreHolder => read_fixed_payload(reader, 1),
        ArgumentInfoKind::Time => read_fixed_payload(reader, 4),
        ArgumentInfoKind::RegistryBacked => read_registry_key_argument_payload(reader),
    }
}

fn read_numeric_argument_payload<R: Read>(
    reader: &mut R,
    kind: NumericArgumentKind,
) -> io::Result<Vec<u8>> {
    let mut payload = read_fixed_payload(reader, 1)?;
    let flags = payload[0];
    let value_width = match kind {
        NumericArgumentKind::Float | NumericArgumentKind::Integer => 4,
        NumericArgumentKind::Double | NumericArgumentKind::Long => 8,
    };
    if flags & 1 != 0 {
        payload.extend(read_fixed_payload(reader, value_width)?);
    }
    if flags & 2 != 0 {
        payload.extend(read_fixed_payload(reader, value_width)?);
    }
    Ok(payload)
}

fn read_string_argument_payload<R: Read>(reader: &mut R) -> io::Result<Vec<u8>> {
    let ordinal = read_var_i32(reader)?;
    if !(0..=2).contains(&ordinal) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("Invalid StringArgumentType ordinal {ordinal}"),
        ));
    }
    let mut payload = Vec::new();
    write_var_i32(&mut payload, ordinal)?;
    Ok(payload)
}

fn read_registry_key_argument_payload<R: Read>(reader: &mut R) -> io::Result<Vec<u8>> {
    let key = read_identifier(reader)?;
    let mut payload = Vec::new();
    write_identifier(&mut payload, &key)?;
    Ok(payload)
}

fn read_fixed_payload<R: Read>(reader: &mut R, len: usize) -> io::Result<Vec<u8>> {
    let mut payload = vec![0; len];
    reader.read_exact(&mut payload)?;
    Ok(payload)
}

impl ClientboundCommandSuggestionsPacket {
    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_var_i32(writer, self.transaction_id)?;
        write_var_i32(writer, self.start)?;
        write_var_i32(writer, self.length)?;
        write_var_i32(writer, self.suggestions.len() as i32)?;
        for suggestion in &self.suggestions {
            write_string(writer, &suggestion.text, 32767)?;
            match &suggestion.tooltip {
                Some(tooltip) => {
                    write_bool(writer, true)?;
                    write_network_tag(writer, tooltip)?;
                }
                None => write_bool(writer, false)?,
            }
        }
        Ok(())
    }
}

impl ClientboundDebugSamplePacket {
    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_var_i32(writer, self.sample.len() as i32)?;
        for value in &self.sample {
            write_i64(writer, *value)?;
        }
        write_enum_index(
            writer,
            self.sample_type as usize,
            RemoteDebugSampleType::COUNT,
        )
    }
}

impl RemoteDebugSampleType {
    const COUNT: usize = 1;
}

impl ClientboundStartConfigurationPacket {
    pub fn write<W: Write>(&self, _writer: &mut W) -> io::Result<()> {
        Ok(())
    }
}

impl ClientboundStopSoundPacket {
    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        let flags =
            (if self.source.is_some() { 1 } else { 0 }) | (if self.name.is_some() { 2 } else { 0 });
        writer.write_all(&[flags])?;
        if let Some(source) = self.source {
            write_enum_index(writer, source as usize, SoundSource::COUNT)?;
        }
        if let Some(name) = &self.name {
            write_identifier(writer, name)?;
        }
        Ok(())
    }
}

impl SoundSource {
    const COUNT: usize = 11;
}

impl MobEffectFlags {
    pub const AMBIENT: Self = Self(1);
    pub const VISIBLE: Self = Self(2);
    pub const SHOW_ICON: Self = Self(4);
    pub const BLEND: Self = Self(8);

    pub fn from_parts(ambient: bool, visible: bool, show_icon: bool, blend: bool) -> Self {
        Self(
            (if ambient { Self::AMBIENT.0 } else { 0 })
                | (if visible { Self::VISIBLE.0 } else { 0 })
                | (if show_icon { Self::SHOW_ICON.0 } else { 0 })
                | (if blend { Self::BLEND.0 } else { 0 }),
        )
    }
}

impl ClientboundRemoveMobEffectPacket {
    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_var_i32(writer, self.entity_id)?;
        write_var_i32(writer, self.effect_id)
    }
}

impl ClientboundUpdateMobEffectPacket {
    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_var_i32(writer, self.entity_id)?;
        write_var_i32(writer, self.effect_id)?;
        write_var_i32(writer, self.amplifier)?;
        write_var_i32(writer, self.duration_ticks)?;
        writer.write_all(&[self.flags.0])
    }
}

impl ClientboundPlayerLookAtPacket {
    pub fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        let from_anchor = EntityAnchor::from_index(read_enum_index(reader, EntityAnchor::COUNT)?)?;
        let x = read_f64(reader)?;
        let y = read_f64(reader)?;
        let z = read_f64(reader)?;
        let target_entity = if read_bool(reader)? {
            Some((
                read_var_i32(reader)?,
                EntityAnchor::from_index(read_enum_index(reader, EntityAnchor::COUNT)?)?,
            ))
        } else {
            None
        };
        expect_empty_payload(reader)?;
        Ok(Self {
            from_anchor,
            x,
            y,
            z,
            target_entity,
        })
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_enum_index(writer, self.from_anchor as usize, EntityAnchor::COUNT)?;
        write_f64(writer, self.x)?;
        write_f64(writer, self.y)?;
        write_f64(writer, self.z)?;
        match self.target_entity {
            Some((entity_id, to_anchor)) => {
                write_bool(writer, true)?;
                write_var_i32(writer, entity_id)?;
                write_enum_index(writer, to_anchor as usize, EntityAnchor::COUNT)
            }
            None => write_bool(writer, false),
        }
    }
}

impl ClientboundSetTitleTextPacket {
    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_network_tag(writer, &self.text)
    }
}

impl ClientboundSetSubtitleTextPacket {
    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_network_tag(writer, &self.text)
    }
}

impl ClientboundSetActionBarTextPacket {
    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_network_tag(writer, &self.text)
    }
}

impl ClientboundSystemChatPacket {
    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_network_tag(writer, &self.content)?;
        write_bool(writer, self.overlay)
    }
}

impl ClientboundDisguisedChatPacket {
    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_network_tag(writer, &self.message)?;
        self.chat_type.write(writer)
    }
}

impl ClientboundCustomChatCompletionsPacket {
    pub fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        let action = CustomChatCompletionsAction::from_index(read_enum_index(reader, 3)?)?;
        let entry_count = read_limited_len(reader, 65_536, "custom chat completion entry count")?;
        let mut entries = Vec::with_capacity(entry_count);
        for _ in 0..entry_count {
            entries.push(read_string(reader, 32767)?);
        }
        expect_empty_payload(reader)?;
        Ok(Self { action, entries })
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_enum_index(writer, self.action.index(), 3)?;
        write_var_i32(writer, self.entries.len() as i32)?;
        for entry in &self.entries {
            write_string(writer, entry, 32767)?;
        }
        Ok(())
    }
}

impl CustomChatCompletionsAction {
    fn from_index(index: usize) -> io::Result<Self> {
        match index {
            0 => Ok(Self::Add),
            1 => Ok(Self::Remove),
            2 => Ok(Self::Set),
            _ => Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "invalid custom chat completion action",
            )),
        }
    }

    fn index(self) -> usize {
        match self {
            Self::Add => 0,
            Self::Remove => 1,
            Self::Set => 2,
        }
    }
}

impl ChatTypeBound {
    pub(super) fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_var_i32(writer, self.chat_type_id + 1)?;
        write_network_tag(writer, &self.name)?;
        write_optional(writer, self.target_name.as_ref(), |writer, target_name| {
            write_network_tag(writer, target_name)
        })
    }
}

impl ClientboundTabListPacket {
    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_network_tag(writer, &self.header)?;
        write_network_tag(writer, &self.footer)
    }
}

impl EntityAnchor {
    const COUNT: usize = 2;

    fn from_index(index: usize) -> io::Result<Self> {
        match index {
            0 => Ok(Self::Feet),
            1 => Ok(Self::Eyes),
            _ => Err(invalid_data("invalid entity anchor")),
        }
    }
}

impl ClientboundResetScorePacket {
    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_string(writer, &self.owner, 32767)?;
        match &self.objective_name {
            Some(objective_name) => {
                write_bool(writer, true)?;
                write_string(writer, objective_name, 32767)
            }
            None => write_bool(writer, false),
        }
    }
}

impl ClientboundSetDisplayObjectivePacket {
    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_var_i32(writer, self.slot)?;
        write_string(writer, &self.objective_name, 32767)
    }
}

impl ClientboundSetObjectivePacket {
    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_string(writer, &self.objective_name, 32767)?;
        match &self.method {
            ObjectiveMethod::Add {
                display_name,
                render_type,
                number_format,
            } => {
                writer.write_all(&[0])?;
                write_objective_payload(writer, display_name, *render_type, number_format)
            }
            ObjectiveMethod::Remove => writer.write_all(&[1]),
            ObjectiveMethod::Change {
                display_name,
                render_type,
                number_format,
            } => {
                writer.write_all(&[2])?;
                write_objective_payload(writer, display_name, *render_type, number_format)
            }
        }
    }
}

pub(super) fn write_objective_payload<W: Write>(
    writer: &mut W,
    display_name: &Tag,
    render_type: ObjectiveRenderType,
    number_format: &Option<NumberFormat>,
) -> io::Result<()> {
    write_network_tag(writer, display_name)?;
    write_var_i32(writer, render_type as i32)?;
    write_optional_number_format(writer, number_format.as_ref())
}

impl ClientboundSetScorePacket {
    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_string(writer, &self.owner, 32767)?;
        write_string(writer, &self.objective_name, 32767)?;
        write_var_i32(writer, self.score)?;
        write_optional(writer, self.display.as_ref(), |writer, display| {
            write_network_tag(writer, display)
        })?;
        write_optional_number_format(writer, self.number_format.as_ref())
    }
}

pub(super) fn write_optional_number_format<W: Write>(
    writer: &mut W,
    number_format: Option<&NumberFormat>,
) -> io::Result<()> {
    write_optional(writer, number_format, |writer, number_format| {
        number_format.write(writer)
    })
}
