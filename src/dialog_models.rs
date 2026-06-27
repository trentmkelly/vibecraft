use crate::chat_component::common_components::CommonComponents;
use crate::chat_component::Component;
use crate::registry::Identifier;

use super::{
    DialogAction, DialogActionModel, DialogBody, DialogType, InputControl, StaticDialogAction,
    COMMON_BUTTON_DEFAULT_WIDTH, DIALOG_WIDTH_MAX, DIALOG_WIDTH_MIN,
};

pub const BUTTON_LIST_DEFAULT_COLUMNS: i32 = 2;
pub const DIALOG_LIST_DEFAULT_BUTTON_WIDTH: i32 = 150;
pub const SERVER_LINKS_DEFAULT_BUTTON_WIDTH: i32 = 150;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommonButtonData {
    pub label: Component,
    pub tooltip: Option<Component>,
    pub width: i32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ActionButton {
    pub button: CommonButtonData,
    pub action: Option<DialogActionModel>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CommonDialogData {
    pub title: Component,
    pub external_title: Option<Component>,
    pub can_close_with_escape: bool,
    pub pause: bool,
    pub after_action: DialogAction,
    pub body: Vec<DialogBody>,
    pub inputs: Vec<InputControl>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct NoticeDialog {
    pub common: CommonDialogData,
    pub action: ActionButton,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ConfirmationDialog {
    pub common: CommonDialogData,
    pub yes_button: ActionButton,
    pub no_button: ActionButton,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SimpleDialogModel {
    Notice(Box<NoticeDialog>),
    Confirmation(Box<ConfirmationDialog>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DialogHolderSet {
    Tag(Identifier),
    List(Vec<Identifier>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct MultiActionDialog {
    pub common: CommonDialogData,
    pub actions: Vec<ActionButton>,
    pub exit_action: Option<ActionButton>,
    pub columns: i32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DialogListDialog {
    pub common: CommonDialogData,
    pub dialogs: DialogHolderSet,
    pub exit_action: Option<ActionButton>,
    pub columns: i32,
    pub button_width: i32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ServerLinksDialog {
    pub common: CommonDialogData,
    pub exit_action: Option<ActionButton>,
    pub columns: i32,
    pub button_width: i32,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ButtonListDialogModel {
    MultiAction(Box<MultiActionDialog>),
    DialogList(Box<DialogListDialog>),
    ServerLinks(Box<ServerLinksDialog>),
}

impl CommonButtonData {
    pub fn new(
        label: Component,
        tooltip: Option<Component>,
        width: i32,
    ) -> Result<Self, String> {
        if !(DIALOG_WIDTH_MIN..=DIALOG_WIDTH_MAX).contains(&width) {
            return Err(format!(
                "button width {width} is outside Java Dialog.WIDTH_CODEC range {DIALOG_WIDTH_MIN}..={DIALOG_WIDTH_MAX}"
            ));
        }

        Ok(Self {
            label,
            tooltip,
            width,
        })
    }

    pub fn with_width(label: Component, width: i32) -> Result<Self, String> {
        Self::new(label, None, width)
    }

    pub fn with_default_width(label: Component) -> Self {
        Self {
            label,
            tooltip: None,
            width: COMMON_BUTTON_DEFAULT_WIDTH,
        }
    }
}

impl ActionButton {
    pub fn new(button: CommonButtonData, action: Option<DialogActionModel>) -> Self {
        Self { button, action }
    }
}

impl CommonDialogData {
    pub fn new(
        title: Component,
        external_title: Option<Component>,
        can_close_with_escape: bool,
        pause: bool,
        after_action: DialogAction,
        body: Vec<DialogBody>,
        inputs: Vec<InputControl>,
    ) -> Result<Self, String> {
        if pause && !after_action.will_unpause() {
            return Err(
                "Dialogs that pause the game must use after_action values that unpause it after user action!"
                    .to_string(),
            );
        }

        Ok(Self {
            title,
            external_title,
            can_close_with_escape,
            pause,
            after_action,
            body,
            inputs,
        })
    }

    pub fn with_defaults(title: Component) -> Self {
        Self {
            title,
            external_title: None,
            can_close_with_escape: true,
            pause: true,
            after_action: DialogAction::Close,
            body: Vec::new(),
            inputs: Vec::new(),
        }
    }

    pub fn compute_external_title(&self) -> Component {
        match &self.external_title {
            Some(title) => title.clone(),
            None => self.title.clone(),
        }
    }
}

impl NoticeDialog {
    pub fn new(common: CommonDialogData, action: ActionButton) -> Self {
        Self { common, action }
    }

    pub fn default_action() -> ActionButton {
        ActionButton::new(
            CommonButtonData {
                label: CommonComponents::gui_ok(),
                tooltip: None,
                width: COMMON_BUTTON_DEFAULT_WIDTH,
            },
            None,
        )
    }

    pub fn dialog_type(&self) -> DialogType {
        DialogType::Notice
    }

    pub fn on_cancel(&self) -> Option<&DialogActionModel> {
        self.action.action.as_ref()
    }

    pub fn main_actions(&self) -> Vec<&ActionButton> {
        vec![&self.action]
    }
}

impl ConfirmationDialog {
    pub fn new(common: CommonDialogData, yes_button: ActionButton, no_button: ActionButton) -> Self {
        Self {
            common,
            yes_button,
            no_button,
        }
    }

    pub fn dialog_type(&self) -> DialogType {
        DialogType::Confirmation
    }

    pub fn on_cancel(&self) -> Option<&DialogActionModel> {
        self.no_button.action.as_ref()
    }

    pub fn main_actions(&self) -> Vec<&ActionButton> {
        vec![&self.yes_button, &self.no_button]
    }
}

impl DialogHolderSet {
    pub fn tag(id: &str) -> Result<Self, String> {
        Ok(Self::Tag(Identifier::parse(id)?))
    }

    pub fn list(ids: &[&str]) -> Result<Self, String> {
        ids.iter()
            .map(|id| Identifier::parse(id))
            .collect::<Result<Vec<_>, _>>()
            .map(Self::List)
    }
}

impl MultiActionDialog {
    pub fn new(
        common: CommonDialogData,
        actions: Vec<ActionButton>,
        exit_action: Option<ActionButton>,
        columns: i32,
    ) -> Result<Self, String> {
        if actions.is_empty() {
            return Err("multi action dialog requires a non-empty actions list".to_string());
        }
        validate_positive_columns(columns)?;

        Ok(Self {
            common,
            actions,
            exit_action,
            columns,
        })
    }

    pub fn with_default_columns(
        common: CommonDialogData,
        actions: Vec<ActionButton>,
        exit_action: Option<ActionButton>,
    ) -> Result<Self, String> {
        Self::new(common, actions, exit_action, BUTTON_LIST_DEFAULT_COLUMNS)
    }

    pub fn dialog_type(&self) -> DialogType {
        DialogType::MultiAction
    }

    pub fn on_cancel(&self) -> Option<&DialogActionModel> {
        exit_action_on_cancel(self.exit_action.as_ref())
    }
}

impl DialogListDialog {
    pub fn new(
        common: CommonDialogData,
        dialogs: DialogHolderSet,
        exit_action: Option<ActionButton>,
        columns: i32,
        button_width: i32,
    ) -> Result<Self, String> {
        validate_positive_columns(columns)?;
        validate_button_width("dialog list button_width", button_width)?;

        Ok(Self {
            common,
            dialogs,
            exit_action,
            columns,
            button_width,
        })
    }

    pub fn with_defaults(
        common: CommonDialogData,
        dialogs: DialogHolderSet,
        exit_action: Option<ActionButton>,
    ) -> Self {
        Self {
            common,
            dialogs,
            exit_action,
            columns: BUTTON_LIST_DEFAULT_COLUMNS,
            button_width: DIALOG_LIST_DEFAULT_BUTTON_WIDTH,
        }
    }

    pub fn dialog_type(&self) -> DialogType {
        DialogType::DialogList
    }

    pub fn on_cancel(&self) -> Option<&DialogActionModel> {
        exit_action_on_cancel(self.exit_action.as_ref())
    }
}

impl ServerLinksDialog {
    pub fn new(
        common: CommonDialogData,
        exit_action: Option<ActionButton>,
        columns: i32,
        button_width: i32,
    ) -> Result<Self, String> {
        validate_positive_columns(columns)?;
        validate_button_width("server links button_width", button_width)?;

        Ok(Self {
            common,
            exit_action,
            columns,
            button_width,
        })
    }

    pub fn with_defaults(common: CommonDialogData, exit_action: Option<ActionButton>) -> Self {
        Self {
            common,
            exit_action,
            columns: BUTTON_LIST_DEFAULT_COLUMNS,
            button_width: SERVER_LINKS_DEFAULT_BUTTON_WIDTH,
        }
    }

    pub fn dialog_type(&self) -> DialogType {
        DialogType::ServerLinks
    }

    pub fn on_cancel(&self) -> Option<&DialogActionModel> {
        exit_action_on_cancel(self.exit_action.as_ref())
    }
}

impl ButtonListDialogModel {
    pub fn dialog_type(&self) -> DialogType {
        match self {
            Self::MultiAction(dialog) => dialog.dialog_type(),
            Self::DialogList(dialog) => dialog.dialog_type(),
            Self::ServerLinks(dialog) => dialog.dialog_type(),
        }
    }

    pub fn common(&self) -> &CommonDialogData {
        match self {
            Self::MultiAction(dialog) => &dialog.common,
            Self::DialogList(dialog) => &dialog.common,
            Self::ServerLinks(dialog) => &dialog.common,
        }
    }

    pub fn columns(&self) -> i32 {
        match self {
            Self::MultiAction(dialog) => dialog.columns,
            Self::DialogList(dialog) => dialog.columns,
            Self::ServerLinks(dialog) => dialog.columns,
        }
    }

    pub fn exit_action(&self) -> Option<&ActionButton> {
        match self {
            Self::MultiAction(dialog) => dialog.exit_action.as_ref(),
            Self::DialogList(dialog) => dialog.exit_action.as_ref(),
            Self::ServerLinks(dialog) => dialog.exit_action.as_ref(),
        }
    }

    pub fn on_cancel(&self) -> Option<&DialogActionModel> {
        match self {
            Self::MultiAction(dialog) => dialog.on_cancel(),
            Self::DialogList(dialog) => dialog.on_cancel(),
            Self::ServerLinks(dialog) => dialog.on_cancel(),
        }
    }
}

fn validate_positive_columns(columns: i32) -> Result<(), String> {
    if columns <= 0 {
        Err("button list dialog columns must be positive".to_string())
    } else {
        Ok(())
    }
}

fn validate_button_width(field_name: &str, width: i32) -> Result<(), String> {
    if !(DIALOG_WIDTH_MIN..=DIALOG_WIDTH_MAX).contains(&width) {
        Err(format!(
            "{field_name} {width} is outside Java Dialog.WIDTH_CODEC range {DIALOG_WIDTH_MIN}..={DIALOG_WIDTH_MAX}"
        ))
    } else {
        Ok(())
    }
}

fn exit_action_on_cancel(exit_action: Option<&ActionButton>) -> Option<&DialogActionModel> {
    match exit_action {
        Some(button) => button.action.as_ref(),
        None => None,
    }
}

impl SimpleDialogModel {
    pub fn dialog_type(&self) -> DialogType {
        match self {
            Self::Notice(dialog) => dialog.dialog_type(),
            Self::Confirmation(dialog) => dialog.dialog_type(),
        }
    }

    pub fn common(&self) -> &CommonDialogData {
        match self {
            Self::Notice(dialog) => &dialog.common,
            Self::Confirmation(dialog) => &dialog.common,
        }
    }

    pub fn on_cancel(&self) -> Option<&DialogActionModel> {
        match self {
            Self::Notice(dialog) => dialog.on_cancel(),
            Self::Confirmation(dialog) => dialog.on_cancel(),
        }
    }

    pub fn main_actions(&self) -> Vec<&ActionButton> {
        match self {
            Self::Notice(dialog) => dialog.main_actions(),
            Self::Confirmation(dialog) => dialog.main_actions(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chat_component::ClickEvent;

    #[test]
    fn multi_action_dialog_matches_java_nonempty_actions_columns_and_cancel_behavior() {
        let common = common_dialog();
        let (exit_button, exit_action) = exit_button();
        let action_button = action_button();

        let multi = MultiActionDialog::with_default_columns(
            common.clone(),
            vec![action_button.clone()],
            Some(exit_button.clone()),
        );
        assert_eq!(
            multi,
            Ok(MultiActionDialog {
                common: common.clone(),
                actions: vec![action_button.clone()],
                exit_action: Some(exit_button.clone()),
                columns: BUTTON_LIST_DEFAULT_COLUMNS,
            })
        );
        let multi = match multi {
            Ok(dialog) => dialog,
            Err(err) => panic!("{err}"),
        };
        assert_eq!(multi.dialog_type(), DialogType::MultiAction);
        assert_eq!(multi.on_cancel(), Some(&exit_action));
        assert!(MultiActionDialog::with_default_columns(common.clone(), Vec::new(), None).is_err());
        assert!(MultiActionDialog::new(common, vec![action_button], None, 0).is_err());

        let button_list = ButtonListDialogModel::MultiAction(Box::new(multi));
        assert_eq!(button_list.dialog_type(), DialogType::MultiAction);
        assert_eq!(button_list.exit_action(), Some(&exit_button));
        assert_eq!(button_list.on_cancel(), Some(&exit_action));
    }

    #[test]
    fn dialog_list_dialog_matches_java_holder_set_columns_width_and_cancel_behavior() {
        let common = common_dialog();
        let dialog_set = match DialogHolderSet::tag("minecraft:quick_actions") {
            Ok(dialog_set) => dialog_set,
            Err(err) => panic!("{err}"),
        };
        let dialog_list =
            DialogListDialog::with_defaults(common.clone(), dialog_set.clone(), None);

        assert_eq!(
            dialog_list,
            DialogListDialog {
                common: common.clone(),
                dialogs: dialog_set.clone(),
                exit_action: None,
                columns: BUTTON_LIST_DEFAULT_COLUMNS,
                button_width: DIALOG_LIST_DEFAULT_BUTTON_WIDTH,
            }
        );
        assert_eq!(dialog_list.dialog_type(), DialogType::DialogList);
        assert_eq!(dialog_list.on_cancel(), None);
        assert!(DialogListDialog::new(common.clone(), dialog_set.clone(), None, -1, 150).is_err());
        assert!(DialogListDialog::new(common.clone(), dialog_set.clone(), None, 1, 0).is_err());
        assert!(
            DialogListDialog::new(common, dialog_set, None, 1, DIALOG_WIDTH_MAX + 1).is_err()
        );
        assert!(DialogHolderSet::list(&["minecraft:server_links", "custom:menu"]).is_ok());
    }

    #[test]
    fn server_links_dialog_matches_java_columns_width_and_cancel_behavior() {
        let common = common_dialog();
        let (exit_button, exit_action) = exit_button();
        let server_links =
            ServerLinksDialog::with_defaults(common.clone(), Some(exit_button.clone()));

        assert_eq!(
            server_links,
            ServerLinksDialog {
                common: common.clone(),
                exit_action: Some(exit_button.clone()),
                columns: BUTTON_LIST_DEFAULT_COLUMNS,
                button_width: SERVER_LINKS_DEFAULT_BUTTON_WIDTH,
            }
        );
        assert_eq!(server_links.dialog_type(), DialogType::ServerLinks);
        assert_eq!(server_links.on_cancel(), Some(&exit_action));
        assert!(ServerLinksDialog::new(common.clone(), None, 0, 150).is_err());
        assert!(ServerLinksDialog::new(common.clone(), None, 1, 1025).is_err());

        let button_list = ButtonListDialogModel::ServerLinks(Box::new(server_links));
        assert_eq!(button_list.dialog_type(), DialogType::ServerLinks);
        assert_eq!(button_list.common(), &common);
        assert_eq!(button_list.columns(), BUTTON_LIST_DEFAULT_COLUMNS);
        assert_eq!(button_list.exit_action(), Some(&exit_button));
        assert_eq!(button_list.on_cancel(), Some(&exit_action));
    }

    fn common_dialog() -> CommonDialogData {
        CommonDialogData::with_defaults(Component::literal("Buttons"))
    }

    fn exit_button() -> (ActionButton, DialogActionModel) {
        let action = DialogActionModel::Static(StaticDialogAction::new(ClickEvent::RunCommand(
            "/back".to_string(),
        )));
        let button = ActionButton::new(
            CommonButtonData::with_default_width(Component::literal("Back")),
            Some(action.clone()),
        );
        (button, action)
    }

    fn action_button() -> ActionButton {
        ActionButton::new(
            CommonButtonData::with_default_width(Component::literal("Action")),
            None,
        )
    }

    #[test]
    #[cfg(vibecraft_has_decompiled_sources)]
    fn button_list_dialog_sources_match_java_26_1_2() {
        const BUTTON_LIST: &str =
            vibecraft_java_source!("/net/minecraft/server/dialog/ButtonListDialog.java");
        const MULTI_ACTION: &str =
            vibecraft_java_source!("/net/minecraft/server/dialog/MultiActionDialog.java");
        const DIALOG_LIST: &str =
            vibecraft_java_source!("/net/minecraft/server/dialog/DialogListDialog.java");
        const SERVER_LINKS: &str =
            vibecraft_java_source!("/net/minecraft/server/dialog/ServerLinksDialog.java");

        for sentinel in [
            "public interface ButtonListDialog extends Dialog",
            "MapCodec<? extends ButtonListDialog> codec();",
            "int columns();",
            "Optional<ActionButton> exitAction();",
            "return this.exitAction().flatMap(ActionButton::action);",
        ] {
            assert!(
                BUTTON_LIST.contains(sentinel),
                "ButtonListDialog.java is missing sentinel: {sentinel}"
            );
        }

        for sentinel in [
            "public record MultiActionDialog(CommonDialogData common, List<ActionButton> actions, Optional<ActionButton> exitAction, int columns) implements ButtonListDialog",
            "CommonDialogData.MAP_CODEC.forGetter(MultiActionDialog::common)",
            "ExtraCodecs.nonEmptyList(ActionButton.CODEC.listOf()).fieldOf(\"actions\").forGetter(MultiActionDialog::actions)",
            "ActionButton.CODEC.optionalFieldOf(\"exit_action\").forGetter(MultiActionDialog::exitAction)",
            "ExtraCodecs.POSITIVE_INT.optionalFieldOf(\"columns\", 2).forGetter(MultiActionDialog::columns)",
            ".apply(i, MultiActionDialog::new)",
        ] {
            assert!(
                MULTI_ACTION.contains(sentinel),
                "MultiActionDialog.java is missing sentinel: {sentinel}"
            );
        }

        for sentinel in [
            "public record DialogListDialog(CommonDialogData common, HolderSet<Dialog> dialogs, Optional<ActionButton> exitAction, int columns, int buttonWidth)",
            "CommonDialogData.MAP_CODEC.forGetter(DialogListDialog::common)",
            "Dialog.LIST_CODEC.fieldOf(\"dialogs\").forGetter(DialogListDialog::dialogs)",
            "ActionButton.CODEC.optionalFieldOf(\"exit_action\").forGetter(DialogListDialog::exitAction)",
            "ExtraCodecs.POSITIVE_INT.optionalFieldOf(\"columns\", 2).forGetter(DialogListDialog::columns)",
            "WIDTH_CODEC.optionalFieldOf(\"button_width\", 150).forGetter(DialogListDialog::buttonWidth)",
            ".apply(i, DialogListDialog::new)",
        ] {
            assert!(
                DIALOG_LIST.contains(sentinel),
                "DialogListDialog.java is missing sentinel: {sentinel}"
            );
        }

        for sentinel in [
            "public record ServerLinksDialog(CommonDialogData common, Optional<ActionButton> exitAction, int columns, int buttonWidth) implements ButtonListDialog",
            "CommonDialogData.MAP_CODEC.forGetter(ServerLinksDialog::common)",
            "ActionButton.CODEC.optionalFieldOf(\"exit_action\").forGetter(ServerLinksDialog::exitAction)",
            "ExtraCodecs.POSITIVE_INT.optionalFieldOf(\"columns\", 2).forGetter(ServerLinksDialog::columns)",
            "WIDTH_CODEC.optionalFieldOf(\"button_width\", 150).forGetter(ServerLinksDialog::buttonWidth)",
            ".apply(i, ServerLinksDialog::new)",
        ] {
            assert!(
                SERVER_LINKS.contains(sentinel),
                "ServerLinksDialog.java is missing sentinel: {sentinel}"
            );
        }
    }
}
