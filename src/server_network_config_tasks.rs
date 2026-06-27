#![allow(dead_code)]

use crate::network::codec::Uuid;
use crate::server_network_common::ConfigurationTaskTypeModel;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConfigurationTaskPacketModel {
    FinishConfiguration,
    CodeOfConduct {
        code_of_conduct: String,
    },
    ResourcePackPush {
        id: Uuid,
        url: String,
        hash: String,
        required: bool,
        prompt: Option<String>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JoinWorldTaskModel;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerCodeOfConductConfigurationTaskModel {
    code_of_conduct: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerResourcePackInfoModel {
    pub id: Uuid,
    pub url: String,
    pub hash: String,
    pub required: bool,
    pub prompt: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerResourcePackConfigurationTaskModel {
    info: ServerResourcePackInfoModel,
}

impl JoinWorldTaskModel {
    pub const TYPE_ID: &'static str = "join_world";

    pub fn start(&self) -> Vec<ConfigurationTaskPacketModel> {
        vec![ConfigurationTaskPacketModel::FinishConfiguration]
    }

    pub fn task_type(&self) -> ConfigurationTaskTypeModel {
        ConfigurationTaskTypeModel::new(Self::TYPE_ID)
    }
}

impl ServerCodeOfConductConfigurationTaskModel {
    pub const TYPE_ID: &'static str = "server_code_of_conduct";

    pub fn new(code_of_conduct: impl Into<String>) -> Self {
        Self {
            code_of_conduct: code_of_conduct.into(),
        }
    }

    pub fn start(&self) -> Vec<ConfigurationTaskPacketModel> {
        vec![ConfigurationTaskPacketModel::CodeOfConduct {
            code_of_conduct: self.code_of_conduct.clone(),
        }]
    }

    pub fn task_type(&self) -> ConfigurationTaskTypeModel {
        ConfigurationTaskTypeModel::new(Self::TYPE_ID)
    }
}

impl ServerResourcePackConfigurationTaskModel {
    pub const TYPE_ID: &'static str = "server_resource_pack";

    pub fn new(info: ServerResourcePackInfoModel) -> Self {
        Self { info }
    }

    pub fn start(&self) -> Vec<ConfigurationTaskPacketModel> {
        vec![ConfigurationTaskPacketModel::ResourcePackPush {
            id: self.info.id,
            url: self.info.url.clone(),
            hash: self.info.hash.clone(),
            required: self.info.required,
            prompt: self.info.prompt.clone(),
        }]
    }

    pub fn task_type(&self) -> ConfigurationTaskTypeModel {
        ConfigurationTaskTypeModel::new(Self::TYPE_ID)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn uuid(byte: u8) -> Uuid {
        Uuid([byte; 16])
    }

    #[test]
    fn join_world_task_sends_finish_configuration_singleton_and_type() {
        let task = JoinWorldTaskModel;

        assert_eq!(task.task_type().to_string(), "join_world");
        assert_eq!(
            task.start(),
            vec![ConfigurationTaskPacketModel::FinishConfiguration]
        );
    }

    #[test]
    fn code_of_conduct_task_sends_supplied_text_and_type() {
        let task = ServerCodeOfConductConfigurationTaskModel::new("Be excellent.");

        assert_eq!(task.task_type().to_string(), "server_code_of_conduct");
        assert_eq!(
            task.start(),
            vec![ConfigurationTaskPacketModel::CodeOfConduct {
                code_of_conduct: "Be excellent.".to_string(),
            }]
        );
    }

    #[test]
    fn resource_pack_task_sends_push_packet_fields_and_nullable_prompt() {
        let info = ServerResourcePackInfoModel {
            id: uuid(3),
            url: "https://example.test/pack.zip".to_string(),
            hash: "0123456789abcdef0123456789abcdef01234567".to_string(),
            required: true,
            prompt: Some("{\"text\":\"Download?\"}".to_string()),
        };
        let task = ServerResourcePackConfigurationTaskModel::new(info.clone());

        assert_eq!(task.task_type().to_string(), "server_resource_pack");
        assert_eq!(
            task.start(),
            vec![ConfigurationTaskPacketModel::ResourcePackPush {
                id: info.id,
                url: info.url,
                hash: info.hash,
                required: info.required,
                prompt: Some("{\"text\":\"Download?\"}".to_string()),
            }]
        );

        let no_prompt = ServerResourcePackConfigurationTaskModel::new(ServerResourcePackInfoModel {
            id: uuid(4),
            url: "https://example.test/no-prompt.zip".to_string(),
            hash: String::new(),
            required: false,
            prompt: None,
        });
        assert_eq!(
            no_prompt.start(),
            vec![ConfigurationTaskPacketModel::ResourcePackPush {
                id: uuid(4),
                url: "https://example.test/no-prompt.zip".to_string(),
                hash: String::new(),
                required: false,
                prompt: None,
            }]
        );
    }

    #[test]
    #[cfg(vibecraft_has_decompiled_sources)]
    fn simple_config_task_sources_match_java_26_1_2() {
        const JOIN_WORLD_TASK: &str =
            vibecraft_java_source!("/net/minecraft/server/network/config/JoinWorldTask.java");
        const CODE_OF_CONDUCT_TASK: &str = vibecraft_java_source!(
            "/net/minecraft/server/network/config/ServerCodeOfConductConfigurationTask.java"
        );
        const RESOURCE_PACK_TASK: &str = vibecraft_java_source!(
            "/net/minecraft/server/network/config/ServerResourcePackConfigurationTask.java"
        );

        for sentinel in [
            "public class JoinWorldTask implements ConfigurationTask",
            "public static final ConfigurationTask.Type TYPE = new ConfigurationTask.Type(\"join_world\");",
            "connection.accept(ClientboundFinishConfigurationPacket.INSTANCE);",
            "public ConfigurationTask.Type type()",
            "return TYPE;",
        ] {
            assert!(
                JOIN_WORLD_TASK.contains(sentinel),
                "JoinWorldTask.java is missing sentinel: {sentinel}"
            );
        }

        for sentinel in [
            "public class ServerCodeOfConductConfigurationTask implements ConfigurationTask",
            "public static final ConfigurationTask.Type TYPE = new ConfigurationTask.Type(\"server_code_of_conduct\");",
            "private final Supplier<String> codeOfConduct;",
            "public ServerCodeOfConductConfigurationTask(final Supplier<String> codeOfConduct)",
            "this.codeOfConduct = codeOfConduct;",
            "connection.accept(new ClientboundCodeOfConductPacket(this.codeOfConduct.get()));",
            "public ConfigurationTask.Type type()",
            "return TYPE;",
        ] {
            assert!(
                CODE_OF_CONDUCT_TASK.contains(sentinel),
                "ServerCodeOfConductConfigurationTask.java is missing sentinel: {sentinel}"
            );
        }

        for sentinel in [
            "public class ServerResourcePackConfigurationTask implements ConfigurationTask",
            "public static final ConfigurationTask.Type TYPE = new ConfigurationTask.Type(\"server_resource_pack\");",
            "private final MinecraftServer.ServerResourcePackInfo info;",
            "public ServerResourcePackConfigurationTask(final MinecraftServer.ServerResourcePackInfo info)",
            "this.info = info;",
            "new ClientboundResourcePackPushPacket(",
            "this.info.id(), this.info.url(), this.info.hash(), this.info.isRequired(), Optional.ofNullable(this.info.prompt())",
            "public ConfigurationTask.Type type()",
            "return TYPE;",
        ] {
            assert!(
                RESOURCE_PACK_TASK.contains(sentinel),
                "ServerResourcePackConfigurationTask.java is missing sentinel: {sentinel}"
            );
        }
    }
}
