mod command_line_parsing;
pub mod command_line_parsing_results;
mod default_argument;
mod flag_argument;
mod optional_argument;
mod positional_argument;

use crate::command_line_parsing::CommandLineParsing;
use crate::command_line_parsing_results::CmdParsingResults;
use crate::default_argument::DefaultArgument;
use crate::flag_argument::FlagArgument;
use crate::optional_argument::OptionalArgument;
use crate::positional_argument::PositionalArgument;
use std::any::Any;
use std::cell::RefCell;
use std::env;

pub struct Parser {
    name: String,
    doc: String,
    defaults: Vec<DefaultArgument>,
    actions: Vec<Parser>,
    positionals: Vec<PositionalArgument>,
    optionals: Vec<OptionalArgument>,
    flags: Vec<FlagArgument>,
    main: RefCell<Option<Box<dyn FnOnce(&CmdParsingResults) -> Result<(), String>>>>,
}

impl Parser {
    pub fn new(name: &str, doc: &str) -> Parser {
        Parser {
            name: name.to_string(),
            doc: doc.to_string(),
            defaults: Vec::new(),
            actions: Vec::new(),
            positionals: Vec::new(),
            optionals: Vec::new(),
            flags: Vec::new(),
            main: RefCell::new(None),
        }
    }

    pub fn with_main<F: FnOnce(&CmdParsingResults) -> Result<(), String> + 'static>(
        self,
        f: F,
    ) -> Parser {
        *self.main.borrow_mut() = Some(Box::new(f));
        self
    }

    #[allow(unused)]
    pub fn add_action(mut self, parser: Parser) -> Parser {
        self.actions.push(parser);
        self
    }

    #[allow(unused)]
    pub fn add_default(self, name: String, value: String) -> Parser {
        self.add_parsed_default(name, value, |val: &String| Box::new(val.clone()))
    }

    #[allow(unused)]
    pub fn add_parsed_default(
        mut self,
        name: String,
        value: String,
        parser: fn(&String) -> Box<dyn Any>,
    ) -> Parser {
        self.defaults
            .push(DefaultArgument::new(name, value, parser));
        self
    }

    pub fn add_positional(self, name: &str, doc: &str) -> Parser {
        self.add_parsed_positional(name, |val: &String| Box::new(val.clone()), doc)
    }

    pub fn add_parsed_positional(
        mut self,
        name: &str,
        parser: fn(&String) -> Box<dyn Any>,
        doc: &str,
    ) -> Parser {
        self.positionals.push(PositionalArgument::new(
            name.to_string(),
            parser,
            doc.to_string(),
        ));
        self
    }

    #[allow(unused)]
    pub fn add_optional(
        self,
        name: &str,
        long: &str,
        short: char,
        default: Option<&str>,
        doc: &str,
    ) -> Parser {
        self.add_parsed_optional(name, long, short, default, |val| Box::new(val.clone()), doc)
    }

    pub fn add_parsed_optional(
        mut self,
        name: &str,
        long: &str,
        short: char,
        default: Option<&str>,
        parser: fn(&String) -> Box<dyn Any>,
        doc: &str,
    ) -> Parser {
        let conv_default = match default {
            Some(str) => Some(str.to_string()),
            None => None,
        };
        self.optionals.push(OptionalArgument::new(
            name.to_string(),
            long.to_string(),
            short,
            conv_default,
            parser,
            doc.to_string(),
        ));
        self
    }

    #[allow(unused)]
    pub fn add_flag(mut self, name: &str, long: &str, short: char, doc: &str) -> Parser {
        self.flags.push(FlagArgument::new(
            name.to_string(),
            long.to_string(),
            short,
            doc.to_string(),
        ));
        self
    }

    pub fn parse_cmdline(&self) -> Result<CmdParsingResults, String> {
        let arg_slice = env::args().collect::<Vec<String>>();
        self.parse(arg_slice[1..].to_vec())
    }

    pub fn parse(&self, cmdline_args: Vec<String>) -> Result<CmdParsingResults, String> {
        let mut result = CmdParsingResults::new();
        result.set_action(self.name.clone());
        //println!("{} parsing remaining: {}",self.name, cmdline_args.join(" "));
        match CommandLineParsing::parse(self, &mut result, &cmdline_args[..]) {
            Ok(remaining) => {
                if remaining.is_empty() {
                    Ok(result)
                } else {
                    Err(format!(
                        "Too many cmd arguments after: {:?} \n\n {}",
                        remaining,
                        self.help()
                    ))
                }
            }
            Err(msg) => Err(msg),
        }
    }

    fn check_for_help(&self, cmdline: &[String]) -> Result<(), String> {
        if !cmdline.is_empty() {
            match cmdline.first().expect("remaining is not empty").as_str() {
                "--help" | "-h" => Err(self.help()),
                _ => Ok(()),
            }
        } else {
            Ok(())
        }
    }

    fn parse_default_arguments<'b>(
        &self,
        result: &mut CmdParsingResults,
        cmdline: &'b [String],
    ) -> Result<&'b [String], String> {
        let mut remaining_cmd_line = cmdline;
        for (_, item) in self.defaults.iter().enumerate() {
            match item.parse(result, remaining_cmd_line) {
                Ok(remains) => remaining_cmd_line = remains,
                Err(e) => return Err(format!("Bad Cmd Arguments: {} \n\n {}", e, self.help())),
            }
        }
        Ok(remaining_cmd_line)
    }

    fn parse_positional_arguments<'b>(
        &self,
        result: &mut CmdParsingResults,
        cmdline: &'b [String],
    ) -> Result<&'b [String], String> {
        let mut remaining_cmd_line = cmdline;
        for (_, item) in self.positionals.iter().enumerate() {
            self.check_for_help(remaining_cmd_line)?;
            match item.parse(result, remaining_cmd_line) {
                Ok(remains) => remaining_cmd_line = remains,
                Err(e) => return Err(format!("Bad Cmd Arguments: {} \n\n {}", e, self.help())),
            }
        }
        Ok(remaining_cmd_line)
    }

    fn parse_optional_arguments<'b>(
        &self,
        result: &mut CmdParsingResults,
        cmdline: &'b [String],
    ) -> Result<&'b [String], String> {
        let mut remaining_cmd_line = cmdline;
        // parse the command line content
        for (_, item) in self.optionals.iter().enumerate() {
            self.check_for_help(remaining_cmd_line)?;
            match item.parse(result, remaining_cmd_line) {
                Ok(remains) => remaining_cmd_line = remains,
                Err(e) => return Err(format!("Bad Cmd Arguments: {} \n\n {}", e, self.help())),
            }
        }
        // add default args (if exist) for all not added optional arguments
        for (_, item) in self.optionals.iter().enumerate() {
            if !result.keys().contains(&&item.name()) {
                match item.default() {
                    Some(default) => result.add_result_value(item.name(), item.parser()(&default)),
                    None => {}
                }
            }
        }
        Ok(remaining_cmd_line)
    }

    fn parse_flag_arguments<'b>(
        &self,
        result: &mut CmdParsingResults,
        cmdline: &'b [String],
    ) -> Result<&'b [String], String> {
        let mut remaining_cmd_line = cmdline;
        // parse the command line content
        for (_, item) in self.flags.iter().enumerate() {
            self.check_for_help(remaining_cmd_line)?;
            match item.parse(result, remaining_cmd_line) {
                Ok(remains) => remaining_cmd_line = remains,
                Err(e) => return Err(format!("Bad Cmd Arguments: {} \n\n {}", e, self.help())),
            }
        }
        // add false for all not added flags
        for (_, item) in self.flags.iter().enumerate() {
            if !result.keys().contains(&&item.name().to_string()) {
                result.add_result_value(item.name().to_string(), Box::new(false))
            }
        }
        Ok(remaining_cmd_line)
    }

    fn find_matching_action(&self, name: &str) -> Option<&Parser> {
        self.actions.iter().find(|action| action.name == name)
    }

    fn parse_action_arguments<'b>(
        &self,
        result: &mut CmdParsingResults,
        cmdline: &'b [String],
    ) -> Result<&'b [String], String> {
        if self.actions.is_empty() {
            let main = self
                .main
                .borrow_mut()
                .take()
                .unwrap_or_else(|| panic!("leaf parser '{}' has no main function", self.name));
            result.set_main(main);
            return Ok(cmdline);
        }
        let action_name = cmdline
            .first()
            .ok_or_else(|| format!("You have to chose an action. \n\n {}", self.help()))?;
        let action = self
            .find_matching_action(action_name)
            .ok_or_else(|| format!("Unknown action {}", action_name))?;
        let remaining = CommandLineParsing::parse(action, result, &cmdline[1..])?;
        if remaining.is_empty() {
            Ok(remaining)
        } else {
            Err(format!(
                "Too many supplied arguments after: {:?}\n\n{}",
                remaining,
                self.help()
            ))
        }
    }
}

impl Parser {
    fn build_usage_line(&self) -> String {
        let mut usage = "usage: ".to_string() + self.name.as_str();
        for positional in self.positionals.iter() {
            usage += &format!(" [{}]", positional.name());
        }
        if !self.optionals.is_empty() || !self.flags.is_empty() {
            usage += " {";
            let shorts: Vec<String> = self
                .optionals
                .iter()
                .map(|o| format!("-{}", o.short()))
                .chain(self.flags.iter().map(|f| format!("-{}", f.short())))
                .collect();
            usage += &shorts.join(",");
            usage += "}";
        }
        for (num, action) in self.actions.iter().enumerate() {
            if num != 0 {
                usage += ",";
            } else {
                usage += " ";
            }
            usage += action.name.as_str();
        }
        usage.trim().to_string()
    }

    fn build_help_body(&self) -> String {
        let mut body = String::new();
        for positional in self.positionals.iter() {
            body += &positional.help();
            body += "\n";
        }
        for optional in self.optionals.iter() {
            body += &optional.help();
            body += "\n";
        }
        for flag in self.flags.iter() {
            body += &flag.help();
            body += "\n";
        }
        for action in self.actions.iter() {
            let name = action.name.as_str();
            let spaces = 22 - name.len();
            body += name;
            body += &String::from_utf8(vec![b' '; spaces])
                .expect("should be a string of whitespaces");
            body += action.doc.as_str();
            body += "\n";
        }
        body
    }
}

impl CommandLineParsing for Parser {
    fn help(&self) -> String {
        let header = self.name.clone() + " - " + self.doc.as_str();
        header + "\n\n" + &self.build_usage_line() + "\n\n" + &self.build_help_body()
    }

    fn parse<'b>(
        &self,
        result: &mut CmdParsingResults,
        cmdline: &'b [String],
    ) -> Result<&'b [String], String> {
        result.set_action(self.name.clone());
        // add the defaults of this parser to the command line
        let mut remaining_cmd_line: &[String] = cmdline;
        // first parse default arguments
        match self.parse_default_arguments(result, remaining_cmd_line) {
            Ok(remaining) => remaining_cmd_line = remaining,
            Err(e) => return Err(e),
        }
        // parse the positional arguments
        match self.parse_positional_arguments(result, &remaining_cmd_line) {
            Ok(remaining) => remaining_cmd_line = remaining,
            Err(e) => return Err(e),
        }
        // parse the optional arguments
        match self.parse_optional_arguments(result, &remaining_cmd_line) {
            Ok(remaining) => remaining_cmd_line = remaining,
            Err(e) => return Err(e),
        }
        // parse the flags arguments
        match self.parse_flag_arguments(result, &remaining_cmd_line) {
            Ok(remaining) => remaining_cmd_line = remaining,
            Err(e) => return Err(e),
        }
        // parse the action arguments
        match self.parse_action_arguments(result, &remaining_cmd_line) {
            Ok(remaining) => remaining_cmd_line = remaining,
            Err(e) => return Err(e),
        }
        //todo: when I reach this remaining should be empty
        Ok(remaining_cmd_line)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn stub_main(_: &CmdParsingResults) -> Result<(), String> {
        Ok(())
    }

    fn get_basic_cmd_parser() -> Parser {
        Parser::new("test", "I am a test")
            .add_default("default".to_string(), "test".to_string())
            .add_positional("positional", "I am the positional")
            .add_optional(
                "optional",
                "optional",
                'o',
                Some("default"),
                "I am the optional",
            )
            .add_flag("flag", "flag", 'f', "I am the flag")
            .with_main(stub_main)
    }

    fn get_nested_parser() -> Parser {
        get_basic_cmd_parser().add_action(
            Parser::new("compute", "I am da computaaah")
                .add_positional("stuff", "stuff indeed")
                .with_main(stub_main),
        )
    }

    #[test]
    fn parse_command_line_wrong_action() {
        let args: &[String] = &[
            "positional".to_string(),
            "-o".to_string(),
            "optional".to_string(),
            "-f".to_string(),
            "wrong-action".to_string(),
        ];
        let parser: Parser = get_nested_parser();
        assert!(parser.parse(Vec::from(args)).is_err())
    }

    #[test]
    fn parse_command_line_missing_action() {
        let args: &[String] = &[
            "positional".to_string(),
            "-o".to_string(),
            "optional".to_string(),
            "-f".to_string(),
        ];
        let parser: Parser = get_nested_parser();
        assert!(parser.parse(Vec::from(args)).is_err())
    }

    #[test]
    fn parse_command_line_full_action() {
        let args: &[String] = &[
            "positional".to_string(),
            "-o".to_string(),
            "optional".to_string(),
            "-f".to_string(),
        ];
        let parser: Parser = get_basic_cmd_parser();
        match parser.parse(Vec::from(args)) {
            Ok(result) => {
                assert_eq!(result.get_action(), "test");
                assert_eq!(result.get_value::<String>("positional"), "positional");
                assert_eq!(result.get_value::<String>("optional"), "optional");
                assert!(result.get_flag("flag"));
            }
            Err(e) => panic!("{}", e),
        }
    }

    #[test]
    fn parse_command_line_full_sub_action() {
        let args: &[String] = &[
            "positional".to_string(),
            "-o".to_string(),
            "optional".to_string(),
            "-f".to_string(),
            "compute".to_string(),
            "values".to_string(),
        ];
        let parser: Parser = get_nested_parser();
        match parser.parse(Vec::from(args)) {
            Ok(result) => {
                assert_eq!(result.get_action(), "compute");
                assert_eq!(result.get_value::<String>("positional"), "positional");
                assert_eq!(result.get_value::<String>("optional"), "optional");
                assert_eq!(result.get_value::<String>("stuff"), "values");
                assert!(result.get_flag("flag"));
            }
            Err(e) => panic!("{}", e),
        }
    }

    #[test]
    fn run_calls_main_and_propagates_ok() {
        let args: &[String] = &["positional".to_string()];
        let parser = Parser::new("test", "doc")
            .add_positional("positional", "a value")
            .with_main(|_| Ok(()));
        let result = parser.parse(Vec::from(args)).unwrap();
        assert!(result.run().is_ok());
    }

    #[test]
    fn run_propagates_error_from_main() {
        let args: &[String] = &["positional".to_string()];
        let parser = Parser::new("test", "doc")
            .add_positional("positional", "a value")
            .with_main(|_| Err("something went wrong".to_string()));
        let result = parser.parse(Vec::from(args)).unwrap();
        assert_eq!(result.run(), Err("something went wrong".to_string()));
    }

    #[test]
    #[should_panic(expected = "leaf parser 'test' has no main function")]
    fn parse_leaf_without_main_panics() {
        let args: &[String] = &["positional".to_string()];
        let parser = Parser::new("test", "doc").add_positional("positional", "a value");
        parser.parse(Vec::from(args)).unwrap();
    }

    #[test]
    fn run_dispatches_correct_main_for_sub_action() {
        let args: &[String] = &[
            "pos".to_string(),
            "compute".to_string(),
            "stuff".to_string(),
        ];
        let parser = Parser::new("test", "doc")
            .add_positional("positional", "a value")
            .with_main(|_| Err("wrong main called".to_string()))
            .add_action(
                Parser::new("compute", "compute things")
                    .add_positional("stuff", "stuff")
                    .with_main(|_| Ok(())),
            );
        let result = parser.parse(Vec::from(args)).unwrap();
        assert!(result.run().is_ok());
    }

    #[test]
    fn generate_help_message_no_actions() {
        let parser = Parser::new("tool", "A simple tool")
            .add_positional("input", "the input value")
            .add_flag("verbose", "verbose", 'v', "enable verbose output");
        let expected = r#"tool - A simple tool

usage: tool [input] {-v}

[input]               the input value
-v,--verbose          enable verbose output
"#;
        assert_eq!(parser.help(), expected);
    }

    #[test]
    fn generate_help_message() {
        let parser: Parser = get_nested_parser();
        let help = parser.help();
        let expected = r#"test - I am a test

usage: test [positional] {-o,-f} compute

[positional]          I am the positional
-o,--optional         I am the optional
-f,--flag             I am the flag
compute               I am da computaaah
"#;
        assert_eq!(help, expected);
    }

    #[test]
    fn triggering_help_flag_outer_parser() {
        let args: &[String] = &[
            "positional".to_string(),
            "-o".to_string(),
            "optional".to_string(),
            "-h".to_string(),
            "compute".to_string(),
            "values".to_string(),
        ];
        let parser: Parser = get_nested_parser();
        let expected = r#"test - I am a test

usage: test [positional] {-o,-f} compute

[positional]          I am the positional
-o,--optional         I am the optional
-f,--flag             I am the flag
compute               I am da computaaah
"#;
        match parser.parse(Vec::from(args)) {
            Ok(_) => panic!("Should not have parsed"),
            Err(msg) => {
                assert_eq!(msg, expected)
            }
        }
    }

    #[test]
    fn triggering_help_flag_inner_parser() {
        let args: &[String] = &[
            "positional".to_string(),
            "-o".to_string(),
            "optional".to_string(),
            "-f".to_string(),
            "compute".to_string(),
            "--help".to_string(),
        ];
        let parser: Parser = get_nested_parser();
        let expected = r#"compute - I am da computaaah

usage: compute [stuff]

[stuff]               stuff indeed
"#;
        match parser.parse(Vec::from(args)) {
            Ok(_) => panic!("Should not have parsed"),
            Err(msg) => {
                println!("{}", msg);
                assert_eq!(msg, expected)
            }
        }
    }
}
