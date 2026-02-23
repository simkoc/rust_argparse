use crate::command_line_parsing::CommandLineParsing;
use crate::command_line_parsing_results::CmdParsingResults;
use std::any::Any;

pub(crate) struct OptionalArgument {
    name: String,
    long: String,
    short: char,
    default: Option<String>,
    parser: fn(&String) -> Box<dyn Any>,
    doc: String,
}

impl OptionalArgument {
    pub(crate) fn new(
        name: String,
        long: String,
        short: char,
        default: Option<String>,
        parser: fn(&String) -> Box<dyn Any>,
        doc: String,
    ) -> OptionalArgument {
        OptionalArgument {
            name,
            long,
            short,
            default,
            parser,
            doc,
        }
    }

    pub(crate) fn short(&self) -> char {
        self.short
    }

    pub(crate) fn name(&self) -> String {
        self.name.clone()
    }

    pub(crate) fn default(&self) -> Option<String> {
        self.default.clone()
    }

    pub(crate) fn parser(&self) -> fn(&String) -> Box<dyn Any> {
        self.parser
    }
}

impl CommandLineParsing for OptionalArgument {
    fn help(&self) -> String {
        let name = format!("-{},--{}", self.short, self.long);
        let spaces = 22 - name.len();
        name + String::from_utf8(vec![b' '; spaces])
            .expect("should be a string of whitespaces")
            .as_str()
            + &self.doc
    }

    fn parse<'b>(
        &self,
        result: &mut CmdParsingResults,
        cmdline: &'b [String],
    ) -> Result<&'b [String], String> {
        match cmdline.first() {
            Some(peeked) => {
                // we need at least two char for a short flag, e.g., -f
                if peeked.len() >= 2 {
                    // if the name matches
                    if peeked[2..] == self.long
                        || peeked[1..]
                            .chars()
                            .next()
                            .expect("there needs to be a char")
                            == self.short
                    {
                        return match cmdline[1..].first() {
                            // store the value
                            Some(value) => {
                                result.add_result_value(
                                    self.name.clone(),
                                    (self.parser)(&value.clone()),
                                );
                                Ok(&cmdline[2..])
                            }
                            // if there is no value panic
                            None => Err(format!("unexpected eol after {}", self.name)),
                        };
                    }
                }
                Ok(cmdline)
            }
            None => Ok(cmdline),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn get_string_optional() -> OptionalArgument {
        OptionalArgument::new(
            "test".to_string(),
            "test".to_string(),
            't',
            None,
            |val| Box::new(val.clone()),
            "test optional".to_string(),
        )
    }

    fn get_i32_optional() -> OptionalArgument {
        OptionalArgument::new(
            "test".to_string(),
            "test".to_string(),
            't',
            None,
            |val| Box::new(val.parse::<i32>().expect("there should be a number")),
            "test optional".to_string(),
        )
    }

    #[test]
    fn parse_optional_argument_long() -> Result<(), String> {
        let cmdline: &[String] = &[
            "--test".to_string(),
            "value".to_string(),
            "chaff".to_string(),
        ];
        let mut result: CmdParsingResults = CmdParsingResults::new();
        let optional: OptionalArgument = get_string_optional();
        let remaining = optional.parse(&mut result, cmdline)?;
        if remaining == ["chaff".to_string()] {
            if result.get_value::<String>(&*optional.name) == "value" {
                Ok(())
            } else {
                Err("did not extract value as value".to_string())
            }
        } else {
            Err("consumed too much".to_string())
        }
    }

    #[test]
    fn parse_optional_argument_short() -> Result<(), String> {
        let cmdline: &[String] = &["-t".to_string(), "value".to_string(), "chaff".to_string()];
        let mut result: CmdParsingResults = CmdParsingResults::new();
        let optional: OptionalArgument = get_string_optional();
        let remaining = optional.parse(&mut result, cmdline)?;
        if remaining == ["chaff".to_string()] {
            if result.get_value::<String>(&*optional.name) == "value" {
                Ok(())
            } else {
                Err("did not extract value as value".to_string())
            }
        } else {
            Err(format!(
                "wrong consumption of cmd line. It remains {:?}",
                remaining
            ))
        }
    }

    #[test]
    fn parse_optional_argument_converted() -> Result<(), String> {
        let cmdline: &[String] = &["-t".to_string(), "42".to_string(), "chaff".to_string()];
        let mut result: CmdParsingResults = CmdParsingResults::new();
        let optional: OptionalArgument = get_i32_optional();
        let remaining = optional.parse(&mut result, cmdline)?;
        if remaining == ["chaff".to_string()] {
            if *result.get_value::<i32>(&*optional.name) == 42 {
                Ok(())
            } else {
                Err("did not extract 42 as value".to_string())
            }
        } else {
            Err(format!(
                "wrong consumption of cmd line. It remains {:?}",
                remaining
            ))
        }
    }

    #[test]
    #[should_panic(expected = "there should be a number")]
    fn parse_optional_argument_converted_bad() {
        let cmdline: &[String] = &["-t".to_string(), "test".to_string(), "chaff".to_string()];
        let mut result: CmdParsingResults = CmdParsingResults::new();
        let optional: OptionalArgument = get_i32_optional();
        optional
            .parse(&mut result, cmdline)
            .expect("expect a panic message");
    }

    #[test]
    fn parse_optional_argument_missing() {
        let cmdline: &[String] = &["-t".to_string()];
        let mut result: CmdParsingResults = CmdParsingResults::new();
        let optional: OptionalArgument = get_string_optional();
        assert!(optional.parse(&mut result, cmdline).is_err());
    }

    #[test]
    fn proper_help_msg_line() {
        let optional: OptionalArgument = get_string_optional();
        assert_eq!(optional.help(), "-t,--test             test optional")
    }
}
