#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct InputPredicateModel {
    pub forward: Option<bool>,
    pub backward: Option<bool>,
    pub left: Option<bool>,
    pub right: Option<bool>,
    pub jump: Option<bool>,
    pub sneak: Option<bool>,
    pub sprint: Option<bool>,
}

impl InputPredicateModel {
    pub fn new(
        forward: Option<bool>,
        backward: Option<bool>,
        left: Option<bool>,
        right: Option<bool>,
        jump: Option<bool>,
        sneak: Option<bool>,
        sprint: Option<bool>,
    ) -> Self {
        Self {
            forward,
            backward,
            left,
            right,
            jump,
            sneak,
            sprint,
        }
    }

    pub fn matches(&self, input: &InputModel) -> bool {
        matches_optional(self.forward, input.forward)
            && matches_optional(self.backward, input.backward)
            && matches_optional(self.left, input.left)
            && matches_optional(self.right, input.right)
            && matches_optional(self.jump, input.jump)
            && matches_optional(self.sneak, input.shift)
            && matches_optional(self.sprint, input.sprint)
    }
}

fn matches_optional(expected: Option<bool>, value: bool) -> bool {
    expected.is_none_or(|expected| expected == value)
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct InputModel {
    forward: bool,
    backward: bool,
    left: bool,
    right: bool,
    jump: bool,
    shift: bool,
    sprint: bool,
}

impl InputModel {
    pub fn new(
        forward: bool,
        backward: bool,
        left: bool,
        right: bool,
        jump: bool,
        shift: bool,
        sprint: bool,
    ) -> Self {
        Self {
            forward,
            backward,
            left,
            right,
            jump,
            shift,
            sprint,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input(
        forward: bool,
        backward: bool,
        left: bool,
        right: bool,
        jump: bool,
        shift: bool,
        sprint: bool,
    ) -> InputModel {
        InputModel::new(forward, backward, left, right, jump, shift, sprint)
    }

    #[test]
    fn omitted_fields_match_any_input_values() {
        let predicate = InputPredicateModel::default();

        assert!(predicate.matches(&input(false, false, false, false, false, false, false)));
        assert!(predicate.matches(&input(true, true, true, true, true, true, true)));
    }

    #[test]
    fn explicit_true_and_false_values_must_equal_input() {
        let predicate = InputPredicateModel::new(
            Some(true),
            Some(false),
            Some(true),
            Some(false),
            Some(true),
            Some(false),
            Some(true),
        );

        assert!(predicate.matches(&input(true, false, true, false, true, false, true)));
        assert!(!predicate.matches(&input(false, false, true, false, true, false, true)));
        assert!(!predicate.matches(&input(true, true, true, false, true, false, true)));
        assert!(!predicate.matches(&input(true, false, false, false, true, false, true)));
        assert!(!predicate.matches(&input(true, false, true, true, true, false, true)));
        assert!(!predicate.matches(&input(true, false, true, false, false, false, true)));
        assert!(!predicate.matches(&input(true, false, true, false, true, true, true)));
        assert!(!predicate.matches(&input(true, false, true, false, true, false, false)));
    }

    #[test]
    fn sneak_field_matches_input_shift_accessor() {
        let sneaking = InputPredicateModel::new(None, None, None, None, None, Some(true), None);
        let not_sneaking =
            InputPredicateModel::new(None, None, None, None, None, Some(false), None);

        assert!(sneaking.matches(&input(false, false, false, false, false, true, false)));
        assert!(!sneaking.matches(&input(false, false, false, false, false, false, false)));
        assert!(not_sneaking.matches(&input(false, false, false, false, false, false, false)));
    }

    #[test]
    fn sprint_field_matches_input_sprint_accessor() {
        let sprinting = InputPredicateModel::new(None, None, None, None, None, None, Some(true));

        assert!(sprinting.matches(&input(false, false, false, false, false, false, true)));
        assert!(!sprinting.matches(&input(false, false, false, false, false, false, false)));
    }
}
