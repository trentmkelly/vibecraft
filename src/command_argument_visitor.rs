use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseResultsModel {
    context: CommandContextBuilderModel,
}

impl ParseResultsModel {
    pub fn new(context: CommandContextBuilderModel) -> Self {
        Self { context }
    }

    fn context(&self) -> &CommandContextBuilderModel {
        &self.context
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandContextBuilderModel {
    id: &'static str,
    root_node: &'static str,
    arguments: BTreeMap<String, ParsedArgumentModel>,
    nodes: Vec<ParsedCommandNodeModel>,
    child: Option<Box<CommandContextBuilderModel>>,
}

impl CommandContextBuilderModel {
    pub fn new(id: &'static str, root_node: &'static str) -> Self {
        Self {
            id,
            root_node,
            arguments: BTreeMap::new(),
            nodes: Vec::new(),
            child: None,
        }
    }

    pub fn with_argument(mut self, name: &str, value: &str) -> Self {
        self.arguments
            .insert(name.to_string(), ParsedArgumentModel::new(value));
        self
    }

    pub fn with_node(mut self, node: ParsedCommandNodeModel) -> Self {
        self.nodes.push(node);
        self
    }

    pub fn with_child(mut self, child: CommandContextBuilderModel) -> Self {
        self.child = Some(Box::new(child));
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParsedCommandNodeModel {
    Literal { name: String },
    Argument { name: String },
}

impl ParsedCommandNodeModel {
    pub fn literal(name: &str) -> Self {
        Self::Literal {
            name: name.to_string(),
        }
    }

    pub fn argument(name: &str) -> Self {
        Self::Argument {
            name: name.to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedArgumentModel {
    value: String,
}

impl ParsedArgumentModel {
    pub fn new(value: &str) -> Self {
        Self {
            value: value.to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArgumentVisitModel {
    pub context_id: &'static str,
    pub argument_name: String,
    pub value: Option<ParsedArgumentModel>,
}

pub struct ArgumentVisitorModel;

impl ArgumentVisitorModel {
    pub fn visit_arguments(
        command: &ParseResultsModel,
        output: &mut impl ArgumentVisitorOutputModel,
        reject_root_redirects: bool,
    ) {
        let root_context = command.context();
        let mut context = root_context;
        Self::visit_node_arguments(context, output);

        while let Some(child) = context.child.as_deref() {
            if reject_root_redirects && child.root_node == root_context.root_node {
                break;
            }

            Self::visit_node_arguments(child, output);
            context = child;
        }
    }

    fn visit_node_arguments(
        context: &CommandContextBuilderModel,
        output: &mut impl ArgumentVisitorOutputModel,
    ) {
        for node in &context.nodes {
            if let ParsedCommandNodeModel::Argument { name } = node {
                let value = context.arguments.get(name).cloned();
                output.accept(context, name, value);
            }
        }
    }
}

pub trait ArgumentVisitorOutputModel {
    fn accept(
        &mut self,
        context: &CommandContextBuilderModel,
        argument_name: &str,
        value: Option<ParsedArgumentModel>,
    );
}

#[derive(Debug, Default, PartialEq, Eq)]
pub struct RecordingOutputModel {
    visits: Vec<ArgumentVisitModel>,
}

impl RecordingOutputModel {
    pub fn visits(&self) -> &[ArgumentVisitModel] {
        &self.visits
    }
}

impl ArgumentVisitorOutputModel for RecordingOutputModel {
    fn accept(
        &mut self,
        context: &CommandContextBuilderModel,
        argument_name: &str,
        value: Option<ParsedArgumentModel>,
    ) {
        self.visits.push(ArgumentVisitModel {
            context_id: context.id,
            argument_name: argument_name.to_string(),
            value,
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn argument_visit(
        context_id: &'static str,
        argument_name: &str,
        value: Option<&str>,
    ) -> ArgumentVisitModel {
        ArgumentVisitModel {
            context_id,
            argument_name: argument_name.to_string(),
            value: value.map(ParsedArgumentModel::new),
        }
    }

    fn run(command: ParseResultsModel, reject_root_redirects: bool) -> RecordingOutputModel {
        let mut output = RecordingOutputModel::default();
        ArgumentVisitorModel::visit_arguments(&command, &mut output, reject_root_redirects);
        output
    }

    #[test]
    fn visits_argument_nodes_in_context_node_order_and_skips_literals() {
        let context = CommandContextBuilderModel::new("root", "root-node")
            .with_argument("target", "Steve")
            .with_argument("count", "3")
            .with_node(ParsedCommandNodeModel::literal("give"))
            .with_node(ParsedCommandNodeModel::argument("target"))
            .with_node(ParsedCommandNodeModel::literal("item"))
            .with_node(ParsedCommandNodeModel::argument("count"));

        let output = run(ParseResultsModel::new(context), false);

        assert_eq!(
            output.visits(),
            [
                argument_visit("root", "target", Some("Steve")),
                argument_visit("root", "count", Some("3")),
            ]
        );
    }

    #[test]
    fn missing_argument_value_is_forwarded_as_null() {
        let context = CommandContextBuilderModel::new("root", "root-node")
            .with_node(ParsedCommandNodeModel::argument("missing"));

        let output = run(ParseResultsModel::new(context), false);

        assert_eq!(output.visits(), [argument_visit("root", "missing", None)]);
    }

    #[test]
    fn visits_child_context_chain_when_root_redirects_are_allowed() {
        let grandchild = CommandContextBuilderModel::new("grandchild", "grandchild-root")
            .with_argument("function", "minecraft:test")
            .with_node(ParsedCommandNodeModel::argument("function"));
        let child = CommandContextBuilderModel::new("child", "child-root")
            .with_argument("targets", "@s")
            .with_node(ParsedCommandNodeModel::argument("targets"))
            .with_child(grandchild);
        let root = CommandContextBuilderModel::new("root", "root-node")
            .with_argument("command", "execute")
            .with_node(ParsedCommandNodeModel::argument("command"))
            .with_child(child);

        let output = run(ParseResultsModel::new(root), false);

        assert_eq!(
            output.visits(),
            [
                argument_visit("root", "command", Some("execute")),
                argument_visit("child", "targets", Some("@s")),
                argument_visit("grandchild", "function", Some("minecraft:test")),
            ]
        );
    }

    #[test]
    fn reject_root_redirects_stops_before_child_with_same_root_node_as_original_root() {
        let redirected_child = CommandContextBuilderModel::new("redirected", "root-node")
            .with_argument("again", "seed")
            .with_node(ParsedCommandNodeModel::argument("again"));
        let root = CommandContextBuilderModel::new("root", "root-node")
            .with_argument("command", "execute")
            .with_node(ParsedCommandNodeModel::argument("command"))
            .with_child(redirected_child);

        let output = run(ParseResultsModel::new(root), true);

        assert_eq!(
            output.visits(),
            [argument_visit("root", "command", Some("execute"))]
        );
    }

    #[test]
    fn reject_root_redirects_still_visits_child_with_different_root_node() {
        let child = CommandContextBuilderModel::new("child", "child-root")
            .with_argument("targets", "@a")
            .with_node(ParsedCommandNodeModel::argument("targets"));
        let root = CommandContextBuilderModel::new("root", "root-node")
            .with_argument("command", "execute")
            .with_node(ParsedCommandNodeModel::argument("command"))
            .with_child(child);

        let output = run(ParseResultsModel::new(root), true);

        assert_eq!(
            output.visits(),
            [
                argument_visit("root", "command", Some("execute")),
                argument_visit("child", "targets", Some("@a")),
            ]
        );
    }
}
