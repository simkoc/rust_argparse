use crate::command_line_parsing::CommandLineParsing;
use crate::command_line_parsing_results::CmdParsingResults;
use std::any::Any;

pub(crate) struct PositionalArgument {
    name: String,
    doc: String,
    parser: fn(&String) -> Box<dyn Any>,
}

impl PositionalArgument {
    pub(crate) fn name(&self) -> &str {
        self.name.as_str()
    }

    pub(crate) fn new(
        name: String,
        parser: fn(&String) -> Box<dyn Any>,
        doc: String,
    ) -> PositionalArgument {
        PositionalArgument { name, parser, doc }
    }
}

impl CommandLineParsing for PositionalArgument {
    //todo this is a magic constant that needs to be unified across all help generators
    fn help(&self) -> String {
        let spaces = 20 - self.name.len();
        let spaced_name: String = "[".to_string()
            + &self.name
            + "]"
            + String::from_utf8(vec![b' '; spaces])
                .expect("should be a string of whitespaces")
                .as_str();
        spaced_name + &self.doc
    }

    fn parse<'b>(
        &self,
        result: &mut CmdParsingResults,
        cmdline: &'b [String],
    ) -> Result<&'b [String], String> {
        if cmdline.is_empty() {
            Err(format!(
                "missing required positional argument: {}",
                self.name
            ))
        } else {
            let parsed: Box<dyn Any> = (self.parser)(&cmdline[0].clone());
            result.add_result_value(self.name.clone(), parsed);
            Ok(&cmdline[1..])
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_positional_string_argument() -> Result<(), String> {
        let cmd_line: &[String] = &["pos1".to_string(), "pos2".to_string()];
        let positional: PositionalArgument = PositionalArgument::new(
            "test".to_string(),
            |x| Box::new(x.clone()),
            "test value for unit testing".to_string(),
        );
        let mut result: CmdParsingResults = CmdParsingResults::new();
        let remaining = positional.parse(&mut result, cmd_line)?;
        if remaining == ["pos2".to_string()] {
            Ok(())
        } else {
            Err(format!("bad remaining args {:?}", remaining))
        }
    }

    #[test]
    fn parse_missing_positional_argument() {
        let cmd_line: &[String] = &[];
        let positional: PositionalArgument = PositionalArgument::new(
            "test".to_string(),
            |x| Box::new(x.clone()),
            "test value for unit testing".to_string(),
        );
        let mut result: CmdParsingResults = CmdParsingResults::new();
        assert!(positional.parse(&mut result, cmd_line).is_err());
    }

    #[test]
    fn parse_converted_positional_argument() -> Result<(), String> {
        let cmd_line: &[String] = &["42".to_string()];
        let positional: PositionalArgument = PositionalArgument::new(
            "test".to_string(),
            |x| Box::new(x.parse::<i32>().unwrap()),
            "test value for unit testing".to_string(),
        );
        let mut result: CmdParsingResults = CmdParsingResults::new();
        let remaining = positional.parse(&mut result, cmd_line)?;
        if remaining.is_empty() {
            if *result.get_value::<i32>("test") == 42 {
                Ok(())
            } else {
                Err("bad result value".to_string())
            }
        } else {
            Err("did not fully consume the arguments".to_string())
        }
    }

    #[test]
    #[should_panic(expected = "this should be an int")]
    fn parse_converted_bad_positional_argument() {
        let cmd_line: &[String] = &["thisisnoint".to_string()];
        let positional: PositionalArgument = PositionalArgument::new(
            "test".to_string(),
            |x| Box::new(x.parse::<i32>().expect("this should be an int")),
            "test value for unit testing".to_string(),
        );
        let mut result: CmdParsingResults = CmdParsingResults::new();
        positional
            .parse(&mut result, cmd_line)
            .expect("expect a panic message");
    }

    #[test]
    fn proper_help_msg_line() {
        let positional: PositionalArgument = PositionalArgument::new(
            "test".to_string(),
            |x| Box::new(x.parse::<i32>().expect("this should be an int")),
            "test value for unit testing".to_string(),
        );
        assert_eq!(
            positional.help().as_str(),
            "[test]                test value for unit testing"
        );
    }
}
