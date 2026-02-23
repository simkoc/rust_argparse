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
use std::env;

pub struct Parser {
    name: String,
    doc: String,
    defaults: Vec<DefaultArgument>,
    actions: Vec<Parser>,
    positionals: Vec<PositionalArgument>,
    optionals: Vec<OptionalArgument>,
    flags: Vec<FlagArgument>,
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
        }
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

    fn parse_action_arguments<'b>(
        &self,
        result: &mut CmdParsingResults,
        cmdline: &'b [String],
    ) -> Result<&'b [String], String> {
        //println!("parsing actions starting with : {}", cmdline.join(" "));
        let remaining_cmd_line = cmdline;
        // if there are no elements remaining but there should be actions
        if self.actions.is_empty() {
            if remaining_cmd_line.is_empty() {
                return Ok(remaining_cmd_line);
            }
        } else {
            if remaining_cmd_line.is_empty() {
                return Err(format!("You have to chose an action. \n\n {}", self.help()));
            }
        }
        // otherwise go through all action elements
        for (_, item) in self.actions.iter().enumerate() {
            //println!("testing action {}", item.name);
            // if we have an action with a name matching the current first cmdline argument
            if item.name == *remaining_cmd_line.first().unwrap() {
                // run the parser
                return match CommandLineParsing::parse(item, result, &remaining_cmd_line[1..]) {
                    // if success
                    Ok(remains) =>
                    // and we fully parsed the cmd line string
                    {
                        if remains.is_empty() {
                            // return the result
                            //println!("parsed ...");
                            Ok(remains)
                        // if we did not fully parse
                        } else {
                            // thats an error
                            Err(format!(
                                "Too many supplied arguments after: {:?}\n\n{}",
                                remains,
                                self.help()
                            ))
                        }
                    }
                    // if unsuccessful
                    Err(e) =>
                    // return the encountered issue down the stack
                    {
                        Err(e)
                    }
                };
            }
        }
        // if we are here the currently specified action is unknown
        Err(format!(
            "Unknown action {}",
            remaining_cmd_line
                .first()
                .expect("the remaining_cmd_line should be non empty")
        ))
    }
}

impl CommandLineParsing for Parser {
    fn help(&self) -> String {
        let mut help_msg_head = self.name.clone() + " - " + self.doc.as_str();
        let mut help_msg_cmd_line: String = "usage: ".to_string();
        let mut help_msg_body: String = "".to_string();
        help_msg_cmd_line += self.name.as_str();
        for positional in self.positionals.iter() {
            help_msg_cmd_line += " ";
            help_msg_cmd_line += "[";
            help_msg_cmd_line += positional.name();
            help_msg_cmd_line += "]";
            help_msg_body += positional.help().as_str();
            help_msg_body += "\n";
        }
        let mut counter = 1000;
        if !self.optionals.is_empty() || !self.flags.is_empty() {
            help_msg_cmd_line += " ";
            help_msg_cmd_line += "{";
        }
        for (num, optional) in self.optionals.iter().enumerate() {
            if counter != 1000 {
                help_msg_cmd_line += ",";
            }
            counter = num;
            help_msg_cmd_line += "-";
            help_msg_cmd_line += String::from(optional.short()).as_str();
            help_msg_body += optional.help().as_str();
            help_msg_body += "\n";
        }
        for (num, flag) in self.flags.iter().enumerate() {
            if counter != 1000 {
                help_msg_cmd_line += ",";
            }
            counter = num;
            help_msg_cmd_line += "-";
            help_msg_cmd_line += String::from(flag.short()).as_str();
            help_msg_body += flag.help().as_str();
            help_msg_body += "\n";
        }
        if !self.optionals.is_empty() || !self.flags.is_empty() {
            help_msg_cmd_line += "}";
        }
        help_msg_cmd_line += " ";
        for (num, action) in self.actions.iter().enumerate() {
            if num != 0 {
                help_msg_cmd_line += ",";
            }
            help_msg_cmd_line += action.name.as_str();
            let name = action.name.as_str();
            let spaces = 22 - name.len();
            help_msg_body += name;
            help_msg_body += String::from_utf8(vec![b' '; spaces])
                .expect("should be a string of whitespaces")
                .as_str();
            help_msg_body += action.doc.as_str();
            help_msg_body += "\n";
        }
        help_msg_head += "\n\n";
        help_msg_head += &*help_msg_cmd_line.trim();
        help_msg_head += "\n\n";
        help_msg_head += &*help_msg_body;
        help_msg_head
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
    }

    fn get_nested_parser() -> Parser {
        get_basic_cmd_parser().add_action(
            Parser::new("compute", "I am da computaaah").add_positional("stuff", "stuff indeed"),
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
