#![allow(dead_code)]

use crate::presentation_data::{DamageTypeDef, FallVariant};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageKeyFamily {
    Chat,
    Command,
    Brigadier,
    Disconnect,
    Death,
    Sleep,
    ResourcePack,
    ServerStatus,
    RegistryDescription,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MessageKeyDef {
    pub key: &'static str,
    pub family: MessageKeyFamily,
    pub args: &'static [&'static str],
}

// Representative subset of server-facing localization keys used by protocol and gameplay paths.
mod catalog_data_a;
mod catalog_data_b;

use std::sync::LazyLock;

pub static LOCALIZATION_KEYS: LazyLock<&'static [MessageKeyDef]> = LazyLock::new(|| {
    let combined: Vec<MessageKeyDef> = catalog_data_a::ENTRIES
        .iter()
        .chain(catalog_data_b::ENTRIES.iter())
        .copied()
        .collect();
    Box::leak(combined.into_boxed_slice())
});

pub fn find_key(key_name: &str) -> Option<&'static MessageKeyDef> {
    LOCALIZATION_KEYS.iter().find(|entry| entry.key == key_name)
}

pub fn keys_in_family(family: MessageKeyFamily) -> Vec<&'static str> {
    LOCALIZATION_KEYS
        .iter()
        .filter(|entry| entry.family == family)
        .map(|entry| entry.key)
        .collect()
}

pub fn emitted_disconnect_key(reason: DisconnectReason) -> &'static str {
    match reason {
        DisconnectReason::Kicked => "multiplayer.disconnect.kicked",
        DisconnectReason::Banned => "multiplayer.disconnect.banned",
        DisconnectReason::NotWhitelisted => "multiplayer.disconnect.not_whitelisted",
        DisconnectReason::ServerFull => "multiplayer.disconnect.server_full",
        DisconnectReason::IdleTimeout => "multiplayer.disconnect.idling",
        DisconnectReason::ChatValidationFailed => "multiplayer.disconnect.chat_validation_failed",
        DisconnectReason::ExceededPacketRate => "disconnect.exceeded_packet_rate",
        DisconnectReason::RequiredResourcePackDeclined => {
            "multiplayer.requiredTexturePrompt.disconnect"
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DisconnectReason {
    Kicked,
    Banned,
    NotWhitelisted,
    ServerFull,
    IdleTimeout,
    ChatValidationFailed,
    ExceededPacketRate,
    RequiredResourcePackDeclined,
}

pub fn emitted_death_key(
    damage_type: DamageTypeDef,
    has_attacker: bool,
    has_item: bool,
    fall_variant: Option<FallVariant>,
) -> String {
    match fall_variant {
        Some(variant) => damage_type
            .death_message_key(has_attacker, has_item, Some(variant))
            .to_string(),
        None => damage_type.typed_death_message_key(has_attacker, has_item),
    }
}

pub fn assert_known_emitted_key(key_name: &str) -> Result<&'static MessageKeyDef, String> {
    find_key(key_name).ok_or_else(|| format!("unknown emitted localization key: {key_name}"))
}

const fn key(
    key: &'static str,
    family: MessageKeyFamily,
    args: &'static [&'static str],
) -> MessageKeyDef {
    MessageKeyDef { key, family, args }
}

#[cfg(test)]
mod tests;
