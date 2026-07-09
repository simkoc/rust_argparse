use std::any::Any;
use std::collections::HashMap;

pub struct CmdParsingResults {
    results: HashMap<String, Box<dyn Any>>,
    action: Option<String>,
}

impl CmdParsingResults {
    pub(crate) fn keys(&self) -> Vec<&String> {
        self.results.keys().collect::<Vec<&String>>()
    }

    pub(crate) fn new() -> CmdParsingResults {
        CmdParsingResults {
            results: HashMap::new(),
            action: None,
        }
    }

    pub(crate) fn set_action(&mut self, action: String) -> () {
        self.action = Some(action);
    }

    pub fn get_action(&self) -> String {
        self.action.clone().expect("no main set for leaf action")
    }

    pub(crate) fn add_result_value(&mut self, name: String, result: Box<dyn Any>) -> () {
        self.results.insert(name.clone(), result);
    }

    pub fn get_value<T: 'static>(&self, name: &str) -> &T {
        match self.results.get(name) {
            Some(value) => match value.downcast_ref::<T>() {
                Some(value) => value,
                None => panic!("value {} is not of expected type", name),
            },
            None => panic!("{} not found", name),
        }
    }

    pub fn get_optional_value<T: 'static>(&self, name: &str) -> Option<&T> {
        match self.results.get(name) {
            Some(value) => match value.downcast_ref::<T>() {
                Some(value) => Some(value),
                None => None,
            },
            None => None,
        }
    }

    pub fn get_flag(&self, name: &str) -> bool {
        match self.results.get(name) {
            Some(value) => match value.downcast_ref::<bool>() {
                Some(value) => *value,
                None => panic!("value {} is not a flag", name),
            },
            None => panic!("flag {} not found", name),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn set_and_get_action() {
        let mut res: CmdParsingResults = CmdParsingResults::new();
        res.set_action("mycommand".to_string());
        assert_eq!(res.get_action(), "mycommand");
    }

    #[test]
    fn set_and_get_action_overwrite() {
        let mut res: CmdParsingResults = CmdParsingResults::new();
        res.set_action("first".to_string());
        res.set_action("second".to_string());
        assert_eq!(res.get_action(), "second");
    }

    #[test]
    #[should_panic(expected = "no main set for leaf action")]
    fn get_action_without_set_panics() {
        let res: CmdParsingResults = CmdParsingResults::new();
        res.get_action();
    }

    #[test]
    fn add_and_retrieve_value() {
        let mut res: CmdParsingResults = CmdParsingResults::new();
        res.add_result_value("test".to_string(), Box::new(String::from("test")));
        assert_eq!(*res.get_value::<String>("test"), *String::from("test"));
    }

    #[test]
    fn add_and_retrieve_value_i32() {
        let mut res: CmdParsingResults = CmdParsingResults::new();
        res.add_result_value("count".to_string(), Box::new(42_i32));
        assert_eq!(*res.get_value::<i32>("count"), 42);
    }

    #[test]
    #[should_panic(expected = "not of expected type")]
    fn retrieve_value_wrong_type_panics() {
        let mut res: CmdParsingResults = CmdParsingResults::new();
        res.add_result_value("test".to_string(), Box::new(String::from("hello")));
        res.get_value::<i32>("test");
    }

    #[test]
    fn add_and_retrieve_optional_existing_value() {
        let mut res: CmdParsingResults = CmdParsingResults::new();
        res.add_result_value("test".to_string(), Box::new(String::from("test")));
        assert_eq!(
            res.get_optional_value::<String>("test"),
            Some(&String::from("test"))
        );
    }

    #[test]
    fn add_and_retrieve_optional_existing_i32_value() {
        let mut res: CmdParsingResults = CmdParsingResults::new();
        res.add_result_value("count".to_string(), Box::new(7_i32));
        assert_eq!(res.get_optional_value::<i32>("count"), Some(&7_i32));
    }

    #[test]
    fn retrieve_option_missing_value() {
        let res: CmdParsingResults = CmdParsingResults::new();
        assert_eq!(res.get_optional_value::<String>("test"), None);
    }

    #[test]
    fn retrieve_flag_value_that_exists() {
        let mut res: CmdParsingResults = CmdParsingResults::new();
        res.add_result_value("test".to_string(), Box::new(true));
        assert_eq!(res.get_flag("test"), true);
    }

    #[test]
    #[should_panic(expected = "not found")]
    fn retrieve_flag_value_that_does_not_exists() {
        let res: CmdParsingResults = CmdParsingResults::new();
        res.get_flag("test");
    }

    #[test]
    #[should_panic(expected = "not a flag")]
    fn retrieve_non_flag_value_as_flag() {
        let mut res: CmdParsingResults = CmdParsingResults::new();
        res.add_result_value("test".to_string(), Box::new(String::from("test")));
        res.get_flag("test");
    }
}
