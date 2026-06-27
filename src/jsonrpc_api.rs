#![allow(dead_code)]

use std::collections::BTreeMap;

use crate::player_access::NameAndId;
use crate::registry::Identifier;

pub const JSONRPC_SCHEMA_REF_PREFIX: &str = "#/components/schemas/";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JsonRpcSchema {
    pub reference: Option<String>,
    pub types: Vec<String>,
    pub items: Option<Box<JsonRpcSchema>>,
    pub properties: BTreeMap<String, JsonRpcSchema>,
    pub enum_values: Vec<String>,
    pub codec: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SchemaComponent {
    pub name: String,
    pub reference: String,
    pub schema: JsonRpcSchema,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParamInfo {
    pub name: String,
    pub schema: JsonRpcSchema,
    pub required: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResultInfo {
    pub name: String,
    pub schema: JsonRpcSchema,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MethodInfo {
    pub description: String,
    pub params: Option<ParamInfo>,
    pub result: Option<ResultInfo>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NamedMethodInfo {
    pub name: Identifier,
    pub contents: MethodInfo,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JsonRpcPlayerDto {
    pub id: Option<String>,
    pub name: Option<String>,
}

impl JsonRpcSchema {
    pub fn info(&self) -> Self {
        Self {
            reference: self.reference.clone(),
            types: self.types.clone(),
            items: self.items.as_ref().map(|item| Box::new(item.info())),
            properties: self
                .properties
                .iter()
                .map(|(key, value)| (key.clone(), value.info()))
                .collect(),
            enum_values: self.enum_values.clone(),
            codec: self.codec.clone(),
        }
    }

    pub fn of_ref(reference: impl Into<String>, codec: impl Into<String>) -> Self {
        Self {
            reference: Some(reference.into()),
            types: Vec::new(),
            items: None,
            properties: BTreeMap::new(),
            enum_values: Vec::new(),
            codec: codec.into(),
        }
    }

    pub fn of_type(schema_type: impl Into<String>, codec: impl Into<String>) -> Self {
        Self::of_types(vec![schema_type.into()], codec)
    }

    pub fn of_types(types: Vec<String>, codec: impl Into<String>) -> Self {
        Self {
            reference: None,
            types,
            items: None,
            properties: BTreeMap::new(),
            enum_values: Vec::new(),
            codec: codec.into(),
        }
    }

    pub fn of_enum(enum_values: Vec<String>, codec: impl Into<String>) -> Self {
        Self {
            reference: None,
            types: vec!["string".to_string()],
            items: None,
            properties: BTreeMap::new(),
            enum_values,
            codec: codec.into(),
        }
    }

    pub fn array_of(item: &Self, codec: impl Into<String>) -> Self {
        Self {
            reference: None,
            types: vec!["array".to_string()],
            items: Some(Box::new(item.clone())),
            properties: BTreeMap::new(),
            enum_values: Vec::new(),
            codec: codec.into(),
        }
    }

    pub fn record(codec: impl Into<String>) -> Self {
        Self {
            reference: None,
            types: vec!["object".to_string()],
            items: None,
            properties: BTreeMap::new(),
            enum_values: Vec::new(),
            codec: codec.into(),
        }
    }

    pub fn with_field(&self, name: impl Into<String>, field: Self) -> Self {
        let mut properties = self.properties.clone();
        properties.insert(name.into(), field);
        Self {
            reference: None,
            types: vec!["object".to_string()],
            items: None,
            properties,
            enum_values: Vec::new(),
            codec: self.codec.clone(),
        }
    }

    pub fn as_array(&self) -> Self {
        Self::array_of(self, format!("{}.listOf()", self.codec))
    }
}

impl SchemaComponent {
    pub fn new(name: impl Into<String>, schema: JsonRpcSchema) -> Self {
        let name = name.into();
        Self {
            reference: create_local_reference(&name),
            name,
            schema,
        }
    }

    pub fn as_ref(&self) -> JsonRpcSchema {
        JsonRpcSchema::of_ref(self.reference.clone(), self.schema.codec.clone())
    }

    pub fn as_array(&self) -> JsonRpcSchema {
        JsonRpcSchema::array_of(&self.as_ref(), self.schema.codec.clone())
    }
}

impl ParamInfo {
    pub fn new(name: impl Into<String>, schema: JsonRpcSchema) -> Self {
        Self {
            name: name.into(),
            schema,
            required: true,
        }
    }

    pub fn new_with_required(
        name: impl Into<String>,
        schema: JsonRpcSchema,
        required: bool,
    ) -> Self {
        Self {
            name: name.into(),
            schema,
            required,
        }
    }
}

impl ResultInfo {
    pub fn new(name: impl Into<String>, schema: JsonRpcSchema) -> Self {
        Self {
            name: name.into(),
            schema,
        }
    }
}

impl MethodInfo {
    pub fn new(
        description: impl Into<String>,
        params: Option<ParamInfo>,
        result: Option<ResultInfo>,
    ) -> Self {
        Self {
            description: description.into(),
            params,
            result,
        }
    }

    pub fn new_nullable(
        description: impl Into<String>,
        param_info: Option<ParamInfo>,
        result_info: Option<ResultInfo>,
    ) -> Self {
        Self::new(description, param_info, result_info)
    }

    pub fn params_to_optional(params: &[ParamInfo]) -> Option<ParamInfo> {
        params.first().cloned()
    }

    pub fn params_to_list(param: &Option<ParamInfo>) -> Vec<ParamInfo> {
        match param {
            Some(param) => vec![param.clone()],
            None => Vec::new(),
        }
    }

    pub fn named(&self, name: Identifier) -> NamedMethodInfo {
        NamedMethodInfo {
            name,
            contents: self.clone(),
        }
    }
}

impl JsonRpcPlayerDto {
    pub fn new(id: Option<String>, name: Option<String>) -> Self {
        Self { id, name }
    }

    pub fn from_name_and_id(name_and_id: &NameAndId) -> Self {
        Self {
            id: Some(name_and_id.uuid.clone()),
            name: Some(name_and_id.name.clone()),
        }
    }
}

pub fn create_local_reference(type_id: &str) -> String {
    format!("{JSONRPC_SCHEMA_REF_PREFIX}{type_id}")
}

pub fn parse_reference(value: &str) -> Result<String, String> {
    if value.chars().any(char::is_whitespace) {
        Err(format!("Illegal character in path at index {}", first_whitespace_index(value)))
    } else {
        Ok(value.to_string())
    }
}

pub fn schema_registry() -> Vec<SchemaComponent> {
    let player = player_schema_component();
    let version = version_schema_component();
    let message = message_schema_component();

    vec![
        difficulty_schema_component(),
        game_type_schema_component(),
        player.clone(),
        version.clone(),
        server_state_schema_component(&player, &version),
        typed_game_rule_schema_component(),
        untyped_game_rule_schema_component(),
        message.clone(),
        system_message_schema_component(&player, &message),
        kick_player_schema_component(&player, &message),
        operator_schema_component(&player),
        incoming_ip_ban_schema_component(&player),
        ip_ban_schema_component(),
        user_ban_schema_component(&player),
    ]
}

fn difficulty_schema_component() -> SchemaComponent {
    SchemaComponent::new(
        "difficulty",
        JsonRpcSchema::of_enum(
            vec![
                "peaceful".to_string(),
                "easy".to_string(),
                "normal".to_string(),
                "hard".to_string(),
            ],
            "Difficulty.CODEC",
        ),
    )
}

fn game_type_schema_component() -> SchemaComponent {
    SchemaComponent::new(
        "game_type",
        JsonRpcSchema::of_enum(
            vec![
                "survival".to_string(),
                "creative".to_string(),
                "adventure".to_string(),
                "spectator".to_string(),
            ],
            "GameType.CODEC",
        ),
    )
}

fn player_schema_component() -> SchemaComponent {
    SchemaComponent::new(
        "player",
        JsonRpcSchema::record("PlayerDto.CODEC")
            .with_field("id", JsonRpcSchema::of_type("string", "UUIDUtil.CODEC"))
            .with_field("name", JsonRpcSchema::of_type("string", "Codec.STRING")),
    )
}

fn version_schema_component() -> SchemaComponent {
    SchemaComponent::new(
        "version",
        JsonRpcSchema::record("DiscoverInfo.CODEC")
            .with_field("name", JsonRpcSchema::of_type("string", "Codec.STRING"))
            .with_field("protocol", JsonRpcSchema::of_type("integer", "Codec.INT")),
    )
}

fn server_state_schema_component(
    player: &SchemaComponent,
    version: &SchemaComponent,
) -> SchemaComponent {
    SchemaComponent::new(
        "server_state",
        JsonRpcSchema::record("ServerState.CODEC")
            .with_field("started", JsonRpcSchema::of_type("boolean", "Codec.BOOL"))
            .with_field("players", player.as_ref().as_array())
            .with_field("version", version.as_ref()),
    )
}

fn typed_game_rule_schema_component() -> SchemaComponent {
    SchemaComponent::new(
        "typed_game_rule",
        game_rule_schema("GameRuleUpdate.TYPED_CODEC").with_field(
            "type",
            JsonRpcSchema::of_enum(
                vec!["boolean".to_string(), "integer".to_string()],
                "GameRuleType",
            ),
        ),
    )
}

fn untyped_game_rule_schema_component() -> SchemaComponent {
    SchemaComponent::new("untyped_game_rule", game_rule_schema("GameRuleUpdate.CODEC"))
}

fn game_rule_schema(codec: &str) -> JsonRpcSchema {
    JsonRpcSchema::record(codec)
        .with_field("key", JsonRpcSchema::of_type("string", "Codec.STRING"))
        .with_field(
            "value",
            JsonRpcSchema::of_types(
                vec!["boolean".to_string(), "integer".to_string()],
                "Codec.either(Codec.BOOL, Codec.INT)",
            ),
        )
}

fn message_schema_component() -> SchemaComponent {
    SchemaComponent::new(
        "message",
        JsonRpcSchema::record("Message.CODEC")
            .with_field("literal", JsonRpcSchema::of_type("string", "Codec.STRING"))
            .with_field("translatable", JsonRpcSchema::of_type("string", "Codec.STRING"))
            .with_field(
                "translatableParams",
                JsonRpcSchema::of_type("string", "Codec.STRING").as_array(),
            ),
    )
}

fn system_message_schema_component(
    player: &SchemaComponent,
    message: &SchemaComponent,
) -> SchemaComponent {
    SchemaComponent::new(
        "system_message",
        JsonRpcSchema::record("SystemMessage.CODEC")
            .with_field("message", message.as_ref())
            .with_field("overlay", JsonRpcSchema::of_type("boolean", "Codec.BOOL"))
            .with_field("receivingPlayers", player.as_ref().as_array()),
    )
}

fn kick_player_schema_component(
    player: &SchemaComponent,
    message: &SchemaComponent,
) -> SchemaComponent {
    SchemaComponent::new(
        "kick_player",
        JsonRpcSchema::record("KickDto.CODEC")
            .with_field("message", message.as_ref())
            .with_field("player", player.as_ref()),
    )
}

fn operator_schema_component(player: &SchemaComponent) -> SchemaComponent {
    SchemaComponent::new(
        "operator",
        JsonRpcSchema::record("OperatorDto.CODEC")
            .with_field("player", player.as_ref())
            .with_field("bypassesPlayerLimit", JsonRpcSchema::of_type("boolean", "Codec.BOOL"))
            .with_field("permissionLevel", JsonRpcSchema::of_type("integer", "Codec.INT")),
    )
}

fn incoming_ip_ban_schema_component(player: &SchemaComponent) -> SchemaComponent {
    SchemaComponent::new(
        "incoming_ip_ban",
        ip_ban_schema("IncomingIpBanDto.CODEC").with_field("player", player.as_ref()),
    )
}

fn ip_ban_schema_component() -> SchemaComponent {
    SchemaComponent::new("ip_ban", ip_ban_schema("IpBanDto.CODEC"))
}

fn user_ban_schema_component(player: &SchemaComponent) -> SchemaComponent {
    SchemaComponent::new(
        "user_ban",
        JsonRpcSchema::record("UserBanDto.CODEC")
            .with_field("player", player.as_ref())
            .with_field("reason", JsonRpcSchema::of_type("string", "Codec.STRING"))
            .with_field("source", JsonRpcSchema::of_type("string", "Codec.STRING"))
            .with_field("expires", JsonRpcSchema::of_type("string", "Codec.STRING")),
    )
}

fn ip_ban_schema(codec: &str) -> JsonRpcSchema {
    JsonRpcSchema::record(codec)
        .with_field("ip", JsonRpcSchema::of_type("string", "Codec.STRING"))
        .with_field("reason", JsonRpcSchema::of_type("string", "Codec.STRING"))
        .with_field("source", JsonRpcSchema::of_type("string", "Codec.STRING"))
        .with_field("expires", JsonRpcSchema::of_type("string", "Codec.STRING"))
}

fn first_whitespace_index(value: &str) -> usize {
    value
        .char_indices()
        .find_map(|(index, ch)| ch.is_whitespace().then_some(index))
        .unwrap_or(value.len())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reference_util_matches_java_local_reference_and_uri_validation() {
        assert_eq!(create_local_reference("player"), "#/components/schemas/player");
        assert_eq!(
            parse_reference("#/components/schemas/player"),
            Ok("#/components/schemas/player".to_string())
        );
        assert!(parse_reference("bad uri").is_err());
    }

    #[test]
    fn player_dto_matches_java_optional_id_and_name_shape() {
        let player = NameAndId {
            uuid: "12345678-1234-1234-1234-123456789abc".to_string(),
            name: "Steve".to_string(),
        };
        assert_eq!(
            JsonRpcPlayerDto::from_name_and_id(&player),
            JsonRpcPlayerDto {
                id: Some(player.uuid),
                name: Some(player.name),
            }
        );
        assert_eq!(JsonRpcPlayerDto::new(None, Some("Alex".to_string())).id, None);
    }

    #[test]
    fn schema_builders_match_java_metadata_transformations() {
        let string_schema = JsonRpcSchema::of_type("string", "Codec.STRING");
        assert_eq!(string_schema.types, vec!["string"]);
        assert_eq!(string_schema.reference, None);

        let enum_schema =
            JsonRpcSchema::of_enum(vec!["easy".to_string(), "hard".to_string()], "Difficulty.CODEC");
        assert_eq!(enum_schema.types, vec!["string"]);
        assert_eq!(enum_schema.enum_values, vec!["easy", "hard"]);

        let record = JsonRpcSchema::record("PlayerDto.CODEC")
            .with_field("id", JsonRpcSchema::of_type("string", "UUIDUtil.CODEC"));
        assert_eq!(record.types, vec!["object"]);
        assert_eq!(
            record.properties.get("id").map(|schema| schema.codec.as_str()),
            Some("UUIDUtil.CODEC")
        );

        let array = string_schema.as_array();
        assert_eq!(array.types, vec!["array"]);
        assert_eq!(array.items.as_ref().map(|item| item.types.clone()), Some(vec!["string".to_string()]));
    }

    #[test]
    fn schema_component_and_method_info_match_java_helpers() {
        let player_schema = SchemaComponent::new(
            "player",
            JsonRpcSchema::record("PlayerDto.CODEC"),
        );
        assert_eq!(player_schema.reference, "#/components/schemas/player");
        assert_eq!(player_schema.as_ref().reference, Some(player_schema.reference.clone()));
        assert_eq!(player_schema.as_array().types, vec!["array"]);

        let param = ParamInfo::new("player", player_schema.as_ref());
        assert!(param.required);
        assert_eq!(
            MethodInfo::params_to_optional(std::slice::from_ref(&param)),
            Some(param.clone())
        );
        assert_eq!(MethodInfo::params_to_list(&Some(param.clone())), vec![param.clone()]);
        assert_eq!(MethodInfo::params_to_list(&None), Vec::<ParamInfo>::new());

        let method = MethodInfo::new(
            "Kick a player",
            Some(param),
            Some(ResultInfo::new("players", player_schema.as_array())),
        );
        let name = match Identifier::parse("minecraft:players/kick") {
            Ok(name) => name,
            Err(err) => panic!("{err}"),
        };
        assert_eq!(method.named(name.clone()).name, name);
    }

    #[test]
    fn schema_registry_matches_java_component_names_and_core_fields() {
        let registry = schema_registry();
        let names = registry
            .iter()
            .map(|component| component.name.as_str())
            .collect::<Vec<_>>();
        assert_eq!(
            names,
            vec![
                "difficulty",
                "game_type",
                "player",
                "version",
                "server_state",
                "typed_game_rule",
                "untyped_game_rule",
                "message",
                "system_message",
                "kick_player",
                "operator",
                "incoming_ip_ban",
                "ip_ban",
                "user_ban",
            ]
        );
        let player = registry.iter().find(|component| component.name == "player");
        let player = match player {
            Some(player) => player,
            None => panic!("player schema missing"),
        };
        assert!(player.schema.properties.contains_key("id"));
        assert!(player.schema.properties.contains_key("name"));
    }

    #[test]
    #[cfg(vibecraft_has_decompiled_sources)]
    fn jsonrpc_api_sources_match_java_26_1_2() {
        const METHOD_INFO: &str =
            vibecraft_java_source!("/net/minecraft/server/jsonrpc/api/MethodInfo.java");
        const PARAM_INFO: &str =
            vibecraft_java_source!("/net/minecraft/server/jsonrpc/api/ParamInfo.java");
        const PLAYER_DTO: &str =
            vibecraft_java_source!("/net/minecraft/server/jsonrpc/api/PlayerDto.java");
        const REFERENCE_UTIL: &str =
            vibecraft_java_source!("/net/minecraft/server/jsonrpc/api/ReferenceUtil.java");
        const RESULT_INFO: &str =
            vibecraft_java_source!("/net/minecraft/server/jsonrpc/api/ResultInfo.java");
        const SCHEMA: &str = vibecraft_java_source!("/net/minecraft/server/jsonrpc/api/Schema.java");
        const SCHEMA_COMPONENT: &str =
            vibecraft_java_source!("/net/minecraft/server/jsonrpc/api/SchemaComponent.java");

        assert_java_source_contains_all(
            "MethodInfo.java",
            METHOD_INFO,
            &[
            "public record MethodInfo<Params, Result>(String description, Optional<ParamInfo<Params>> params, Optional<ResultInfo<Result>> result)",
            "return list.isEmpty() ? Optional.empty() : Optional.of(list.getFirst());",
            "return opt.isPresent() ? List.of(opt.get()) : List.of();",
            "public MethodInfo.Named<Params, Result> named(final Identifier name)",
            "public record Named<Params, Result>(Identifier name, MethodInfo<Params, Result> contents)",
        ],
        );

        assert_java_source_contains_all(
            "ParamInfo.java",
            PARAM_INFO,
            &[
                "public record ParamInfo<Param>(String name, Schema<Param> schema, boolean required)",
                "public ParamInfo(final String name, final Schema<Param> schema)",
                "Codec.BOOL.fieldOf(\"required\").forGetter(ParamInfo::required)",
            ],
        );

        assert_java_source_contains_all(
            "PlayerDto.java",
            PLAYER_DTO,
            &[
            "public record PlayerDto(Optional<UUID> id, Optional<String> name)",
            "UUIDUtil.STRING_CODEC.optionalFieldOf(\"id\").forGetter(PlayerDto::id)",
            "Codec.STRING.optionalFieldOf(\"name\").forGetter(PlayerDto::name)",
            "public static PlayerDto from(final GameProfile gameProfile)",
            "public static PlayerDto from(final NameAndId nameAndId)",
        ],
        );

        assert_java_source_contains_all(
            "ReferenceUtil.java",
            REFERENCE_UTIL,
            &[
            "public static final Codec<URI> REFERENCE_CODEC = Codec.STRING.comapFlatMap",
            "return DataResult.success(new URI(string));",
            "return DataResult.error(e::getMessage);",
            "return URI.create(\"#/components/schemas/\" + typeId);",
        ],
        );

        assert_java_source_contains_all(
            "ResultInfo.java",
            RESULT_INFO,
            &[
            "public record ResultInfo<Result>(String name, Schema<Result> schema)",
            "Codec.STRING.fieldOf(\"name\").forGetter(ResultInfo::name)",
            "Schema.typedCodec().fieldOf(\"schema\").forGetter(ResultInfo::schema)",
        ],
        );

        assert_java_source_contains_all(
            "Schema.java",
            SCHEMA,
            &[
            "public record Schema<T>(",
            "Optional<URI> reference, List<String> type, Optional<Schema<?>> items, Map<String, Schema<?>> properties, List<String> enumValues, Codec<T> codec",
            "private static final List<SchemaComponent<?>> SCHEMA_REGISTRY = new ArrayList<>();",
            "public static final Schema<Boolean> BOOL_SCHEMA = ofType(\"boolean\", Codec.BOOL);",
            "public static final Schema<Integer> INT_SCHEMA = ofType(\"integer\", Codec.INT);",
            "public static final Schema<Float> NUMBER_SCHEMA = ofType(\"number\", Codec.FLOAT);",
            "public static final Schema<String> STRING_SCHEMA = ofType(\"string\", Codec.STRING);",
            "registerSchema(",
            "\"player\", record(PlayerDto.CODEC.codec()).withField(\"id\", UUID_SCHEMA).withField(\"name\", STRING_SCHEMA)",
            "\"server_state\",",
            "\"typed_game_rule\",",
            "\"message\",",
            "\"user_ban\",",
            "public Schema<T> info()",
            "public static <T> Schema<T> ofRef(final URI ref, final Codec<T> codec)",
            "public static <T> Schema<T> ofTypes(final List<String> types, final Codec<T> codec)",
            "public static <T> Schema<T> ofEnum(final List<String> enumValues, final Codec<T> codec)",
            "public static <T> Schema<List<T>> arrayOf(final Schema<?> item, final Codec<T> codec)",
            "public Schema<T> withField(final String name, final Schema<?> field)",
            "public Schema<List<T>> asArray()",
        ],
        );

        assert_java_source_contains_all(
            "SchemaComponent.java",
            SCHEMA_COMPONENT,
            &[
            "public record SchemaComponent<T>(String name, URI ref, Schema<T> schema)",
            "return Schema.ofRef(this.ref, this.schema.codec());",
            "return Schema.arrayOf(this.asRef(), this.schema.codec());",
        ],
        );
    }

    #[cfg(vibecraft_has_decompiled_sources)]
    fn assert_java_source_contains_all(name: &str, source: &str, sentinels: &[&str]) {
        for sentinel in sentinels {
            assert!(
                source.contains(sentinel),
                "{name} is missing sentinel: {sentinel}"
            );
        }
    }
}
