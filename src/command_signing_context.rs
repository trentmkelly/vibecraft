use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommandSigningContextModel {
    Anonymous,
    SignedArguments(SignedArgumentsModel),
}

impl CommandSigningContextModel {
    pub fn anonymous() -> Self {
        Self::Anonymous
    }

    pub fn signed(
        arguments: impl IntoIterator<Item = (&'static str, PlayerChatMessageModel)>,
    ) -> Self {
        Self::SignedArguments(SignedArgumentsModel::new(arguments))
    }

    pub fn get_argument(&self, name: &str) -> Option<&PlayerChatMessageModel> {
        match self {
            Self::Anonymous => None,
            Self::SignedArguments(arguments) => arguments.get_argument(name),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SignedArgumentsModel {
    arguments: BTreeMap<String, PlayerChatMessageModel>,
}

impl SignedArgumentsModel {
    pub fn new(
        arguments: impl IntoIterator<Item = (&'static str, PlayerChatMessageModel)>,
    ) -> Self {
        Self {
            arguments: arguments
                .into_iter()
                .map(|(name, message)| (name.to_string(), message))
                .collect(),
        }
    }

    pub fn get_argument(&self, name: &str) -> Option<&PlayerChatMessageModel> {
        self.arguments.get(name)
    }

    pub fn arguments(&self) -> &BTreeMap<String, PlayerChatMessageModel> {
        &self.arguments
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlayerChatMessageModel {
    body: &'static str,
    signature: &'static str,
}

impl PlayerChatMessageModel {
    pub fn new(body: &'static str, signature: &'static str) -> Self {
        Self { body, signature }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn message(body: &'static str) -> PlayerChatMessageModel {
        PlayerChatMessageModel::new(body, "signature")
    }

    #[test]
    fn anonymous_context_returns_null_for_every_argument_name() {
        let context = CommandSigningContextModel::anonymous();

        assert_eq!(context.get_argument("message"), None);
        assert_eq!(context.get_argument(""), None);
    }

    #[test]
    fn signed_arguments_returns_message_for_exact_name() {
        let chat = message("hello");
        let context = CommandSigningContextModel::signed([("message", chat.clone())]);

        assert_eq!(context.get_argument("message"), Some(&chat));
    }

    #[test]
    fn signed_arguments_returns_null_for_missing_name() {
        let context = CommandSigningContextModel::signed([("message", message("hello"))]);

        assert_eq!(context.get_argument("target"), None);
    }

    #[test]
    fn signed_arguments_use_case_sensitive_string_keys() {
        let lower = message("lower");
        let upper = message("upper");
        let context = CommandSigningContextModel::signed([
            ("message", lower.clone()),
            ("Message", upper.clone()),
        ]);

        assert_eq!(context.get_argument("message"), Some(&lower));
        assert_eq!(context.get_argument("Message"), Some(&upper));
        assert_eq!(context.get_argument("MESSAGE"), None);
    }

    #[test]
    fn signed_arguments_record_preserves_argument_map() {
        let signed =
            SignedArgumentsModel::new([("first", message("one")), ("second", message("two"))]);

        assert_eq!(signed.arguments().len(), 2);
        assert_eq!(signed.get_argument("first"), Some(&message("one")));
        assert_eq!(signed.get_argument("second"), Some(&message("two")));
    }
}
