use std::error::Error;
use std::fmt;

use crate::chat_component::Component;

#[derive(Debug)]
pub struct ThrowingComponent {
    component: Component,
    message: String,
    cause: Option<Box<dyn Error + Send + Sync + 'static>>,
}

impl ThrowingComponent {
    pub fn new(component: Component) -> Self {
        Self::from_parts(component, None)
    }

    pub fn with_cause(component: Component, cause: impl Error + Send + Sync + 'static) -> Self {
        Self::from_parts(component, Some(Box::new(cause)))
    }

    fn from_parts(
        component: Component,
        cause: Option<Box<dyn Error + Send + Sync + 'static>>,
    ) -> Self {
        let message = component.get_string();
        Self {
            component,
            message,
            cause,
        }
    }

    pub fn get_component(&self) -> &Component {
        &self.component
    }
}

impl fmt::Display for ThrowingComponent {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl Error for ThrowingComponent {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.cause
            .as_deref()
            .map(|cause| cause as &(dyn Error + 'static))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug)]
    struct Cause;

    impl fmt::Display for Cause {
        fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            formatter.write_str("cause")
        }
    }

    impl Error for Cause {}

    #[test]
    fn throwing_component_message_matches_component_get_string() {
        let component = Component::literal("chat").append(Component::literal(" failure"));
        let error = ThrowingComponent::new(component.clone());

        assert_eq!(error.to_string(), "chat failure");
        assert_eq!(error.get_component(), &component);
        assert!(error.source().is_none());
    }

    #[test]
    fn throwing_component_preserves_cause_and_uses_component_message() {
        let error = ThrowingComponent::with_cause(Component::literal("bad command"), Cause);

        assert_eq!(error.to_string(), "bad command");
        assert_eq!(
            error.source().map(ToString::to_string),
            Some("cause".to_string())
        );
    }
}
