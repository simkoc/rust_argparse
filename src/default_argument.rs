use crate::command_line_parsing::CommandLineParsing;
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
}

impl CommandLineParsing for DefaultArgument {
    fn help(&self) -> String {
        String::from("must not be displayed")
    }

    fn parse<'b>(
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
    fn parse_default_argument() {
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
}
