use crate::chat_component::common_components::CommonComponents;
use crate::chat_component::Component;

use super::{
    DialogAction, DialogActionModel, DialogBody, DialogType, InputControl,
    COMMON_BUTTON_DEFAULT_WIDTH, DIALOG_WIDTH_MAX, DIALOG_WIDTH_MIN,
};

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
