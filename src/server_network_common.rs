#![allow(dead_code)]

use crate::network::codec::Uuid;
use crate::network::common::ClientInformation;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GameProfileModel {
    pub id: Uuid,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommonListenerCookieModel {
    pub game_profile: GameProfileModel,
    pub latency: i32,
    pub client_information: ClientInformation,
    pub transferred: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfigurationTaskTypeModel {
    id: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfigurationTaskModel {
    task_type: ConfigurationTaskTypeModel,
    sent_packets: Vec<String>,
}

impl CommonListenerCookieModel {
    pub fn new(
        game_profile: GameProfileModel,
        latency: i32,
        client_information: ClientInformation,
        transferred: bool,
    ) -> Self {
        Self {
            game_profile,
            latency,
            client_information,
            transferred,
        }
    }

    pub fn create_initial(game_profile: GameProfileModel, transferred: bool) -> Self {
        Self::new(game_profile, 0, ClientInformation::default(), transferred)
    }
}

impl ConfigurationTaskTypeModel {
    pub fn new(id: impl Into<String>) -> Self {
        Self { id: id.into() }
    }

    pub fn id(&self) -> &str {
        &self.id
    }
}

impl std::fmt::Display for ConfigurationTaskTypeModel {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.id)
    }
}

impl ConfigurationTaskModel {
    pub fn new(task_type: ConfigurationTaskTypeModel) -> Self {
        Self {
            task_type,
            sent_packets: Vec::new(),
        }
    }

    pub fn start(&mut self, packet: impl Into<String>) {
        self.sent_packets.push(packet.into());
    }

    pub fn tick(&self) -> bool {
        false
    }

    pub fn task_type(&self) -> &ConfigurationTaskTypeModel {
        &self.task_type
    }

    pub fn sent_packets(&self) -> &[String] {
        &self.sent_packets
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::network::common::{ChatVisibility, HumanoidArm, ParticleStatus};

    fn profile() -> GameProfileModel {
        GameProfileModel {
            id: Uuid([7; 16]),
            name: "Alex".to_string(),
        }
    }

    #[test]
    fn common_listener_cookie_create_initial_matches_java_defaults() {
        let profile = profile();
        let cookie = CommonListenerCookieModel::create_initial(profile.clone(), true);

        assert_eq!(cookie.game_profile, profile);
        assert_eq!(cookie.latency, 0);
        assert!(cookie.transferred);
        assert_eq!(cookie.client_information.language, "en_us");
        assert_eq!(cookie.client_information.view_distance, 2);
        assert_eq!(cookie.client_information.chat_visibility, ChatVisibility::Full);
        assert!(cookie.client_information.chat_colors);
        assert_eq!(cookie.client_information.model_customisation, 0);
        assert_eq!(cookie.client_information.main_hand, HumanoidArm::Right);
        assert!(!cookie.client_information.text_filtering_enabled);
        assert!(!cookie.client_information.allows_listing);
        assert_eq!(cookie.client_information.particle_status, ParticleStatus::All);

        let custom_client = ClientInformation {
            language: "pirate".to_string(),
            view_distance: 8,
            chat_visibility: ChatVisibility::Hidden,
            chat_colors: false,
            model_customisation: 0xff,
            main_hand: HumanoidArm::Left,
            text_filtering_enabled: true,
            allows_listing: true,
            particle_status: ParticleStatus::Minimal,
        };
        assert_eq!(
            CommonListenerCookieModel::new(profile, 123, custom_client.clone(), false),
            CommonListenerCookieModel {
                game_profile: GameProfileModel {
                    id: Uuid([7; 16]),
                    name: "Alex".to_string(),
                },
                latency: 123,
                client_information: custom_client,
                transferred: false,
            }
        );
    }

    #[test]
    fn configuration_task_type_to_string_and_default_tick_match_java() {
        let task_type = ConfigurationTaskTypeModel::new("minecraft:test");
        assert_eq!(task_type.id(), "minecraft:test");
        assert_eq!(task_type.to_string(), "minecraft:test");

        let mut task = ConfigurationTaskModel::new(task_type);
        assert!(!task.tick());
        task.start("packet-a");
        task.start("packet-b");
        assert_eq!(
            task.sent_packets(),
            &["packet-a".to_string(), "packet-b".to_string()]
        );
        assert_eq!(task.task_type().to_string(), "minecraft:test");
    }

    #[test]
    #[cfg(vibecraft_has_decompiled_sources)]
    fn common_network_sources_match_java_26_1_2() {
        const COMMON_LISTENER_COOKIE: &str =
            vibecraft_java_source!("/net/minecraft/server/network/CommonListenerCookie.java");
        const CONFIGURATION_TASK: &str =
            vibecraft_java_source!("/net/minecraft/server/network/ConfigurationTask.java");

        for sentinel in [
            "public record CommonListenerCookie(GameProfile gameProfile, int latency, ClientInformation clientInformation, boolean transferred)",
            "public static CommonListenerCookie createInitial(final GameProfile gameProfile, final boolean transferred)",
            "return new CommonListenerCookie(gameProfile, 0, ClientInformation.createDefault(), transferred);",
        ] {
            assert!(
                COMMON_LISTENER_COOKIE.contains(sentinel),
                "CommonListenerCookie.java is missing sentinel: {sentinel}"
            );
        }

        for sentinel in [
            "public interface ConfigurationTask",
            "void start(Consumer<Packet<?>> connection);",
            "default boolean tick()",
            "return false;",
            "ConfigurationTask.Type type();",
            "record Type(String id)",
            "public String toString()",
            "return this.id;",
        ] {
            assert!(
                CONFIGURATION_TASK.contains(sentinel),
                "ConfigurationTask.java is missing sentinel: {sentinel}"
            );
        }
    }
}
