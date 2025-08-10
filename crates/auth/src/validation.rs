use regex::Regex;
use lazy_static::lazy_static;

lazy_static! {
    /// Regex pattern for validating that a string contains only numeric digits
    pub static ref NUMERIC_REGEX: Regex = Regex::new(r"^[0-9]+$").expect("Failed to compile numeric regex");
}