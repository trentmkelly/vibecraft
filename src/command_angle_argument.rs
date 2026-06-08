#[derive(Debug, Clone, PartialEq)]
pub struct AngleArgumentModel;

impl AngleArgumentModel {
    pub fn angle() -> Self {
        Self
    }

    pub fn parse(
        &self,
        reader: &mut StringReaderModel,
    ) -> Result<SingleAngleModel, AngleParseError> {
        if !reader.can_read() {
            return Err(AngleParseError::NotComplete);
        }

        let is_relative = reader.read_relative_prefix();
        let value = if reader.can_read() && reader.peek() != ' ' {
            reader.read_float()?
        } else {
            0.0
        };

        if value.is_nan() || value.is_infinite() {
            Err(AngleParseError::InvalidAngle)
        } else {
            Ok(SingleAngleModel {
                angle: value,
                is_relative,
            })
        }
    }

    pub fn examples(&self) -> [&'static str; 3] {
        ["0", "~", "~-5"]
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SingleAngleModel {
    angle: f32,
    is_relative: bool,
}

impl SingleAngleModel {
    pub fn get_angle(self, sender: &CommandSourceStackModel) -> f32 {
        wrap_degrees(if self.is_relative {
            self.angle + sender.rotation_y
        } else {
            self.angle
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CommandSourceStackModel {
    pub rotation_y: f32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StringReaderModel {
    input: String,
    cursor: usize,
}

impl StringReaderModel {
    pub fn new(input: impl Into<String>) -> Self {
        Self {
            input: input.into(),
            cursor: 0,
        }
    }

    pub fn cursor(&self) -> usize {
        self.cursor
    }

    fn can_read(&self) -> bool {
        self.cursor < self.input.len()
    }

    fn peek(&self) -> char {
        self.input.as_bytes()[self.cursor] as char
    }

    fn read_relative_prefix(&mut self) -> bool {
        if self.peek() == '~' {
            self.cursor += 1;
            true
        } else {
            false
        }
    }

    fn read_float(&mut self) -> Result<f32, AngleParseError> {
        let start = self.cursor;
        while self.can_read() && self.peek() != ' ' {
            self.cursor += 1;
        }
        self.input[start..self.cursor]
            .parse::<f32>()
            .map_err(|_| AngleParseError::InvalidAngle)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AngleParseError {
    NotComplete,
    InvalidAngle,
}

fn wrap_degrees(mut degrees: f32) -> f32 {
    degrees %= 360.0;
    if degrees >= 180.0 {
        degrees -= 360.0;
    }
    if degrees < -180.0 {
        degrees += 360.0;
    }
    degrees
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(input: &str) -> Result<(SingleAngleModel, usize), AngleParseError> {
        let mut reader = StringReaderModel::new(input);
        let angle = AngleArgumentModel::angle().parse(&mut reader)?;
        Ok((angle, reader.cursor()))
    }

    #[test]
    fn examples_match_java_examples() {
        assert_eq!(AngleArgumentModel::angle().examples(), ["0", "~", "~-5"]);
    }

    #[test]
    fn parse_absolute_angle_reads_float_and_leaves_cursor_after_number() {
        let (angle, cursor) = parse("45 next").unwrap();

        assert_eq!(
            angle,
            SingleAngleModel {
                angle: 45.0,
                is_relative: false,
            }
        );
        assert_eq!(cursor, 2);
    }

    #[test]
    fn parse_relative_angle_consumes_tilde_and_value() {
        let (angle, cursor) = parse("~-5 rest").unwrap();

        assert_eq!(
            angle,
            SingleAngleModel {
                angle: -5.0,
                is_relative: true,
            }
        );
        assert_eq!(cursor, 3);
    }

    #[test]
    fn relative_angle_without_number_defaults_to_zero() {
        let (angle, cursor) = parse("~ rest").unwrap();

        assert_eq!(
            angle,
            SingleAngleModel {
                angle: 0.0,
                is_relative: true,
            }
        );
        assert_eq!(cursor, 1);
    }

    #[test]
    fn empty_input_is_incomplete() {
        assert_eq!(parse(""), Err(AngleParseError::NotComplete));
    }

    #[test]
    fn invalid_nan_and_infinite_values_are_rejected() {
        assert_eq!(parse("abc"), Err(AngleParseError::InvalidAngle));
        assert_eq!(parse("NaN"), Err(AngleParseError::InvalidAngle));
        assert_eq!(parse("inf"), Err(AngleParseError::InvalidAngle));
    }

    #[test]
    fn single_angle_get_angle_wraps_absolute_or_sender_relative_y_rotation() {
        let sender = CommandSourceStackModel { rotation_y: 170.0 };

        assert_eq!(
            SingleAngleModel {
                angle: 200.0,
                is_relative: false,
            }
            .get_angle(&sender),
            -160.0
        );
        assert_eq!(
            SingleAngleModel {
                angle: 20.0,
                is_relative: true,
            }
            .get_angle(&sender),
            -170.0
        );
    }
}
