#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommandSourceModel {
    Null,
    Custom(CustomCommandSourceModel),
}

impl CommandSourceModel {
    pub fn null() -> Self {
        Self::Null
    }

    pub fn custom(
        accepts_success: bool,
        accepts_failure: bool,
        should_inform_admins: bool,
    ) -> Self {
        Self::Custom(CustomCommandSourceModel {
            accepts_success,
            accepts_failure,
            should_inform_admins,
            always_accepts_override: None,
            messages: Vec::new(),
        })
    }

    pub fn with_always_accepts(mut self, always_accepts: bool) -> Self {
        if let Self::Custom(custom) = &mut self {
            custom.always_accepts_override = Some(always_accepts);
        }
        self
    }

    pub fn send_system_message(&mut self, message: ComponentModel) {
        match self {
            Self::Null => {}
            Self::Custom(custom) => custom.messages.push(message),
        }
    }

    pub fn accepts_success(&self) -> bool {
        match self {
            Self::Null => false,
            Self::Custom(custom) => custom.accepts_success,
        }
    }

    pub fn accepts_failure(&self) -> bool {
        match self {
            Self::Null => false,
            Self::Custom(custom) => custom.accepts_failure,
        }
    }

    pub fn should_inform_admins(&self) -> bool {
        match self {
            Self::Null => false,
            Self::Custom(custom) => custom.should_inform_admins,
        }
    }

    pub fn always_accepts(&self) -> bool {
        match self {
            Self::Null => false,
            Self::Custom(custom) => custom.always_accepts_override.unwrap_or(false),
        }
    }

    pub fn messages(&self) -> &[ComponentModel] {
        match self {
            Self::Null => &[],
            Self::Custom(custom) => &custom.messages,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CustomCommandSourceModel {
    accepts_success: bool,
    accepts_failure: bool,
    should_inform_admins: bool,
    always_accepts_override: Option<bool>,
    messages: Vec<ComponentModel>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComponentModel {
    text: &'static str,
}

impl ComponentModel {
    pub fn literal(text: &'static str) -> Self {
        Self { text }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn null_source_discards_messages_and_rejects_all_feedback() {
        let mut source = CommandSourceModel::null();

        source.send_system_message(ComponentModel::literal("ignored"));

        assert!(source.messages().is_empty());
        assert!(!source.accepts_success());
        assert!(!source.accepts_failure());
        assert!(!source.should_inform_admins());
        assert!(!source.always_accepts());
    }

    #[test]
    fn custom_source_records_system_messages() {
        let mut source = CommandSourceModel::custom(true, false, true);

        source.send_system_message(ComponentModel::literal("first"));
        source.send_system_message(ComponentModel::literal("second"));

        assert_eq!(
            source.messages(),
            [
                ComponentModel::literal("first"),
                ComponentModel::literal("second"),
            ]
        );
    }

    #[test]
    fn custom_source_reports_configured_feedback_flags() {
        let source = CommandSourceModel::custom(true, false, true);

        assert!(source.accepts_success());
        assert!(!source.accepts_failure());
        assert!(source.should_inform_admins());
    }

    #[test]
    fn always_accepts_defaults_to_false_for_implementations() {
        let source = CommandSourceModel::custom(true, true, true);

        assert!(!source.always_accepts());
    }

    #[test]
    fn implementations_can_override_always_accepts() {
        let source = CommandSourceModel::custom(false, false, false).with_always_accepts(true);

        assert!(source.always_accepts());
    }
}
