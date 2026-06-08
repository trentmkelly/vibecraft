#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FunctionInstantiationExceptionModel {
    exception_message: String,
    message_component: ComponentModel,
}

impl FunctionInstantiationExceptionModel {
    pub fn new(message_component: ComponentModel) -> Self {
        Self {
            exception_message: message_component.get_string(),
            message_component,
        }
    }

    pub fn get_message(&self) -> &str {
        &self.exception_message
    }

    pub fn message_component(&self) -> &ComponentModel {
        &self.message_component
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComponentModel {
    plain_text: String,
    style: Option<&'static str>,
}

impl ComponentModel {
    pub fn literal(plain_text: impl Into<String>) -> Self {
        Self {
            plain_text: plain_text.into(),
            style: None,
        }
    }

    pub fn with_style(mut self, style: &'static str) -> Self {
        self.style = Some(style);
        self
    }

    fn get_string(&self) -> String {
        self.plain_text.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn constructor_sets_exception_message_from_component_string() {
        let exception = FunctionInstantiationExceptionModel::new(
            ComponentModel::literal("Missing macro argument").with_style("red"),
        );

        assert_eq!(exception.get_message(), "Missing macro argument");
    }

    #[test]
    fn message_component_returns_original_component_with_style_intact() {
        let component = ComponentModel::literal("Invalid macro").with_style("yellow");
        let exception = FunctionInstantiationExceptionModel::new(component.clone());

        assert_eq!(exception.message_component(), &component);
    }
}
