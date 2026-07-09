use crate::command_line_parsing_results::CmdParsingResults;
use std::any::Any;

pub(crate) struct DefaultArgument {
    name: String,
    value: String,
    parser: fn(&String) -> Box<dyn Any>,
}

impl DefaultArgument {
    pub(crate) fn new(
        name: String,
        value: String,
        parser: fn(&String) -> Box<dyn Any>,
    ) -> DefaultArgument {
        DefaultArgument {
            name,
            value,
            parser,
        }
    }

    #[allow(dead_code)]
    pub(crate) fn help(&self) -> String {
        String::from("must not be displayed")
    }

    pub(crate) fn parse<'b>(
        &self,
        result: &mut CmdParsingResults,
        cmdline: &'b [String],
    ) -> Result<&'b [String], String> {
        result.add_result_value(self.name.clone(), (self.parser)(&self.value));
        Ok(cmdline)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn parse_default_argument_leaves_cmdline_unchanged() {
        let cmdline = ["does".to_string(), "not".to_string(), "matter".to_string()];
        let default = DefaultArgument::new("test".to_string(), "test".to_string(), |val| {
            Box::new(val.clone())
        });
        let mut result: CmdParsingResults = CmdParsingResults::new();
        match default.parse(&mut result, &cmdline[..]) {
            Ok(remaining) => assert_eq!(cmdline, remaining),
            Err(msg) => panic!("{}", msg),
        }
    }

    #[test]
    fn parse_default_argument_stores_converted_value() {
        let cmdline: &[String] = &[];
        let default = DefaultArgument::new(
            "count".to_string(),
            "42".to_string(),
            |val| Box::new(val.parse::<i32>().expect("default value must be a number")),
        );
        let mut result: CmdParsingResults = CmdParsingResults::new();
        default.parse(&mut result, cmdline).unwrap();
        assert_eq!(*result.get_value::<i32>("count"), 42);
    }

    #[test]
    #[should_panic(expected = "default value must be a number")]
    fn parse_default_argument_panics_on_bad_conversion() {
        let cmdline: &[String] = &[];
        let default = DefaultArgument::new(
            "count".to_string(),
            "not-a-number".to_string(),
            |val| Box::new(val.parse::<i32>().expect("default value must be a number")),
        );
        let mut result: CmdParsingResults = CmdParsingResults::new();
        default.parse(&mut result, cmdline).unwrap();
    }
}
