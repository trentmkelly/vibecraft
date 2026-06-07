#![allow(dead_code)]

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JavaAnnotationRetentionModel {
    Source,
    Class,
    Runtime,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JavaElementTypeModel {
    Constructor,
    Field,
    Method,
    Type,
}

pub const SUPPRESS_FORBIDDEN_RETENTION: JavaAnnotationRetentionModel =
    JavaAnnotationRetentionModel::Class;

pub const SUPPRESS_FORBIDDEN_TARGETS: [JavaElementTypeModel; 4] = [
    JavaElementTypeModel::Constructor,
    JavaElementTypeModel::Field,
    JavaElementTypeModel::Method,
    JavaElementTypeModel::Type,
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SuppressForbiddenModel {
    reason: String,
}

impl SuppressForbiddenModel {
    pub fn new(reason: impl Into<String>) -> Self {
        Self {
            reason: reason.into(),
        }
    }

    pub fn reason(&self) -> &str {
        &self.reason
    }
}

pub fn suppress_forbidden_accepts_target(target: JavaElementTypeModel) -> bool {
    SUPPRESS_FORBIDDEN_TARGETS.contains(&target)
}

#[cfg(test)]
mod tests {
    use super::{
        suppress_forbidden_accepts_target, JavaAnnotationRetentionModel, JavaElementTypeModel,
        SuppressForbiddenModel, SUPPRESS_FORBIDDEN_RETENTION, SUPPRESS_FORBIDDEN_TARGETS,
    };

    #[test]
    fn suppress_forbidden_annotation_metadata_matches_java() {
        assert_eq!(
            SUPPRESS_FORBIDDEN_RETENTION,
            JavaAnnotationRetentionModel::Class
        );
        assert_eq!(
            SUPPRESS_FORBIDDEN_TARGETS,
            [
                JavaElementTypeModel::Constructor,
                JavaElementTypeModel::Field,
                JavaElementTypeModel::Method,
                JavaElementTypeModel::Type,
            ]
        );
    }

    #[test]
    fn suppress_forbidden_reason_element_matches_java_annotation_member() {
        let annotation = SuppressForbiddenModel::new("platform API required here");

        assert_eq!(annotation.reason(), "platform API required here");
    }

    #[test]
    fn suppress_forbidden_accepts_java_declared_targets() {
        for target in SUPPRESS_FORBIDDEN_TARGETS {
            assert!(suppress_forbidden_accepts_target(target));
        }
    }
}
