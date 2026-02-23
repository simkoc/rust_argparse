use crate::command_line_parsing::CommandLineParsing;
use crate::command_line_parsing_results::CmdParsingResults;

pub(crate) struct FlagArgument {
    name: String,
    long: String,
    short: char,
    doc: String,
}

impl FlagArgument {
    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    pub fn short(&self) -> char {
        self.short
    }

    pub(crate) fn new(name: String, long: String, short: char, doc: String) -> FlagArgument {
        FlagArgument {
            name,
            long,
            short,
            doc,
        }
    }
}

impl CommandLineParsing for FlagArgument {
    fn help(&self) -> String {
        let name = format!("-{},--{}", self.short, self.long);
        let spaces = 22 - name.len();
        name + String::from_utf8(vec![b' '; spaces])
            .expect("should be a string of whitespaces")
            .as_str()
            + &self.doc
    }

    fn parse<'a>(
        &self,
        result: &mut CmdParsingResults,
        cmdline: &'a [String],
    ) -> Result<&'a [String], String> {
        match cmdline.first() {
            Some(peeked_name) => {
                // a flag needs at least two chars, e.g. -f
                if peeked_name.len() >= 2 {
                    // if the name matches either long or short
                    if peeked_name[2..] == self.long
                        || peeked_name[1..]
                            .chars()
                            .next()
                            .expect("there needs to be a char")
                            == self.short
                    {
                        // add the true value to the results
                        result.add_result_value(self.name.clone(), Box::new(true));
                        return Ok(&cmdline[1..]);
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

    fn get_flag() -> FlagArgument {
        FlagArgument::new(
            "test".to_string(),
            "test".to_string(),
            't',
            "test flag".to_string(),
        )
    }

    #[test]
    fn parse_optional_argument_long() -> Result<(), String> {
        let cmdline: &[String] = &["--test".to_string(), "chaff".to_string()];
        let mut result: CmdParsingResults = CmdParsingResults::new();
        let optional: FlagArgument = get_flag();
        let remaining = optional.parse(&mut result, cmdline)?;
        if remaining == ["chaff".to_string()] {
            if result.get_flag(&*optional.name) {
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
        let cmdline: &[String] = &["-t".to_string(), "chaff".to_string()];
        let mut result: CmdParsingResults = CmdParsingResults::new();
        let optional: FlagArgument = get_flag();
        let remaining = optional.parse(&mut result, cmdline)?;
        if remaining == ["chaff".to_string()] {
            if result.get_flag(&*optional.name) {
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
    fn proper_help_msg_line() {
        let optional: FlagArgument = get_flag();
        assert_eq!(optional.help(), "-t,--test             test flag")
    }
}
