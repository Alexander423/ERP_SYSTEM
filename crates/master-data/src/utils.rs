// Utility functions for type conversions and common operations

use rust_decimal::Decimal;
use rust_decimal::prelude::ToPrimitive;
use uuid::Uuid;
use chrono::{DateTime, Utc, NaiveDate};
use std::collections::HashMap;
use serde_json::Value as JsonValue;

/// Convert Option<Decimal> to Option<f64> safely
pub fn decimal_to_f64(decimal: Option<Decimal>) -> Option<f64> {
    decimal.and_then(|d| d.to_f64())
}

/// Convert Decimal to f64 with default value
pub fn decimal_to_f64_or_default(decimal: Option<Decimal>) -> f64 {
    decimal.and_then(|d| d.to_f64()).unwrap_or(0.0)
}

/// Safely unwrap Option<T> to T with error handling
pub fn unwrap_or_error<T>(option: Option<T>, field_name: &str) -> Result<T, String> {
    option.ok_or_else(|| format!("Required field '{}' is missing", field_name))
}

/// Convert Option<Uuid> to Uuid safely
pub fn uuid_or_error(uuid: Option<Uuid>, field_name: &str) -> Result<Uuid, String> {
    uuid.ok_or_else(|| format!("Required UUID field '{}' is missing", field_name))
}

/// Convert Option<String> to String safely
pub fn string_or_error(string: Option<String>, field_name: &str) -> Result<String, String> {
    string.ok_or_else(|| format!("Required string field '{}' is missing", field_name))
}

/// Convert Option<String> to String with default
pub fn string_or_default(string: Option<String>) -> String {
    string.unwrap_or_default()
}

/// Convert Option<DateTime<Utc>> to DateTime<Utc> safely
pub fn datetime_or_error(datetime: Option<DateTime<Utc>>, field_name: &str) -> Result<DateTime<Utc>, String> {
    datetime.ok_or_else(|| format!("Required datetime field '{}' is missing", field_name))
}

/// Convert Option<NaiveDate> to Option<DateTime<Utc>>
pub fn naivedate_to_datetime(date: Option<NaiveDate>) -> Option<DateTime<Utc>> {
    date.map(|d| d.and_hms_opt(0, 0, 0).unwrap().and_utc())
}

/// Convert Option<String> to HashMap<String, JsonValue> for JSONB fields
pub fn string_to_json_map(json_str: Option<String>) -> HashMap<String, JsonValue> {
    json_str
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

/// Convert i64 to f64 safely
pub fn i64_to_f64(value: i64) -> f64 {
    value as f64
}

/// Convert i32 to i64 safely
pub fn i32_to_i64(value: i32) -> i64 {
    value as i64
}

/// Convert Option<i32> to i32 with default
pub fn i32_or_default(value: Option<i32>) -> i32 {
    value.unwrap_or(0)
}

/// Generic function to convert any Option<T> to T with default
pub fn option_or_default<T: Default>(value: Option<T>) -> T {
    value.unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal::Decimal;

    #[test]
    fn test_decimal_to_f64() {
        let decimal = Some(Decimal::new(1234, 2)); // 12.34
        assert_eq!(decimal_to_f64(decimal), Some(12.34));

        assert_eq!(decimal_to_f64(None), None);
    }

    #[test]
    fn test_decimal_to_f64_or_default() {
        let decimal = Some(Decimal::new(1234, 2)); // 12.34
        assert_eq!(decimal_to_f64_or_default(decimal), 12.34);

        assert_eq!(decimal_to_f64_or_default(None), 0.0);
    }

    #[test]
    fn test_unwrap_or_error() {
        assert_eq!(unwrap_or_error(Some("test".to_string()), "field"), Ok("test".to_string()));
        assert!(unwrap_or_error::<String>(None, "field").is_err());
    }
}