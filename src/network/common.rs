#![allow(dead_code)]

use std::collections::HashMap;
use std::io::{self, Read, Write};

use crate::network::codec::{
    read_collection, read_component, read_enum_index, read_identifier, read_optional, read_string,
    read_trusted_component, read_uuid, write_collection, write_component, write_enum_index,
    write_identifier, write_optional, write_string, write_trusted_component, write_uuid,
    ComponentJson, Uuid,
};
use crate::network::cookie::CookieState;
use crate::network::varint::{read_var_i32, write_var_i32};
use crate::registry::Identifier;

pub const MAX_CLIENTBOUND_CUSTOM_PAYLOAD_SIZE: usize = 1_048_576;
pub const MAX_SERVERBOUND_CUSTOM_PAYLOAD_SIZE: usize = 32_767;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientboundKeepAlivePacket {
    pub id: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerboundKeepAlivePacket {
    pub id: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientboundPingPacket {
    pub id: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerboundPongPacket {
    pub id: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientboundDisconnectPacket {
    pub reason: ComponentJson,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClientboundClearDialogPacket;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientboundCustomReportDetailsPacket {
    pub details: Vec<(String, String)>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServerLinkType {
    BugReport,
    CommunityGuidelines,
    Support,
    Status,
    Feedback,
    Community,
    Website,
    Forums,
    News,
    Announcements,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ServerLinkLabel {
    Known(ServerLinkType),
    Custom(ComponentJson),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerLinkEntry {
    pub label: ServerLinkLabel,
    pub link: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientboundServerLinksPacket {
    pub links: Vec<ServerLinkEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TagNetworkPayload {
    pub tags: Vec<(Identifier, Vec<i32>)>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientboundUpdateTagsPacket {
    pub registries: Vec<(Identifier, TagNetworkPayload)>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientboundShowDialogPacket {
    pub payload: Vec<u8>,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct DialogState {
    current: Option<ClientboundShowDialogPacket>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommonSession {
    pub keep_alive: KeepAliveState,
    pub resource_packs: ResourcePackState,
    pub cookies: CookieState,
    pub dialogs: DialogState,
    pub client_information: ClientInformation,
    pub server_links: Vec<ServerLinkEntry>,
    pub disconnected_reason: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChatVisibility {
    Full,
    System,
    Hidden,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HumanoidArm {
    Left,
    Right,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParticleStatus {
    All,
    Decreased,
    Minimal,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientInformation {
    pub language: String,
    pub view_distance: i8,
    pub chat_visibility: ChatVisibility,
    pub chat_colors: bool,
    pub model_customisation: u8,
    pub main_hand: HumanoidArm,
    pub text_filtering_enabled: bool,
    pub allows_listing: bool,
    pub particle_status: ParticleStatus,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerboundClientInformationPacket {
    pub information: ClientInformation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResourcePackAction {
    SuccessfullyLoaded,
    Declined,
    FailedDownload,
    Accepted,
    Downloaded,
    InvalidUrl,
    FailedReload,
    Discarded,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientboundResourcePackPushPacket {
    pub id: Uuid,
    pub url: String,
    pub hash: String,
    pub required: bool,
    pub prompt: Option<ComponentJson>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientboundResourcePackPopPacket {
    pub id: Option<Uuid>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerboundResourcePackPacket {
    pub id: Uuid,
    pub action: ResourcePackAction,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct ResourcePackState {
    active: HashMap<Uuid, ResourcePackRequest>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourcePackRequest {
    pub url: String,
    pub hash: String,
    pub required: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResourcePackStatus {
    Pending,
    Terminal(ResourcePackAction),
    DisconnectRequiredDeclined,
    UnknownPack,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeepAliveState {
    interval_ms: u64,
    timeout_ms: u64,
    keep_alive_time_ms: u64,
    pending: bool,
    challenge: i64,
    latency_ms: i32,
    disconnected: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KeepAliveTick {
    Idle,
    Send(ClientboundKeepAlivePacket),
    Disconnect,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerboundCustomClickActionPacket {
    pub id: Identifier,
    pub payload: Option<Vec<u8>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CustomPayload {
    Brand(String),
    Unknown {
        channel: Identifier,
        payload: Vec<u8>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientboundCustomPayloadPacket {
    pub payload: CustomPayload,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerboundCustomPayloadPacket {
    pub payload: CustomPayload,
}

impl ClientboundKeepAlivePacket {
    pub fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        Ok(Self {
            id: read_i64_be(reader)?,
        })
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        writer.write_all(&self.id.to_be_bytes())
    }
}

impl ServerboundKeepAlivePacket {
    pub fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        Ok(Self {
            id: read_i64_be(reader)?,
        })
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        writer.write_all(&self.id.to_be_bytes())
    }
}

impl ClientboundPingPacket {
    pub fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        Ok(Self {
            id: read_i32_be(reader)?,
        })
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        writer.write_all(&self.id.to_be_bytes())
    }
}

impl ServerboundPongPacket {
    pub fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        Ok(Self {
            id: read_i32_be(reader)?,
        })
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        writer.write_all(&self.id.to_be_bytes())
    }
}

impl ClientboundDisconnectPacket {
    pub fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        Ok(Self {
            reason: read_trusted_component(reader)?,
        })
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_trusted_component(writer, &self.reason)
    }
}

impl ClientboundClearDialogPacket {
    pub fn read<R: Read>(_reader: &mut R) -> io::Result<Self> {
        Ok(Self)
    }

    pub fn write<W: Write>(&self, _writer: &mut W) -> io::Result<()> {
        Ok(())
    }
}

impl ClientboundCustomReportDetailsPacket {
    pub const MAX_DETAIL_KEY_LENGTH: usize = 128;
    pub const MAX_DETAIL_VALUE_LENGTH: usize = 4096;
    pub const MAX_DETAIL_COUNT: usize = 32;

    pub fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        Ok(Self {
            details: read_collection(reader, |reader| {
                Ok((
                    read_string(reader, Self::MAX_DETAIL_KEY_LENGTH)?,
                    read_string(reader, Self::MAX_DETAIL_VALUE_LENGTH)?,
                ))
            })?,
        })
        .and_then(|packet| {
            if packet.details.len() > Self::MAX_DETAIL_COUNT {
                Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "too many custom report details",
                ))
            } else {
                Ok(packet)
            }
        })
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        if self.details.len() > Self::MAX_DETAIL_COUNT {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "too many custom report details",
            ));
        }
        write_collection(writer, &self.details, |writer, (key, value)| {
            write_string(writer, key, Self::MAX_DETAIL_KEY_LENGTH)?;
            write_string(writer, value, Self::MAX_DETAIL_VALUE_LENGTH)
        })
    }
}

impl ClientboundServerLinksPacket {
    pub fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        Ok(Self {
            links: read_collection(reader, ServerLinkEntry::read)?,
        })
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_collection(writer, &self.links, |writer, link| link.write(writer))
    }
}

impl ServerLinkEntry {
    fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        Ok(Self {
            label: ServerLinkLabel::read(reader)?,
            link: read_string(reader, 32767)?,
        })
    }

    fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        self.label.write(writer)?;
        write_string(writer, &self.link, 32767)
    }
}

impl ServerLinkLabel {
    fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        if read_bool(reader)? {
            Ok(Self::Known(ServerLinkType::from_index(read_enum_index(
                reader, 10,
            )?)?))
        } else {
            Ok(Self::Custom(read_component(reader)?))
        }
    }

    fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        match self {
            Self::Known(link_type) => {
                write_bool(writer, true)?;
                write_enum_index(writer, link_type.index(), 10)
            }
            Self::Custom(component) => {
                write_bool(writer, false)?;
                write_component(writer, component)
            }
        }
    }
}

impl ServerLinkType {
    fn index(self) -> usize {
        match self {
            Self::BugReport => 0,
            Self::CommunityGuidelines => 1,
            Self::Support => 2,
            Self::Status => 3,
            Self::Feedback => 4,
            Self::Community => 5,
            Self::Website => 6,
            Self::Forums => 7,
            Self::News => 8,
            Self::Announcements => 9,
        }
    }

    fn from_index(index: usize) -> io::Result<Self> {
        Ok(match index {
            0 => Self::BugReport,
            1 => Self::CommunityGuidelines,
            2 => Self::Support,
            3 => Self::Status,
            4 => Self::Feedback,
            5 => Self::Community,
            6 => Self::Website,
            7 => Self::Forums,
            8 => Self::News,
            9 => Self::Announcements,
            _ => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "server link type",
                ))
            }
        })
    }
}

impl ClientboundUpdateTagsPacket {
    pub fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        Ok(Self {
            registries: read_collection(reader, |reader| {
                Ok((read_identifier(reader)?, TagNetworkPayload::read(reader)?))
            })?,
        })
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_collection(writer, &self.registries, |writer, (registry, payload)| {
            write_identifier(writer, registry)?;
            payload.write(writer)
        })
    }
}

impl ClientboundShowDialogPacket {
    pub const MAX_CONTEXT_FREE_DIALOG_PAYLOAD_SIZE: usize = 1_048_576;

    pub fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        Ok(Self {
            payload: read_remaining_limited(reader, Self::MAX_CONTEXT_FREE_DIALOG_PAYLOAD_SIZE)?,
        })
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        if self.payload.len() > Self::MAX_CONTEXT_FREE_DIALOG_PAYLOAD_SIZE {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "show dialog payload too large",
            ));
        }
        writer.write_all(&self.payload)
    }
}

impl DialogState {
    pub fn show(&mut self, packet: ClientboundShowDialogPacket) {
        self.current = Some(packet);
    }

    pub fn clear(&mut self, _packet: ClientboundClearDialogPacket) {
        self.current = None;
    }

    pub fn current(&self) -> Option<&ClientboundShowDialogPacket> {
        self.current.as_ref()
    }
}

impl CommonSession {
    pub fn new(now_ms: u64) -> Self {
        Self {
            keep_alive: KeepAliveState::new(now_ms, 0),
            resource_packs: ResourcePackState::default(),
            cookies: CookieState::default(),
            dialogs: DialogState::default(),
            client_information: ClientInformation::default(),
            server_links: Vec::new(),
            disconnected_reason: None,
        }
    }

    pub fn handle_client_information(&mut self, packet: ServerboundClientInformationPacket) {
        self.client_information = packet.information;
    }

    pub fn update_server_links(&mut self, packet: ClientboundServerLinksPacket) {
        self.server_links = packet.links;
    }

    pub fn disconnect(&mut self, reason: impl Into<String>) {
        self.disconnected_reason = Some(reason.into());
    }

    pub fn is_disconnected(&self) -> bool {
        self.disconnected_reason.is_some()
    }
}

impl TagNetworkPayload {
    fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        Ok(Self {
            tags: read_collection(reader, |reader| {
                Ok((read_identifier(reader)?, read_int_id_list(reader)?))
            })?,
        })
    }

    fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_collection(writer, &self.tags, |writer, (tag, ids)| {
            write_identifier(writer, tag)?;
            write_int_id_list(writer, ids)
        })
    }
}

impl Default for ClientInformation {
    fn default() -> Self {
        Self {
            language: "en_us".to_string(),
            view_distance: 2,
            chat_visibility: ChatVisibility::Full,
            chat_colors: true,
            model_customisation: 0,
            main_hand: HumanoidArm::Right,
            text_filtering_enabled: false,
            allows_listing: false,
            particle_status: ParticleStatus::All,
        }
    }
}

impl ClientInformation {
    pub fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        Ok(Self {
            language: read_string(reader, 16)?,
            view_distance: read_i8(reader)?,
            chat_visibility: ChatVisibility::from_index(read_enum_index(reader, 3)?)?,
            chat_colors: read_bool(reader)?,
            model_customisation: read_u8(reader)?,
            main_hand: HumanoidArm::from_index(read_enum_index(reader, 2)?)?,
            text_filtering_enabled: read_bool(reader)?,
            allows_listing: read_bool(reader)?,
            particle_status: ParticleStatus::from_index(read_enum_index(reader, 3)?)?,
        })
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_string(writer, &self.language, 16)?;
        writer.write_all(&self.view_distance.to_be_bytes())?;
        write_enum_index(writer, self.chat_visibility.index(), 3)?;
        write_bool(writer, self.chat_colors)?;
        writer.write_all(&[self.model_customisation])?;
        write_enum_index(writer, self.main_hand.index(), 2)?;
        write_bool(writer, self.text_filtering_enabled)?;
        write_bool(writer, self.allows_listing)?;
        write_enum_index(writer, self.particle_status.index(), 3)
    }
}

impl ServerboundClientInformationPacket {
    pub fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        Ok(Self {
            information: ClientInformation::read(reader)?,
        })
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        self.information.write(writer)
    }
}

impl ClientboundResourcePackPushPacket {
    pub const MAX_HASH_LENGTH: usize = 40;

    pub fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        Ok(Self {
            id: read_uuid(reader)?,
            url: read_string(reader, 32767)?,
            hash: read_string(reader, Self::MAX_HASH_LENGTH)?,
            required: read_bool(reader)?,
            prompt: read_optional(reader, read_trusted_component)?,
        })
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        if self.hash.chars().count() > Self::MAX_HASH_LENGTH {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "resource pack hash too long",
            ));
        }
        write_uuid(writer, self.id)?;
        write_string(writer, &self.url, 32767)?;
        write_string(writer, &self.hash, Self::MAX_HASH_LENGTH)?;
        write_bool(writer, self.required)?;
        write_optional(writer, self.prompt.as_ref(), |writer, prompt| {
            write_trusted_component(writer, prompt)
        })
    }
}

impl ClientboundResourcePackPopPacket {
    pub fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        Ok(Self {
            id: read_optional(reader, read_uuid)?,
        })
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_optional(writer, self.id.as_ref(), |writer, id| {
            write_uuid(writer, *id)
        })
    }
}

impl ServerboundResourcePackPacket {
    pub fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        Ok(Self {
            id: read_uuid(reader)?,
            action: ResourcePackAction::from_index(read_enum_index(reader, 8)?)?,
        })
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_uuid(writer, self.id)?;
        write_enum_index(writer, self.action.index(), 8)
    }
}

impl ResourcePackState {
    pub fn push(&mut self, packet: ClientboundResourcePackPushPacket) {
        self.active.insert(
            packet.id,
            ResourcePackRequest {
                url: packet.url,
                hash: packet.hash,
                required: packet.required,
            },
        );
    }

    pub fn pop(&mut self, packet: ClientboundResourcePackPopPacket) {
        match packet.id {
            Some(id) => {
                self.active.remove(&id);
            }
            None => self.active.clear(),
        }
    }

    pub fn handle_response(&mut self, packet: ServerboundResourcePackPacket) -> ResourcePackStatus {
        let Some(request) = self.active.get(&packet.id) else {
            return ResourcePackStatus::UnknownPack;
        };

        if request.required && packet.action == ResourcePackAction::Declined {
            self.active.remove(&packet.id);
            return ResourcePackStatus::DisconnectRequiredDeclined;
        }

        if packet.action.is_terminal() {
            self.active.remove(&packet.id);
            ResourcePackStatus::Terminal(packet.action)
        } else {
            ResourcePackStatus::Pending
        }
    }

    pub fn contains(&self, id: Uuid) -> bool {
        self.active.contains_key(&id)
    }

    pub fn len(&self) -> usize {
        self.active.len()
    }
}

impl KeepAliveState {
    pub const VANILLA_INTERVAL_MS: u64 = 15_000;

    pub fn new(now_ms: u64, latency_ms: i32) -> Self {
        Self {
            interval_ms: Self::VANILLA_INTERVAL_MS,
            timeout_ms: Self::VANILLA_INTERVAL_MS,
            keep_alive_time_ms: now_ms,
            pending: false,
            challenge: 0,
            latency_ms,
            disconnected: false,
        }
    }

    pub fn tick(&mut self, now_ms: u64, singleplayer_owner: bool) -> KeepAliveTick {
        if self.disconnected
            || singleplayer_owner
            || now_ms < self.keep_alive_time_ms + self.interval_ms
        {
            return KeepAliveTick::Idle;
        }

        if self.pending {
            self.disconnected = true;
            return KeepAliveTick::Disconnect;
        }

        self.pending = true;
        self.keep_alive_time_ms = now_ms;
        self.challenge = now_ms as i64;
        KeepAliveTick::Send(ClientboundKeepAlivePacket { id: self.challenge })
    }

    pub fn handle_response(
        &mut self,
        packet: ServerboundKeepAlivePacket,
        now_ms: u64,
        singleplayer_owner: bool,
    ) -> KeepAliveTick {
        if self.pending && packet.id == self.challenge {
            let elapsed = now_ms
                .saturating_sub(self.keep_alive_time_ms)
                .min(i32::MAX as u64) as i32;
            self.latency_ms = (self.latency_ms * 3 + elapsed) / 4;
            self.pending = false;
            KeepAliveTick::Idle
        } else if singleplayer_owner {
            KeepAliveTick::Idle
        } else {
            self.disconnected = true;
            KeepAliveTick::Disconnect
        }
    }

    pub fn latency_ms(&self) -> i32 {
        self.latency_ms
    }

    pub fn is_pending(&self) -> bool {
        self.pending
    }
}

impl ServerboundCustomClickActionPacket {
    pub const MAX_LENGTH_PREFIXED_PAYLOAD_SIZE: usize = 65_536;

    pub fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        Ok(Self {
            id: read_identifier(reader)?,
            payload: read_optional(reader, |reader| {
                read_length_prefixed_bytes(reader, Self::MAX_LENGTH_PREFIXED_PAYLOAD_SIZE)
            })?,
        })
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_identifier(writer, &self.id)?;
        write_optional(writer, self.payload.as_ref(), |writer, payload| {
            write_length_prefixed_bytes(writer, payload, Self::MAX_LENGTH_PREFIXED_PAYLOAD_SIZE)
        })
    }
}

impl ResourcePackAction {
    pub fn is_terminal(self) -> bool {
        !matches!(self, Self::Accepted | Self::Downloaded)
    }

    fn index(self) -> usize {
        match self {
            Self::SuccessfullyLoaded => 0,
            Self::Declined => 1,
            Self::FailedDownload => 2,
            Self::Accepted => 3,
            Self::Downloaded => 4,
            Self::InvalidUrl => 5,
            Self::FailedReload => 6,
            Self::Discarded => 7,
        }
    }

    fn from_index(index: usize) -> io::Result<Self> {
        Ok(match index {
            0 => Self::SuccessfullyLoaded,
            1 => Self::Declined,
            2 => Self::FailedDownload,
            3 => Self::Accepted,
            4 => Self::Downloaded,
            5 => Self::InvalidUrl,
            6 => Self::FailedReload,
            7 => Self::Discarded,
            _ => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "resource pack action out of range",
                ))
            }
        })
    }
}

impl ChatVisibility {
    fn index(self) -> usize {
        match self {
            Self::Full => 0,
            Self::System => 1,
            Self::Hidden => 2,
        }
    }

    fn from_index(index: usize) -> io::Result<Self> {
        Ok(match index {
            0 => Self::Full,
            1 => Self::System,
            2 => Self::Hidden,
            _ => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "chat visibility",
                ))
            }
        })
    }
}

impl HumanoidArm {
    fn index(self) -> usize {
        match self {
            Self::Left => 0,
            Self::Right => 1,
        }
    }

    fn from_index(index: usize) -> io::Result<Self> {
        Ok(match index {
            0 => Self::Left,
            1 => Self::Right,
            _ => return Err(io::Error::new(io::ErrorKind::InvalidData, "humanoid arm")),
        })
    }
}

impl ParticleStatus {
    fn index(self) -> usize {
        match self {
            Self::All => 0,
            Self::Decreased => 1,
            Self::Minimal => 2,
        }
    }

    fn from_index(index: usize) -> io::Result<Self> {
        Ok(match index {
            0 => Self::All,
            1 => Self::Decreased,
            2 => Self::Minimal,
            _ => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "particle status",
                ))
            }
        })
    }
}

impl ClientboundCustomPayloadPacket {
    pub fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        Ok(Self {
            payload: CustomPayload::read(reader, MAX_CLIENTBOUND_CUSTOM_PAYLOAD_SIZE)?,
        })
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        self.payload
            .write(writer, MAX_CLIENTBOUND_CUSTOM_PAYLOAD_SIZE)
    }
}

impl ServerboundCustomPayloadPacket {
    pub fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        Ok(Self {
            payload: CustomPayload::read(reader, MAX_SERVERBOUND_CUSTOM_PAYLOAD_SIZE)?,
        })
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        self.payload
            .write(writer, MAX_SERVERBOUND_CUSTOM_PAYLOAD_SIZE)
    }
}

impl CustomPayload {
    pub fn channel(&self) -> Identifier {
        match self {
            Self::Brand(_) => brand_channel(),
            Self::Unknown { channel, .. } => channel.clone(),
        }
    }

    fn read<R: Read>(reader: &mut R, max_unknown_payload: usize) -> io::Result<Self> {
        let channel = read_identifier(reader)?;
        if channel == brand_channel() {
            return Ok(Self::Brand(read_string(reader, 32767)?));
        }

        Ok(Self::Unknown {
            channel,
            payload: read_remaining_limited(reader, max_unknown_payload)?,
        })
    }

    fn write<W: Write>(&self, writer: &mut W, max_unknown_payload: usize) -> io::Result<()> {
        match self {
            Self::Brand(brand) => {
                write_identifier(writer, &brand_channel())?;
                write_string(writer, brand, 32767)
            }
            Self::Unknown { channel, payload } => {
                if payload.len() > max_unknown_payload {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidInput,
                        "custom payload too large",
                    ));
                }
                write_identifier(writer, channel)?;
                writer.write_all(payload)
            }
        }
    }
}

fn brand_channel() -> Identifier {
    match Identifier::new("minecraft", "brand") {
        Ok(identifier) => identifier,
        Err(err) => panic!("hard-coded minecraft:brand identifier is invalid: {err}"),
    }
}

fn read_i64_be<R: Read>(reader: &mut R) -> io::Result<i64> {
    let mut bytes = [0u8; 8];
    reader.read_exact(&mut bytes)?;
    Ok(i64::from_be_bytes(bytes))
}

fn read_i32_be<R: Read>(reader: &mut R) -> io::Result<i32> {
    let mut bytes = [0u8; 4];
    reader.read_exact(&mut bytes)?;
    Ok(i32::from_be_bytes(bytes))
}

fn read_i8<R: Read>(reader: &mut R) -> io::Result<i8> {
    let mut byte = [0u8; 1];
    reader.read_exact(&mut byte)?;
    Ok(i8::from_be_bytes(byte))
}

fn read_u8<R: Read>(reader: &mut R) -> io::Result<u8> {
    let mut byte = [0u8; 1];
    reader.read_exact(&mut byte)?;
    Ok(byte[0])
}

fn read_bool<R: Read>(reader: &mut R) -> io::Result<bool> {
    // Java `FriendlyByteBuf.readBoolean` = `readByte() != 0` — any non-zero byte is
    // true (matches the play/status `read_bool` helpers), not just 0/1.
    Ok(read_u8(reader)? != 0)
}

fn write_bool<W: Write>(writer: &mut W, value: bool) -> io::Result<()> {
    writer.write_all(&[u8::from(value)])
}

fn read_length_prefixed_bytes<R: Read>(reader: &mut R, max_size: usize) -> io::Result<Vec<u8>> {
    let length = read_var_i32(reader)?;
    if length < 0 || length as usize > max_size {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "length-prefixed payload too large",
        ));
    }
    let mut bytes = vec![0; length as usize];
    reader.read_exact(&mut bytes)?;
    Ok(bytes)
}

fn write_length_prefixed_bytes<W: Write>(
    writer: &mut W,
    payload: &[u8],
    max_size: usize,
) -> io::Result<()> {
    if payload.len() > max_size {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "length-prefixed payload too large",
        ));
    }
    write_var_i32(writer, payload.len() as i32)?;
    writer.write_all(payload)
}

fn read_int_id_list<R: Read>(reader: &mut R) -> io::Result<Vec<i32>> {
    read_collection(reader, read_var_i32)
}

fn write_int_id_list<W: Write>(writer: &mut W, ids: &[i32]) -> io::Result<()> {
    write_collection(writer, ids, |writer, id| write_var_i32(writer, *id))
}

fn read_remaining_limited<R: Read>(reader: &mut R, max_size: usize) -> io::Result<Vec<u8>> {
    let mut payload = Vec::new();
    reader.take(max_size as u64 + 1).read_to_end(&mut payload)?;
    if payload.len() > max_size {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "custom payload too large",
        ));
    }
    Ok(payload)
}

#[cfg(all(test, vibecraft_has_decompiled_sources))]
mod tests;
