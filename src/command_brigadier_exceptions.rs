#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExceptionTypeModel {
    Simple {
        key: &'static str,
        escaped: bool,
    },
    Dynamic {
        key: &'static str,
        escaped: bool,
        arg: &'static str,
    },
    Dynamic2 {
        key: &'static str,
        escaped: bool,
        first_arg: &'static str,
        second_arg: &'static str,
    },
}

impl ExceptionTypeModel {
    pub fn escaped(&self) -> bool {
        match self {
            Self::Simple { escaped, .. }
            | Self::Dynamic { escaped, .. }
            | Self::Dynamic2 { escaped, .. } => *escaped,
        }
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct BrigadierExceptionsModel;

impl BrigadierExceptionsModel {
    pub fn double_too_low(&self) -> ExceptionTypeModel {
        dynamic2("argument.double.low", "min", "found")
    }

    pub fn double_too_high(&self) -> ExceptionTypeModel {
        dynamic2("argument.double.big", "max", "found")
    }

    pub fn float_too_low(&self) -> ExceptionTypeModel {
        dynamic2("argument.float.low", "min", "found")
    }

    pub fn float_too_high(&self) -> ExceptionTypeModel {
        dynamic2("argument.float.big", "max", "found")
    }

    pub fn integer_too_low(&self) -> ExceptionTypeModel {
        dynamic2("argument.integer.low", "min", "found")
    }

    pub fn integer_too_high(&self) -> ExceptionTypeModel {
        dynamic2("argument.integer.big", "max", "found")
    }

    pub fn long_too_low(&self) -> ExceptionTypeModel {
        dynamic2("argument.long.low", "min", "found")
    }

    pub fn long_too_high(&self) -> ExceptionTypeModel {
        dynamic2("argument.long.big", "max", "found")
    }

    pub fn literal_incorrect(&self) -> ExceptionTypeModel {
        dynamic("argument.literal.incorrect", "expected")
    }

    pub fn reader_expected_start_of_quote(&self) -> ExceptionTypeModel {
        simple("parsing.quote.expected.start")
    }

    pub fn reader_expected_end_of_quote(&self) -> ExceptionTypeModel {
        simple("parsing.quote.expected.end")
    }

    pub fn reader_invalid_escape(&self) -> ExceptionTypeModel {
        dynamic("parsing.quote.escape", "character")
    }

    pub fn reader_invalid_bool(&self) -> ExceptionTypeModel {
        dynamic("parsing.bool.invalid", "value")
    }

    pub fn reader_invalid_int(&self) -> ExceptionTypeModel {
        dynamic("parsing.int.invalid", "value")
    }

    pub fn reader_expected_int(&self) -> ExceptionTypeModel {
        simple("parsing.int.expected")
    }

    pub fn reader_invalid_long(&self) -> ExceptionTypeModel {
        dynamic("parsing.long.invalid", "value")
    }

    pub fn reader_expected_long(&self) -> ExceptionTypeModel {
        simple("parsing.long.expected")
    }

    pub fn reader_invalid_double(&self) -> ExceptionTypeModel {
        dynamic("parsing.double.invalid", "value")
    }

    pub fn reader_expected_double(&self) -> ExceptionTypeModel {
        simple("parsing.double.expected")
    }

    pub fn reader_invalid_float(&self) -> ExceptionTypeModel {
        dynamic("parsing.float.invalid", "value")
    }

    pub fn reader_expected_float(&self) -> ExceptionTypeModel {
        simple("parsing.float.expected")
    }

    pub fn reader_expected_bool(&self) -> ExceptionTypeModel {
        simple("parsing.bool.expected")
    }

    pub fn reader_expected_symbol(&self) -> ExceptionTypeModel {
        dynamic("parsing.expected", "symbol")
    }

    pub fn dispatcher_unknown_command(&self) -> ExceptionTypeModel {
        simple("command.unknown.command")
    }

    pub fn dispatcher_unknown_argument(&self) -> ExceptionTypeModel {
        simple("command.unknown.argument")
    }

    pub fn dispatcher_expected_argument_separator(&self) -> ExceptionTypeModel {
        simple("command.expected.separator")
    }

    pub fn dispatcher_parse_exception(&self) -> ExceptionTypeModel {
        dynamic("command.exception", "message")
    }
}

fn simple(key: &'static str) -> ExceptionTypeModel {
    ExceptionTypeModel::Simple {
        key,
        escaped: false,
    }
}

fn dynamic(key: &'static str, arg: &'static str) -> ExceptionTypeModel {
    ExceptionTypeModel::Dynamic {
        key,
        escaped: true,
        arg,
    }
}

fn dynamic2(
    key: &'static str,
    first_arg: &'static str,
    second_arg: &'static str,
) -> ExceptionTypeModel {
    ExceptionTypeModel::Dynamic2 {
        key,
        escaped: true,
        first_arg,
        second_arg,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn numeric_bound_exceptions_use_minecraft_keys_and_limit_then_found_arguments() {
        let provider = BrigadierExceptionsModel;

        assert_eq!(
            provider.double_too_low(),
            dynamic2("argument.double.low", "min", "found")
        );
        assert_eq!(
            provider.double_too_high(),
            dynamic2("argument.double.big", "max", "found")
        );
        assert_eq!(
            provider.float_too_low(),
            dynamic2("argument.float.low", "min", "found")
        );
        assert_eq!(
            provider.float_too_high(),
            dynamic2("argument.float.big", "max", "found")
        );
        assert_eq!(
            provider.integer_too_low(),
            dynamic2("argument.integer.low", "min", "found")
        );
        assert_eq!(
            provider.integer_too_high(),
            dynamic2("argument.integer.big", "max", "found")
        );
        assert_eq!(
            provider.long_too_low(),
            dynamic2("argument.long.low", "min", "found")
        );
        assert_eq!(
            provider.long_too_high(),
            dynamic2("argument.long.big", "max", "found")
        );
    }

    #[test]
    fn literal_and_reader_dynamic_exceptions_use_escaped_translation_components() {
        let provider = BrigadierExceptionsModel;

        assert_eq!(
            provider.literal_incorrect(),
            dynamic("argument.literal.incorrect", "expected")
        );
        assert_eq!(
            provider.reader_invalid_escape(),
            dynamic("parsing.quote.escape", "character")
        );
        assert_eq!(
            provider.reader_invalid_bool(),
            dynamic("parsing.bool.invalid", "value")
        );
        assert_eq!(
            provider.reader_invalid_int(),
            dynamic("parsing.int.invalid", "value")
        );
        assert_eq!(
            provider.reader_invalid_long(),
            dynamic("parsing.long.invalid", "value")
        );
        assert_eq!(
            provider.reader_invalid_double(),
            dynamic("parsing.double.invalid", "value")
        );
        assert_eq!(
            provider.reader_invalid_float(),
            dynamic("parsing.float.invalid", "value")
        );
        assert_eq!(
            provider.reader_expected_symbol(),
            dynamic("parsing.expected", "symbol")
        );
    }

    #[test]
    fn reader_simple_exceptions_use_plain_translation_components() {
        let provider = BrigadierExceptionsModel;

        assert_eq!(
            provider.reader_expected_start_of_quote(),
            simple("parsing.quote.expected.start")
        );
        assert_eq!(
            provider.reader_expected_end_of_quote(),
            simple("parsing.quote.expected.end")
        );
        assert_eq!(
            provider.reader_expected_int(),
            simple("parsing.int.expected")
        );
        assert_eq!(
            provider.reader_expected_long(),
            simple("parsing.long.expected")
        );
        assert_eq!(
            provider.reader_expected_double(),
            simple("parsing.double.expected")
        );
        assert_eq!(
            provider.reader_expected_float(),
            simple("parsing.float.expected")
        );
        assert_eq!(
            provider.reader_expected_bool(),
            simple("parsing.bool.expected")
        );
    }

    #[test]
    fn dispatcher_exceptions_use_minecraft_command_translation_keys() {
        let provider = BrigadierExceptionsModel;

        assert_eq!(
            provider.dispatcher_unknown_command(),
            simple("command.unknown.command")
        );
        assert_eq!(
            provider.dispatcher_unknown_argument(),
            simple("command.unknown.argument")
        );
        assert_eq!(
            provider.dispatcher_expected_argument_separator(),
            simple("command.expected.separator")
        );
        assert_eq!(
            provider.dispatcher_parse_exception(),
            dynamic("command.exception", "message")
        );
    }

    #[test]
    fn every_dynamic_exception_uses_translatable_escape() {
        let provider = BrigadierExceptionsModel;
        let dynamic_exceptions = [
            provider.double_too_low(),
            provider.double_too_high(),
            provider.float_too_low(),
            provider.float_too_high(),
            provider.integer_too_low(),
            provider.integer_too_high(),
            provider.long_too_low(),
            provider.long_too_high(),
            provider.literal_incorrect(),
            provider.reader_invalid_escape(),
            provider.reader_invalid_bool(),
            provider.reader_invalid_int(),
            provider.reader_invalid_long(),
            provider.reader_invalid_double(),
            provider.reader_invalid_float(),
            provider.reader_expected_symbol(),
            provider.dispatcher_parse_exception(),
        ];

        assert!(dynamic_exceptions.iter().all(ExceptionTypeModel::escaped));
    }
}
