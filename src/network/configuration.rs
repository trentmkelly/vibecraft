#![allow(dead_code)]

use std::io::{self, Read, Write};

use crate::network::codec::{
    read_collection, read_identifier, read_optional, read_string, write_collection,
    write_identifier, write_optional, write_string,
};
use crate::registry::{Identifier, Registry};
use crate::storage::nbt::Tag;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClientboundFinishConfigurationPacket;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ServerboundFinishConfigurationPacket;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClientboundResetChatPacket;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ServerboundAcceptCodeOfConductPacket;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientboundCodeOfConductPacket {
    pub code_of_conduct: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientboundUpdateEnabledFeaturesPacket {
    pub features: Vec<Identifier>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KnownPack {
    pub namespace: String,
    pub id: String,
    pub version: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientboundSelectKnownPacks {
    pub known_packs: Vec<KnownPack>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerboundSelectKnownPacks {
    pub known_packs: Vec<KnownPack>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PackedRegistryEntry {
    pub id: Identifier,
    pub data: Option<Tag>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClientboundRegistryDataPacket {
    pub registry: Identifier,
    pub entries: Vec<PackedRegistryEntry>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ConfigurationSession {
    pub state: ConfigurationState,
    pub registries: Vec<ClientboundRegistryDataPacket>,
    pub enabled_features: Vec<Identifier>,
    pub offered_known_packs: Vec<KnownPack>,
    pub selected_known_packs: Vec<KnownPack>,
    pub code_of_conduct: Option<String>,
    pub code_of_conduct_accepted: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfigurationState {
    Building,
    AwaitingKnownPacks,
    AwaitingCodeOfConduct,
    ReadyToFinish,
    Finished,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ConfigurationInstruction {
    RegistryData(ClientboundRegistryDataPacket),
    EnabledFeatures(ClientboundUpdateEnabledFeaturesPacket),
    SelectKnownPacks(ClientboundSelectKnownPacks),
    CodeOfConduct(ClientboundCodeOfConductPacket),
    Finish(ClientboundFinishConfigurationPacket),
}

macro_rules! unit_packet {
    ($ty:ty) => {
        impl $ty {
            pub fn read<R: Read>(_reader: &mut R) -> io::Result<Self> {
                Ok(Self)
            }

            pub fn write<W: Write>(&self, _writer: &mut W) -> io::Result<()> {
                Ok(())
            }
        }
    };
}

unit_packet!(ClientboundFinishConfigurationPacket);
unit_packet!(ServerboundFinishConfigurationPacket);
unit_packet!(ClientboundResetChatPacket);
unit_packet!(ServerboundAcceptCodeOfConductPacket);

impl ClientboundCodeOfConductPacket {
    pub fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        Ok(Self {
            code_of_conduct: read_string(reader, 32767)?,
        })
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_string(writer, &self.code_of_conduct, 32767)
    }
}

impl ClientboundUpdateEnabledFeaturesPacket {
    pub fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        Ok(Self {
            features: read_collection(reader, read_identifier)?,
        })
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_collection(writer, &self.features, |writer, feature| {
            write_identifier(writer, feature)
        })
    }
}

impl KnownPack {
    pub fn vanilla(id: impl Into<String>, version: impl Into<String>) -> Self {
        Self {
            namespace: "minecraft".to_string(),
            id: id.into(),
            version: version.into(),
        }
    }

    pub fn is_vanilla(&self) -> bool {
        self.namespace == "minecraft"
    }

    fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        Ok(Self {
            namespace: read_string(reader, 32767)?,
            id: read_string(reader, 32767)?,
            version: read_string(reader, 32767)?,
        })
    }

    fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_string(writer, &self.namespace, 32767)?;
        write_string(writer, &self.id, 32767)?;
        write_string(writer, &self.version, 32767)
    }
}

impl ClientboundSelectKnownPacks {
    pub fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        Ok(Self {
            known_packs: read_collection(reader, KnownPack::read)?,
        })
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_collection(writer, &self.known_packs, |writer, pack| pack.write(writer))
    }
}

impl ServerboundSelectKnownPacks {
    pub const MAX_KNOWN_PACKS: usize = 64;

    pub fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        let known_packs = read_collection(reader, KnownPack::read)?;
        if known_packs.len() > Self::MAX_KNOWN_PACKS {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "too many selected known packs",
            ));
        }
        Ok(Self { known_packs })
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        if self.known_packs.len() > Self::MAX_KNOWN_PACKS {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "too many selected known packs",
            ));
        }
        write_collection(writer, &self.known_packs, |writer, pack| pack.write(writer))
    }
}

impl ClientboundRegistryDataPacket {
    pub fn from_registry<T>(
        registry: &Registry<T>,
        mut encode: impl FnMut(&T) -> Option<Tag>,
    ) -> Self {
        Self {
            registry: registry.registry_id().clone(),
            entries: registry
                .iter()
                .map(|entry| PackedRegistryEntry {
                    id: entry.key().location().clone(),
                    data: encode(entry.value()),
                })
                .collect(),
        }
    }

    pub fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        Ok(Self {
            registry: read_identifier(reader)?,
            entries: read_collection(reader, PackedRegistryEntry::read)?,
        })
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_identifier(writer, &self.registry)?;
        write_collection(writer, &self.entries, |writer, entry| entry.write(writer))
    }
}

impl PackedRegistryEntry {
    fn read<R: Read>(reader: &mut R) -> io::Result<Self> {
        Ok(Self {
            id: read_identifier(reader)?,
            data: read_optional(reader, read_any_tag)?,
        })
    }

    fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_identifier(writer, &self.id)?;
        write_optional(writer, self.data.as_ref(), |writer, tag| {
            write_any_tag(writer, tag)
        })
    }
}

impl ConfigurationSession {
    pub fn new(enabled_features: Vec<Identifier>) -> Self {
        Self {
            state: ConfigurationState::Building,
            registries: Vec::new(),
            enabled_features,
            offered_known_packs: Vec::new(),
            selected_known_packs: Vec::new(),
            code_of_conduct: None,
            code_of_conduct_accepted: false,
        }
    }

    pub fn add_registry_data(&mut self, packet: ClientboundRegistryDataPacket) {
        self.registries.push(packet);
    }

    pub fn offer_known_packs(&mut self, known_packs: Vec<KnownPack>) {
        self.offered_known_packs = known_packs;
    }

    pub fn require_code_of_conduct(&mut self, code_of_conduct: String) {
        self.code_of_conduct = Some(code_of_conduct);
    }

    pub fn start(&mut self) -> Vec<ConfigurationInstruction> {
        let mut instructions = Vec::new();
        instructions.extend(
            self.registries
                .clone()
                .into_iter()
                .map(ConfigurationInstruction::RegistryData),
        );
        instructions.push(ConfigurationInstruction::EnabledFeatures(
            ClientboundUpdateEnabledFeaturesPacket {
                features: self.enabled_features.clone(),
            },
        ));

        if !self.offered_known_packs.is_empty() {
            self.state = ConfigurationState::AwaitingKnownPacks;
            instructions.push(ConfigurationInstruction::SelectKnownPacks(
                ClientboundSelectKnownPacks {
                    known_packs: self.offered_known_packs.clone(),
                },
            ));
        } else if let Some(code_of_conduct) = &self.code_of_conduct {
            self.state = ConfigurationState::AwaitingCodeOfConduct;
            instructions.push(ConfigurationInstruction::CodeOfConduct(
                ClientboundCodeOfConductPacket {
                    code_of_conduct: code_of_conduct.clone(),
                },
            ));
        } else {
            self.state = ConfigurationState::ReadyToFinish;
            instructions.push(ConfigurationInstruction::Finish(
                ClientboundFinishConfigurationPacket,
            ));
        }

        instructions
    }

    pub fn select_known_packs(
        &mut self,
        packet: ServerboundSelectKnownPacks,
    ) -> io::Result<Vec<ConfigurationInstruction>> {
        if self.state != ConfigurationState::AwaitingKnownPacks {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "unexpected known pack selection",
            ));
        }

        for selected in &packet.known_packs {
            if !self.offered_known_packs.contains(selected) {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "selected unknown pack",
                ));
            }
        }

        self.selected_known_packs = packet.known_packs;
        if let Some(code_of_conduct) = &self.code_of_conduct {
            self.state = ConfigurationState::AwaitingCodeOfConduct;
            Ok(vec![ConfigurationInstruction::CodeOfConduct(
                ClientboundCodeOfConductPacket {
                    code_of_conduct: code_of_conduct.clone(),
                },
            )])
        } else {
            self.state = ConfigurationState::ReadyToFinish;
            Ok(vec![ConfigurationInstruction::Finish(
                ClientboundFinishConfigurationPacket,
            )])
        }
    }

    pub fn accept_code_of_conduct(
        &mut self,
        _packet: ServerboundAcceptCodeOfConductPacket,
    ) -> io::Result<ClientboundFinishConfigurationPacket> {
        if self.state != ConfigurationState::AwaitingCodeOfConduct {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "unexpected code of conduct acceptance",
            ));
        }

        self.code_of_conduct_accepted = true;
        self.state = ConfigurationState::ReadyToFinish;
        Ok(ClientboundFinishConfigurationPacket)
    }

    pub fn finish(&mut self, _packet: ServerboundFinishConfigurationPacket) {
        self.state = ConfigurationState::Finished;
    }
}

fn read_any_tag<R: Read>(reader: &mut R) -> io::Result<Tag> {
    let mut id = [0u8; 1];
    reader.read_exact(&mut id)?;
    Tag::read_payload(id[0], reader)
}

fn write_any_tag<W: Write>(writer: &mut W, tag: &Tag) -> io::Result<()> {
    writer.write_all(&[tag.id()])?;
    tag.write_payload(writer)
}

#[cfg(test)]
mod tests {
    use super::{
        ClientboundCodeOfConductPacket, ClientboundFinishConfigurationPacket,
        ClientboundRegistryDataPacket, ClientboundResetChatPacket, ClientboundSelectKnownPacks,
        ClientboundUpdateEnabledFeaturesPacket, ConfigurationInstruction, ConfigurationSession,
        ConfigurationState, KnownPack, PackedRegistryEntry, ServerboundAcceptCodeOfConductPacket,
        ServerboundFinishConfigurationPacket, ServerboundSelectKnownPacks,
    };
    use crate::registry::{Identifier, Lifecycle, Registry};
    use crate::storage::nbt::Tag;
    use std::io::Cursor;

    #[test]
    fn unit_configuration_packets_are_empty() {
        let mut bytes = Vec::new();
        ClientboundFinishConfigurationPacket
            .write(&mut bytes)
            .unwrap();
        ServerboundFinishConfigurationPacket
            .write(&mut bytes)
            .unwrap();
        ClientboundResetChatPacket.write(&mut bytes).unwrap();
        ServerboundAcceptCodeOfConductPacket
            .write(&mut bytes)
            .unwrap();
        assert!(bytes.is_empty());
        assert_eq!(
            ClientboundFinishConfigurationPacket::read(&mut Cursor::new(Vec::new())).unwrap(),
            ClientboundFinishConfigurationPacket
        );
    }

    #[test]
    fn round_trips_code_of_conduct_and_enabled_features() {
        let conduct = ClientboundCodeOfConductPacket {
            code_of_conduct: "Be excellent.".to_string(),
        };
        let mut bytes = Vec::new();
        conduct.write(&mut bytes).unwrap();
        assert_eq!(
            ClientboundCodeOfConductPacket::read(&mut Cursor::new(bytes)).unwrap(),
            conduct
        );

        let features = ClientboundUpdateEnabledFeaturesPacket {
            features: vec![Identifier::parse("minecraft:vanilla").unwrap()],
        };
        let mut bytes = Vec::new();
        features.write(&mut bytes).unwrap();
        assert_eq!(
            ClientboundUpdateEnabledFeaturesPacket::read(&mut Cursor::new(bytes)).unwrap(),
            features
        );
    }

    #[test]
    fn round_trips_known_pack_selection_and_enforces_serverbound_limit() {
        let pack = KnownPack::vanilla("core", "26.1.2");
        assert!(pack.is_vanilla());

        let clientbound = ClientboundSelectKnownPacks {
            known_packs: vec![pack.clone()],
        };
        let mut bytes = Vec::new();
        clientbound.write(&mut bytes).unwrap();
        assert_eq!(
            ClientboundSelectKnownPacks::read(&mut Cursor::new(bytes)).unwrap(),
            clientbound
        );

        let serverbound = ServerboundSelectKnownPacks {
            known_packs: vec![pack; ServerboundSelectKnownPacks::MAX_KNOWN_PACKS + 1],
        };
        assert!(serverbound.write(&mut Vec::new()).is_err());
    }

    #[test]
    fn round_trips_registry_data_entries_with_optional_nbt() {
        let packet = ClientboundRegistryDataPacket {
            registry: Identifier::parse("minecraft:damage_type").unwrap(),
            entries: vec![PackedRegistryEntry {
                id: Identifier::parse("minecraft:generic").unwrap(),
                data: Some(Tag::Compound(vec![(
                    "message_id".to_string(),
                    Tag::String("generic".to_string()),
                )])),
            }],
        };
        let mut bytes = Vec::new();
        packet.write(&mut bytes).unwrap();
        assert_eq!(
            ClientboundRegistryDataPacket::read(&mut Cursor::new(bytes)).unwrap(),
            packet
        );
    }

    #[test]
    fn configuration_session_syncs_registries_features_known_packs_and_code_of_conduct() {
        let feature = Identifier::parse("minecraft:vanilla").unwrap();
        let pack = KnownPack::vanilla("core", "26.1.2");
        let registry = ClientboundRegistryDataPacket {
            registry: Identifier::parse("minecraft:damage_type").unwrap(),
            entries: vec![PackedRegistryEntry {
                id: Identifier::parse("minecraft:generic").unwrap(),
                data: None,
            }],
        };

        let mut session = ConfigurationSession::new(vec![feature.clone()]);
        session.add_registry_data(registry.clone());
        session.offer_known_packs(vec![pack.clone()]);
        session.require_code_of_conduct("Be excellent.".to_string());

        let start = session.start();
        assert_eq!(session.state, ConfigurationState::AwaitingKnownPacks);
        assert_eq!(
            start,
            vec![
                ConfigurationInstruction::RegistryData(registry),
                ConfigurationInstruction::EnabledFeatures(ClientboundUpdateEnabledFeaturesPacket {
                    features: vec![feature]
                }),
                ConfigurationInstruction::SelectKnownPacks(ClientboundSelectKnownPacks {
                    known_packs: vec![pack.clone()]
                })
            ]
        );

        let next = session
            .select_known_packs(ServerboundSelectKnownPacks {
                known_packs: vec![pack],
            })
            .unwrap();
        assert_eq!(session.state, ConfigurationState::AwaitingCodeOfConduct);
        assert_eq!(
            next,
            vec![ConfigurationInstruction::CodeOfConduct(
                ClientboundCodeOfConductPacket {
                    code_of_conduct: "Be excellent.".to_string()
                }
            )]
        );

        let finish = session
            .accept_code_of_conduct(ServerboundAcceptCodeOfConductPacket)
            .unwrap();
        assert_eq!(finish, ClientboundFinishConfigurationPacket);
        assert_eq!(session.state, ConfigurationState::ReadyToFinish);
        session.finish(ServerboundFinishConfigurationPacket);
        assert_eq!(session.state, ConfigurationState::Finished);
    }

    #[test]
    fn builds_registry_sync_packet_from_registry_in_numeric_id_order() {
        let mut registry = Registry::new(Identifier::parse("minecraft:chat_type").unwrap());
        registry
            .register(
                Identifier::parse("minecraft:chat").unwrap(),
                "chat",
                Lifecycle::Stable,
            )
            .unwrap();
        registry
            .register(
                Identifier::parse("minecraft:raw").unwrap(),
                "raw",
                Lifecycle::Stable,
            )
            .unwrap();
        registry.freeze();

        let packet = ClientboundRegistryDataPacket::from_registry(&registry, |value| {
            Some(Tag::Compound(vec![(
                "translation_key".to_string(),
                Tag::String(format!("chat.type.{value}")),
            )]))
        });

        assert_eq!(
            packet.registry,
            Identifier::parse("minecraft:chat_type").unwrap()
        );
        assert_eq!(
            packet.entries[0].id,
            Identifier::parse("minecraft:chat").unwrap()
        );
        assert_eq!(
            packet.entries[1].id,
            Identifier::parse("minecraft:raw").unwrap()
        );
        assert_eq!(packet.entries.len(), 2);
    }

    #[test]
    fn configuration_session_rejects_unoffered_known_packs() {
        let mut session = ConfigurationSession::new(Vec::new());
        session.offer_known_packs(vec![KnownPack::vanilla("core", "26.1.2")]);
        session.start();
        let err = session
            .select_known_packs(ServerboundSelectKnownPacks {
                known_packs: vec![KnownPack::vanilla("other", "26.1.2")],
            })
            .unwrap_err();
        assert_eq!(err.kind(), std::io::ErrorKind::InvalidData);
    }
}
