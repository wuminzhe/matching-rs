use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub struct TinyError {
    details: String
}

impl TinyError {
    pub fn new(msg: &str) -> TinyError {
        TinyError{details: msg.to_string()}
    }
}

impl fmt::Display for TinyError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,"{}",self.details)
    }
}

impl Error for TinyError {
    fn description(&self) -> &str {
        &self.details
    }
}
