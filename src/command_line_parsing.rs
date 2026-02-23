use crate::command_line_parsing_results::CmdParsingResults;

pub(crate) trait CommandLineParsing {
    fn help(&self) -> String;

    fn parse<'b>(
        &self,
        result: &mut CmdParsingResults,
        cmdline: &'b [String],
    ) -> Result<&'b [String], String>;
}
