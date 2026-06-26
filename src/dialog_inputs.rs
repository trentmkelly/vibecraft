use crate::chat_component::{Component, ComponentArgument};

use super::{DIALOG_WIDTH_MAX, DIALOG_WIDTH_MIN, InputControlType};

pub const BOOLEAN_INPUT_DEFAULT_INITIAL: bool = false;
pub const BOOLEAN_INPUT_DEFAULT_ON_TRUE: &str = "true";
pub const BOOLEAN_INPUT_DEFAULT_ON_FALSE: &str = "false";
pub const TEXT_INPUT_DEFAULT_WIDTH: i32 = 200;
pub const TEXT_INPUT_DEFAULT_LABEL_VISIBLE: bool = true;
pub const TEXT_INPUT_DEFAULT_INITIAL: &str = "";
pub const TEXT_INPUT_DEFAULT_MAX_LENGTH: i32 = 32;
pub const TEXT_INPUT_MULTILINE_HEIGHT_MIN: i32 = 1;
pub const TEXT_INPUT_MULTILINE_MAX_HEIGHT: i32 = 512;
pub const SINGLE_OPTION_INPUT_DEFAULT_WIDTH: i32 = 200;
pub const SINGLE_OPTION_INPUT_DEFAULT_LABEL_VISIBLE: bool = true;
pub const NUMBER_RANGE_INPUT_DEFAULT_WIDTH: i32 = 200;
pub const NUMBER_RANGE_INPUT_DEFAULT_LABEL_FORMAT: &str = "options.generic_value";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BooleanInputControl {
    pub label: Component,
    pub initial: bool,
    pub on_true: String,
    pub on_false: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextInputMultilineOptions {
    pub max_lines: Option<i32>,
    pub height: Option<i32>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextInputControl {
    pub width: i32,
    pub label: Component,
    pub label_visible: bool,
    pub initial: String,
    pub max_length: i32,
    pub multiline: Option<TextInputMultilineOptions>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SingleOptionInputEntry {
    pub id: String,
    pub display: Option<Component>,
    pub initial: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SingleOptionInputControl {
    pub width: i32,
    pub entries: Vec<SingleOptionInputEntry>,
    pub label: Component,
    pub label_visible: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NumberRangeInfo {
    pub start: f32,
    pub end: f32,
    pub initial: Option<f32>,
    pub step: Option<f32>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct NumberRangeInputControl {
    pub width: i32,
    pub label: Component,
    pub label_format: String,
    pub range_info: NumberRangeInfo,
}

#[derive(Debug, Clone, PartialEq)]
pub enum InputControl {
    Boolean(BooleanInputControl),
    NumberRange(NumberRangeInputControl),
    SingleOption(SingleOptionInputControl),
    Text(TextInputControl),
}

impl BooleanInputControl {
    pub fn new(
        label: Component,
        initial: bool,
        on_true: impl Into<String>,
        on_false: impl Into<String>,
    ) -> Self {
        Self {
            label,
            initial,
            on_true: on_true.into(),
            on_false: on_false.into(),
        }
    }

    pub fn with_defaults(label: Component) -> Self {
        Self::new(
            label,
            BOOLEAN_INPUT_DEFAULT_INITIAL,
            BOOLEAN_INPUT_DEFAULT_ON_TRUE,
            BOOLEAN_INPUT_DEFAULT_ON_FALSE,
        )
    }

    pub fn input_type(&self) -> InputControlType {
        InputControlType::Boolean
    }
}

impl TextInputMultilineOptions {
    pub fn new(max_lines: Option<i32>, height: Option<i32>) -> Result<Self, String> {
        if matches!(max_lines, Some(lines) if lines <= 0) {
            return Err("text input multiline max_lines must be positive".to_string());
        }
        if matches!(height, Some(height) if !(TEXT_INPUT_MULTILINE_HEIGHT_MIN..=TEXT_INPUT_MULTILINE_MAX_HEIGHT).contains(&height)) {
            return Err(format!(
                "text input multiline height must be in Java range {TEXT_INPUT_MULTILINE_HEIGHT_MIN}..={TEXT_INPUT_MULTILINE_MAX_HEIGHT}"
            ));
        }

        Ok(Self { max_lines, height })
    }
}

impl TextInputControl {
    pub fn new(
        width: i32,
        label: Component,
        label_visible: bool,
        initial: impl Into<String>,
        max_length: i32,
        multiline: Option<TextInputMultilineOptions>,
    ) -> Result<Self, String> {
        if !(DIALOG_WIDTH_MIN..=DIALOG_WIDTH_MAX).contains(&width) {
            return Err(format!(
                "text input width {width} is outside Java Dialog.WIDTH_CODEC range {DIALOG_WIDTH_MIN}..={DIALOG_WIDTH_MAX}"
            ));
        }
        if max_length <= 0 {
            return Err("text input max_length must be positive".to_string());
        }

        let initial = initial.into();
        if java_string_length(&initial) > max_length {
            return Err("Default text length exceeds allowed size".to_string());
        }

        Ok(Self {
            width,
            label,
            label_visible,
            initial,
            max_length,
            multiline,
        })
    }

    pub fn with_defaults(label: Component) -> Self {
        Self {
            width: TEXT_INPUT_DEFAULT_WIDTH,
            label,
            label_visible: TEXT_INPUT_DEFAULT_LABEL_VISIBLE,
            initial: TEXT_INPUT_DEFAULT_INITIAL.to_string(),
            max_length: TEXT_INPUT_DEFAULT_MAX_LENGTH,
            multiline: None,
        }
    }

    pub fn input_type(&self) -> InputControlType {
        InputControlType::Text
    }
}

impl SingleOptionInputEntry {
    pub fn new(id: impl Into<String>, display: Option<Component>, initial: bool) -> Self {
        Self {
            id: id.into(),
            display,
            initial,
        }
    }

    pub fn from_id(id: impl Into<String>) -> Self {
        Self::new(id, None, false)
    }

    pub fn display_or_default(&self) -> Component {
        match &self.display {
            Some(display) => display.clone(),
            None => Component::literal(self.id.clone()),
        }
    }
}

impl SingleOptionInputControl {
    pub fn new(
        width: i32,
        entries: Vec<SingleOptionInputEntry>,
        label: Component,
        label_visible: bool,
    ) -> Result<Self, String> {
        if !(DIALOG_WIDTH_MIN..=DIALOG_WIDTH_MAX).contains(&width) {
            return Err(format!(
                "single option width {width} is outside Java Dialog.WIDTH_CODEC range {DIALOG_WIDTH_MIN}..={DIALOG_WIDTH_MAX}"
            ));
        }
        if entries.is_empty() {
            return Err("single option input requires a non-empty options list".to_string());
        }
        if entries.iter().filter(|entry| entry.initial).count() > 1 {
            return Err("Multiple initial values".to_string());
        }

        Ok(Self {
            width,
            entries,
            label,
            label_visible,
        })
    }

    pub fn with_defaults(
        entries: Vec<SingleOptionInputEntry>,
        label: Component,
    ) -> Result<Self, String> {
        Self::new(
            SINGLE_OPTION_INPUT_DEFAULT_WIDTH,
            entries,
            label,
            SINGLE_OPTION_INPUT_DEFAULT_LABEL_VISIBLE,
        )
    }

    pub fn initial(&self) -> Option<&SingleOptionInputEntry> {
        self.entries.iter().find(|entry| entry.initial)
    }

    pub fn input_type(&self) -> InputControlType {
        InputControlType::SingleOption
    }
}

impl NumberRangeInfo {
    pub fn new(
        start: f32,
        end: f32,
        initial: Option<f32>,
        step: Option<f32>,
    ) -> Result<Self, String> {
        if matches!(step, Some(step) if step <= 0.0) {
            return Err("number range step must be positive".to_string());
        }
        if let Some(initial) = initial {
            let min = start.min(end);
            let max = start.max(end);
            if initial < min || initial > max {
                return Err(format!(
                    "Initial value {initial} is outside of range [{min}, {max}]"
                ));
            }
        }

        Ok(Self {
            start,
            end,
            initial,
            step,
        })
    }

    pub fn compute_scaled_value(&self, slider_value: f32) -> f32 {
        let value_in_range = lerp(slider_value, self.start, self.end);
        let Some(step) = self.step else {
            return value_in_range;
        };

        let initial_value = self.initial_scaled_value();
        let delta_to_initial = value_in_range - initial_value;
        let steps_outside_initial = java_round_f32(delta_to_initial / step);
        let result = initial_value + steps_outside_initial as f32 * step;
        if !self.is_out_of_range(result) {
            return result;
        }

        let one_step_less = steps_outside_initial - sign_i32(steps_outside_initial);
        initial_value + one_step_less as f32 * step
    }

    pub fn initial_slider_value(&self) -> f32 {
        self.scaled_value_to_slider(self.initial_scaled_value())
    }

    fn initial_scaled_value(&self) -> f32 {
        self.initial.unwrap_or((self.start + self.end) / 2.0)
    }

    fn is_out_of_range(&self, scaled_value: f32) -> bool {
        let slider_pos = self.scaled_value_to_slider(scaled_value);
        !(0.0..=1.0).contains(&slider_pos)
    }

    fn scaled_value_to_slider(&self, value: f32) -> f32 {
        if self.start == self.end {
            0.5
        } else {
            inverse_lerp(value, self.start, self.end)
        }
    }
}

impl NumberRangeInputControl {
    pub fn new(
        width: i32,
        label: Component,
        label_format: impl Into<String>,
        range_info: NumberRangeInfo,
    ) -> Result<Self, String> {
        if !(DIALOG_WIDTH_MIN..=DIALOG_WIDTH_MAX).contains(&width) {
            return Err(format!(
                "number range width {width} is outside Java Dialog.WIDTH_CODEC range {DIALOG_WIDTH_MIN}..={DIALOG_WIDTH_MAX}"
            ));
        }

        Ok(Self {
            width,
            label,
            label_format: label_format.into(),
            range_info,
        })
    }

    pub fn with_defaults(label: Component, range_info: NumberRangeInfo) -> Self {
        Self {
            width: NUMBER_RANGE_INPUT_DEFAULT_WIDTH,
            label,
            label_format: NUMBER_RANGE_INPUT_DEFAULT_LABEL_FORMAT.to_string(),
            range_info,
        }
    }

    pub fn compute_label(&self, value: impl Into<String>) -> Component {
        Component::translatable(
            self.label_format.clone(),
            vec![
                ComponentArgument::Component(Box::new(self.label.clone())),
                ComponentArgument::String(value.into()),
            ],
        )
    }

    pub fn input_type(&self) -> InputControlType {
        InputControlType::NumberRange
    }
}

impl InputControl {
    pub fn input_type(&self) -> InputControlType {
        match self {
            Self::Boolean(input) => input.input_type(),
            Self::NumberRange(input) => input.input_type(),
            Self::SingleOption(input) => input.input_type(),
            Self::Text(input) => input.input_type(),
        }
    }
}

fn lerp(alpha: f32, p0: f32, p1: f32) -> f32 {
    p0 + alpha * (p1 - p0)
}

fn inverse_lerp(value: f32, min: f32, max: f32) -> f32 {
    (value - min) / (max - min)
}

fn java_round_f32(value: f32) -> i32 {
    if value.is_nan() {
        0
    } else if value >= i32::MAX as f32 {
        i32::MAX
    } else if value <= i32::MIN as f32 {
        i32::MIN
    } else {
        (value + 0.5).floor() as i32
    }
}

fn sign_i32(value: i32) -> i32 {
    value.signum()
}

fn java_string_length(input: &str) -> i32 {
    input.encode_utf16().count() as i32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn boolean_input_control_uses_java_fields_defaults_and_dispatch_type() {
        let default_input = BooleanInputControl::with_defaults(Component::literal("Enable"));
        assert_eq!(default_input.label, Component::literal("Enable"));
        assert!(!default_input.initial);
        assert_eq!(default_input.on_true, "true");
        assert_eq!(default_input.on_false, "false");
        assert_eq!(default_input.input_type(), InputControlType::Boolean);
        assert_eq!(
            InputControl::Boolean(default_input.clone()).input_type(),
            InputControlType::Boolean
        );

        let custom_input =
            BooleanInputControl::new(Component::literal("Mode"), true, "enabled", "disabled");
        assert_eq!(
            custom_input,
            BooleanInputControl {
                label: Component::literal("Mode"),
                initial: true,
                on_true: "enabled".to_string(),
                on_false: "disabled".to_string(),
            }
        );
    }

    #[test]
    fn text_input_control_uses_java_defaults_ranges_and_utf16_validation() {
        let default_input = TextInputControl::with_defaults(Component::literal("Name"));
        assert_eq!(
            default_input,
            TextInputControl {
                width: 200,
                label: Component::literal("Name"),
                label_visible: true,
                initial: String::new(),
                max_length: 32,
                multiline: None,
            }
        );
        assert_eq!(default_input.input_type(), InputControlType::Text);
        assert_eq!(
            InputControl::Text(default_input.clone()).input_type(),
            InputControlType::Text
        );

        assert_eq!(
            TextInputMultilineOptions::new(Some(3), Some(64)),
            Ok(TextInputMultilineOptions {
                max_lines: Some(3),
                height: Some(64),
            })
        );
        assert_eq!(
            TextInputControl::new(
                DIALOG_WIDTH_MIN,
                Component::literal("Short"),
                false,
                "ok",
                2,
                None,
            ),
            Ok(TextInputControl {
                width: 1,
                label: Component::literal("Short"),
                label_visible: false,
                initial: "ok".to_string(),
                max_length: 2,
                multiline: None,
            })
        );
        assert!(TextInputControl::new(
            DIALOG_WIDTH_MAX + 1,
            Component::empty(),
            true,
            "",
            32,
            None
        )
        .is_err());
        assert!(TextInputControl::new(200, Component::empty(), true, "", 0, None).is_err());
        assert!(TextInputControl::new(200, Component::empty(), true, "toolong", 3, None).is_err());
        assert!(TextInputControl::new(200, Component::empty(), true, "😀", 1, None).is_err());
        assert!(TextInputControl::new(200, Component::empty(), true, "😀", 2, None).is_ok());
        assert!(TextInputMultilineOptions::new(Some(0), None).is_err());
        assert!(TextInputMultilineOptions::new(None, Some(0)).is_err());
        assert!(TextInputMultilineOptions::new(None, Some(513)).is_err());
    }

    #[test]
    fn single_option_input_uses_java_defaults_validation_and_display_fallback() {
        let plain = SingleOptionInputEntry::from_id("alpha");
        let initial = SingleOptionInputEntry::new("beta", Some(Component::literal("Beta")), true);
        assert_eq!(plain.display_or_default(), Component::literal("alpha"));
        assert_eq!(initial.display_or_default(), Component::literal("Beta"));

        let input = SingleOptionInputControl::with_defaults(
            vec![plain.clone(), initial.clone()],
            Component::literal("Pick"),
        );
        assert_eq!(
            input,
            Ok(SingleOptionInputControl {
                width: 200,
                entries: vec![plain.clone(), initial.clone()],
                label: Component::literal("Pick"),
                label_visible: true,
            })
        );
        let input = match input {
            Ok(input) => input,
            Err(err) => panic!("{err}"),
        };
        assert_eq!(input.initial(), Some(&initial));
        assert_eq!(input.input_type(), InputControlType::SingleOption);
        assert_eq!(
            InputControl::SingleOption(input).input_type(),
            InputControlType::SingleOption
        );

        assert!(SingleOptionInputControl::with_defaults(Vec::new(), Component::empty()).is_err());
        assert!(SingleOptionInputControl::new(
            0,
            vec![SingleOptionInputEntry::from_id("a")],
            Component::empty(),
            true
        )
        .is_err());
        assert!(SingleOptionInputControl::with_defaults(
            vec![
                SingleOptionInputEntry::new("a", None, true),
                SingleOptionInputEntry::new("b", None, true),
            ],
            Component::empty()
        )
        .is_err());
    }

    #[test]
    fn number_range_input_uses_java_defaults_validation_and_slider_math() {
        let range = NumberRangeInfo::new(0.0, 10.0, Some(2.0), Some(5.0));
        assert_eq!(
            range,
            Ok(NumberRangeInfo {
                start: 0.0,
                end: 10.0,
                initial: Some(2.0),
                step: Some(5.0),
            })
        );
        let range = match range {
            Ok(range) => range,
            Err(err) => panic!("{err}"),
        };
        assert_close(range.initial_slider_value(), 0.2);
        assert_close(range.compute_scaled_value(1.0), 7.0);

        let unstepped = NumberRangeInfo::new(10.0, 0.0, Some(8.0), None);
        assert_eq!(
            unstepped,
            Ok(NumberRangeInfo {
                start: 10.0,
                end: 0.0,
                initial: Some(8.0),
                step: None,
            })
        );
        let unstepped = match unstepped {
            Ok(range) => range,
            Err(err) => panic!("{err}"),
        };
        assert_close(unstepped.initial_slider_value(), 0.2);
        assert_close(unstepped.compute_scaled_value(0.25), 7.5);

        let input = NumberRangeInputControl::with_defaults(Component::literal("Volume"), range);
        assert_eq!(input.width, 200);
        assert_eq!(input.label_format, "options.generic_value");
        assert_eq!(input.input_type(), InputControlType::NumberRange);
        assert_eq!(
            input.compute_label("7"),
            Component::translatable(
                "options.generic_value",
                vec![
                    ComponentArgument::Component(Box::new(Component::literal("Volume"))),
                    ComponentArgument::String("7".to_string()),
                ]
            )
        );
        assert_eq!(
            InputControl::NumberRange(input).input_type(),
            InputControlType::NumberRange
        );

        assert!(NumberRangeInfo::new(0.0, 1.0, Some(2.0), None).is_err());
        assert!(NumberRangeInfo::new(0.0, 1.0, None, Some(0.0)).is_err());
        assert!(NumberRangeInputControl::new(
            0,
            Component::empty(),
            "options.generic_value",
            range
        )
        .is_err());
        assert_close(java_round_f32(-0.5) as f32, 0.0);
    }

    #[test]
    fn input_control_dispatch_covers_all_java_input_variants() {
        let boolean = InputControl::Boolean(BooleanInputControl::with_defaults(Component::empty()));
        let range_info = match NumberRangeInfo::new(0.0, 1.0, None, None) {
            Ok(range_info) => range_info,
            Err(err) => panic!("{err}"),
        };
        let number_range = InputControl::NumberRange(NumberRangeInputControl::with_defaults(
            Component::empty(),
            range_info,
        ));
        let single_option_control = match SingleOptionInputControl::with_defaults(
            vec![SingleOptionInputEntry::from_id("only")],
            Component::empty(),
        ) {
            Ok(input) => input,
            Err(err) => panic!("{err}"),
        };
        let single_option = InputControl::SingleOption(single_option_control);
        let text = InputControl::Text(TextInputControl::with_defaults(Component::empty()));

        assert_eq!(boolean.input_type(), InputControlType::Boolean);
        assert_eq!(number_range.input_type(), InputControlType::NumberRange);
        assert_eq!(single_option.input_type(), InputControlType::SingleOption);
        assert_eq!(text.input_type(), InputControlType::Text);
    }

    #[test]
    #[cfg(vibecraft_has_decompiled_sources)]
    fn number_range_input_source_matches_java_26_1_2() {
        const NUMBER_RANGE: &str =
            vibecraft_java_source!("/net/minecraft/server/dialog/input/NumberRangeInput.java");

        for sentinel in [
            "public record NumberRangeInput(int width, Component label, String labelFormat, NumberRangeInput.RangeInfo rangeInfo) implements InputControl",
            "Dialog.WIDTH_CODEC.optionalFieldOf(\"width\", 200).forGetter(NumberRangeInput::width)",
            "ComponentSerialization.CODEC.fieldOf(\"label\").forGetter(NumberRangeInput::label)",
            "Codec.STRING.optionalFieldOf(\"label_format\", \"options.generic_value\").forGetter(NumberRangeInput::labelFormat)",
            "NumberRangeInput.RangeInfo.MAP_CODEC.forGetter(NumberRangeInput::rangeInfo)",
            "return Component.translatable(this.labelFormat, this.label, value);",
            "public record RangeInfo(float start, float end, Optional<Float> initial, Optional<Float> step)",
            "ExtraCodecs.POSITIVE_FLOAT.optionalFieldOf(\"step\").forGetter(NumberRangeInput.RangeInfo::step)",
            "if (initial < min || initial > max)",
            "float valueInRange = Mth.lerp(sliderValue, this.start, this.end);",
            "int stepsOutsideInitial = Math.round(deltaToInitial / step);",
            "int oneStepLess = stepsOutsideInitial - Mth.sign(stepsOutsideInitial);",
            "return this.start == this.end ? 0.5F : Mth.inverseLerp(value, this.start, this.end);",
        ] {
            assert!(
                NUMBER_RANGE.contains(sentinel),
                "NumberRangeInput.java is missing sentinel: {sentinel}"
            );
        }
    }

    #[test]
    #[cfg(vibecraft_has_decompiled_sources)]
    fn input_control_source_matches_java_26_1_2() {
        const INPUT_CONTROL: &str =
            vibecraft_java_source!("/net/minecraft/server/dialog/input/InputControl.java");

        for sentinel in [
            "public interface InputControl",
            "MapCodec<InputControl> MAP_CODEC = BuiltInRegistries.INPUT_CONTROL_TYPE.byNameCodec().dispatchMap(InputControl::mapCodec, c -> c);",
            "MapCodec<? extends InputControl> mapCodec();",
        ] {
            assert!(
                INPUT_CONTROL.contains(sentinel),
                "InputControl.java is missing sentinel: {sentinel}"
            );
        }
    }

    #[test]
    #[cfg(vibecraft_has_decompiled_sources)]
    fn single_option_input_source_matches_java_26_1_2() {
        const SINGLE_OPTION: &str =
            vibecraft_java_source!("/net/minecraft/server/dialog/input/SingleOptionInput.java");

        for sentinel in [
            "public record SingleOptionInput(int width, List<SingleOptionInput.Entry> entries, Component label, boolean labelVisible) implements InputControl",
            "Dialog.WIDTH_CODEC.optionalFieldOf(\"width\", 200).forGetter(SingleOptionInput::width)",
            "ExtraCodecs.nonEmptyList(SingleOptionInput.Entry.CODEC.listOf()).fieldOf(\"options\").forGetter(SingleOptionInput::entries)",
            "ComponentSerialization.CODEC.fieldOf(\"label\").forGetter(SingleOptionInput::label)",
            "Codec.BOOL.optionalFieldOf(\"label_visible\", true).forGetter(SingleOptionInput::labelVisible)",
            "long initialCount = o.entries.stream().filter(SingleOptionInput.Entry::initial).count();",
            "return initialCount > 1L ? DataResult.error(() -> \"Multiple initial values\") : DataResult.success(o);",
            "public Optional<SingleOptionInput.Entry> initial()",
            "public record Entry(String id, Optional<Component> display, boolean initial)",
            "Codec.BOOL.optionalFieldOf(\"initial\", false).forGetter(SingleOptionInput.Entry::initial)",
            "Codec.withAlternative(",
            "FULL_CODEC, Codec.STRING, id -> new SingleOptionInput.Entry(id, Optional.empty(), false)",
            "return this.display.orElseGet(() -> Component.literal(this.id));",
        ] {
            assert!(
                SINGLE_OPTION.contains(sentinel),
                "SingleOptionInput.java is missing sentinel: {sentinel}"
            );
        }
    }

    #[test]
    #[cfg(vibecraft_has_decompiled_sources)]
    fn text_input_control_source_matches_java_26_1_2() {
        const TEXT_INPUT: &str =
            vibecraft_java_source!("/net/minecraft/server/dialog/input/TextInput.java");

        for sentinel in [
            "public record TextInput(int width, Component label, boolean labelVisible, String initial, int maxLength, Optional<TextInput.MultilineOptions> multiline)",
            "Dialog.WIDTH_CODEC.optionalFieldOf(\"width\", 200).forGetter(TextInput::width)",
            "ComponentSerialization.CODEC.fieldOf(\"label\").forGetter(TextInput::label)",
            "Codec.BOOL.optionalFieldOf(\"label_visible\", true).forGetter(TextInput::labelVisible)",
            "Codec.STRING.optionalFieldOf(\"initial\", \"\").forGetter(TextInput::initial)",
            "ExtraCodecs.POSITIVE_INT.optionalFieldOf(\"max_length\", 32).forGetter(TextInput::maxLength)",
            "TextInput.MultilineOptions.CODEC.optionalFieldOf(\"multiline\").forGetter(TextInput::multiline)",
            "o.initial.length() > o.maxLength() ? DataResult.error(() -> \"Default text length exceeds allowed size\") : DataResult.success(o)",
            "public record MultilineOptions(Optional<Integer> maxLines, Optional<Integer> height)",
            "public static final int MAX_HEIGHT = 512;",
            "ExtraCodecs.POSITIVE_INT.optionalFieldOf(\"max_lines\").forGetter(TextInput.MultilineOptions::maxLines)",
            "ExtraCodecs.intRange(1, 512).optionalFieldOf(\"height\").forGetter(TextInput.MultilineOptions::height)",
        ] {
            assert!(
                TEXT_INPUT.contains(sentinel),
                "TextInput.java is missing sentinel: {sentinel}"
            );
        }
    }

    #[test]
    #[cfg(vibecraft_has_decompiled_sources)]
    fn boolean_input_control_source_matches_java_26_1_2() {
        const BOOLEAN_INPUT: &str =
            vibecraft_java_source!("/net/minecraft/server/dialog/input/BooleanInput.java");

        for sentinel in [
            "public record BooleanInput(Component label, boolean initial, String onTrue, String onFalse) implements InputControl",
            "ComponentSerialization.CODEC.fieldOf(\"label\").forGetter(BooleanInput::label)",
            "Codec.BOOL.optionalFieldOf(\"initial\", false).forGetter(BooleanInput::initial)",
            "Codec.STRING.optionalFieldOf(\"on_true\", \"true\").forGetter(BooleanInput::onTrue)",
            "Codec.STRING.optionalFieldOf(\"on_false\", \"false\").forGetter(BooleanInput::onFalse)",
            "return MAP_CODEC;",
        ] {
            assert!(
                BOOLEAN_INPUT.contains(sentinel),
                "BooleanInput.java is missing sentinel: {sentinel}"
            );
        }
    }

    fn assert_close(actual: f32, expected: f32) {
        assert!(
            (actual - expected).abs() <= f32::EPSILON * 8.0,
            "expected {actual} to be close to {expected}"
        );
    }
}
