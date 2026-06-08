use std::collections::HashMap;

pub trait ArgumentTypeModel {
    type Output;
}

pub trait SignedArgumentModel: ArgumentTypeModel {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExampleSignedArgument;

impl ArgumentTypeModel for ExampleSignedArgument {
    type Output = String;
}

impl SignedArgumentModel for ExampleSignedArgument {}

pub fn signed_argument_output<T: SignedArgumentModel>(_: &T, value: T::Output) -> T::Output {
    value
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PackageInfoModel {
    package_name: &'static str,
    null_marked: bool,
    annotation_type: &'static str,
}

impl PackageInfoModel {
    pub fn java_commands_arguments_package() -> Self {
        Self {
            package_name: "net.minecraft.commands.arguments",
            null_marked: true,
            annotation_type: "org.jspecify.annotations.NullMarked",
        }
    }

    pub fn package_name(&self) -> &'static str {
        self.package_name
    }

    pub fn is_null_marked(&self) -> bool {
        self.null_marked
    }

    pub fn annotation_type(&self) -> &'static str {
        self.annotation_type
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CommandContextModel {
    selectors: HashMap<String, EntitySelectorModel>,
    source: CommandSourceStackModel,
}

impl CommandContextModel {
    pub fn with_selector(mut self, name: impl Into<String>, selector: EntitySelectorModel) -> Self {
        self.selectors.insert(name.into(), selector);
        self
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CommandSourceStackModel;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntitySelectorModel {
    selected: SelectedEntityModel,
}

impl EntitySelectorModel {
    pub fn new(selected: SelectedEntityModel) -> Self {
        Self { selected }
    }

    pub fn find_single_entity(&self, _source: &CommandSourceStackModel) -> SelectedEntityModel {
        self.selected.clone()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SelectedEntityModel {
    Waypoint(WaypointTransmitterModel),
    Other(EntityModel),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntityModel {
    id: String,
}

impl EntityModel {
    pub fn new(id: impl Into<String>) -> Self {
        Self { id: id.into() }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WaypointTransmitterModel {
    id: String,
}

impl WaypointTransmitterModel {
    pub fn new(id: impl Into<String>) -> Self {
        Self { id: id.into() }
    }

    pub fn id(&self) -> &str {
        &self.id
    }
}

pub fn get_waypoint(
    context: &CommandContextModel,
    name: &str,
) -> Result<WaypointTransmitterModel, WaypointArgumentError> {
    let selector =
        context
            .selectors
            .get(name)
            .ok_or_else(|| WaypointArgumentError::MissingArgument {
                name: name.to_string(),
            })?;
    match selector.find_single_entity(&context.source) {
        SelectedEntityModel::Waypoint(waypoint) => Ok(waypoint),
        SelectedEntityModel::Other(_entity) => Err(WaypointArgumentError::NotAWaypoint),
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WaypointArgumentError {
    MissingArgument { name: String },
    NotAWaypoint,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn java_signed_argument_is_only_an_argument_type_marker_interface() {
        let argument = ExampleSignedArgument;
        assert_eq!(
            signed_argument_output(&argument, "signed payload".to_string()),
            "signed payload"
        );
    }

    #[test]
    fn java_arguments_package_info_is_null_marked() {
        let info = PackageInfoModel::java_commands_arguments_package();

        assert_eq!(info.package_name(), "net.minecraft.commands.arguments");
        assert!(info.is_null_marked());
        assert_eq!(
            info.annotation_type(),
            "org.jspecify.annotations.NullMarked"
        );
    }

    #[test]
    fn java_waypoint_argument_returns_selected_waypoint_transmitter() {
        let waypoint = WaypointTransmitterModel::new("waypoint-1");
        let context = CommandContextModel::default().with_selector(
            "target",
            EntitySelectorModel::new(SelectedEntityModel::Waypoint(waypoint.clone())),
        );

        assert_eq!(get_waypoint(&context, "target"), Ok(waypoint));
        assert_eq!(get_waypoint(&context, "target").unwrap().id(), "waypoint-1");
    }

    #[test]
    fn java_waypoint_argument_rejects_non_waypoint_entities() {
        let context = CommandContextModel::default().with_selector(
            "target",
            EntitySelectorModel::new(SelectedEntityModel::Other(EntityModel::new("plain-entity"))),
        );

        assert_eq!(
            get_waypoint(&context, "target"),
            Err(WaypointArgumentError::NotAWaypoint)
        );
    }

    #[test]
    fn java_waypoint_argument_reads_named_entity_selector_argument() {
        let context = CommandContextModel::default();

        assert_eq!(
            get_waypoint(&context, "missing"),
            Err(WaypointArgumentError::MissingArgument {
                name: "missing".to_string()
            })
        );
    }
}
