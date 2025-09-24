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

/// Convert Decimal to f64 directly (for non-Option types)
pub fn decimal_to_f64_direct(decimal: Decimal) -> f64 {
    decimal.to_f64().unwrap_or(0.0)
}

/// Convert Option<JsonValue> to HashMap<String, f64>
pub fn json_to_f64_map(json: Option<JsonValue>) -> HashMap<String, f64> {
    json.and_then(|v| {
        if let JsonValue::Object(map) = v {
            let mut result = HashMap::new();
            for (k, v) in map {
                if let JsonValue::Number(n) = v {
                    if let Some(f) = n.as_f64() {
                        result.insert(k, f);
                    }
                }
            }
            Some(result)
        } else {
            None
        }
    }).unwrap_or_default()
}

/// Convert Option<JsonValue> to HashMap<String, JsonValue>
pub fn json_to_json_map(json: Option<JsonValue>) -> HashMap<String, JsonValue> {
    json.and_then(|v| {
        if let JsonValue::Object(map) = v {
            Some(map.into_iter().collect())
        } else {
            None
        }
    }).unwrap_or_default()
}

/// Convert Option<Decimal> to f64 safely
pub fn option_decimal_to_f64(decimal: Option<Decimal>) -> f64 {
    decimal.and_then(|d| d.to_f64()).unwrap_or(0.0)
}

/// Convert Option<T> types to required T with proper error handling
pub fn option_uuid_to_uuid(uuid: Option<Uuid>) -> Uuid {
    uuid.unwrap_or_else(|| Uuid::new_v4())
}

/// Convert Option<i32> to i32 with proper error handling
pub fn option_i32_to_i32(value: Option<i32>) -> i32 {
    value.unwrap_or(0)
}

/// Convert Option<String> to String with proper error handling
pub fn option_string_to_string(value: Option<String>) -> String {
    value.unwrap_or_default()
}

/// Convert Option<DateTime<Utc>> to DateTime<Utc> with proper error handling
pub fn option_datetime_to_datetime(value: Option<DateTime<Utc>>) -> DateTime<Utc> {
    value.unwrap_or_else(|| Utc::now())
}

/// Convert Option<MovementType> to MovementType with default
pub fn option_movement_type_to_movement_type<T: Default>(value: Option<T>) -> T {
    value.unwrap_or_default()
}

/// Convert Option<Decimal> to Option<f64> safely
pub fn option_decimal_to_option_f64(decimal: Option<Decimal>) -> Option<f64> {
    decimal.and_then(|d| d.to_f64())
}

/// Convert Option<NaiveDate> to Option<DateTime<Utc>> safely
pub fn option_naivedate_to_option_datetime(date: Option<NaiveDate>) -> Option<DateTime<Utc>> {
    date.map(|d| d.and_hms_opt(0, 0, 0).unwrap().and_utc())
}

/// Convert Decimal to f64 safely with better error handling
pub fn decimal_to_f64_safe(decimal: Decimal) -> f64 {
    decimal.to_f64().unwrap_or(0.0)
}

/// Safe conversion from Option<JsonValue> to specific types
pub fn json_value_to_hashmap_f64(json: Option<JsonValue>) -> HashMap<String, f64> {
    match json {
        Some(JsonValue::Object(map)) => {
            map.into_iter()
                .filter_map(|(k, v)| {
                    v.as_f64().map(|f| (k, f))
                })
                .collect()
        }
        _ => HashMap::new(),
    }
}

/// Convert JsonValue to string safely for SQLX encoding
pub fn json_to_string_safe(json: Option<JsonValue>) -> String {
    json.map(|v| serde_json::to_string(&v).unwrap_or_default())
        .unwrap_or_default()
}

/// Convert string back to JsonValue safely
pub fn string_to_json_safe(s: String) -> Option<JsonValue> {
    if s.is_empty() {
        None
    } else {
        serde_json::from_str(&s).ok()
    }
}

/// Convert SQLX enum types to model enums safely
pub fn convert_sqlx_enum_to_model<T: Default>(value: Option<T>) -> T {
    value.unwrap_or_default()
}

/// Convert Option<Decimal> from SQLX to Option<f64>
pub fn sqlx_decimal_option_to_f64_option(decimal: Option<rust_decimal::Decimal>) -> Option<f64> {
    decimal.map(|d| d.to_f64().unwrap_or(0.0))
}

/// Convert nested options correctly for SQLX results
pub fn sqlx_nested_option_flatten<T>(nested: Option<Option<T>>) -> Option<T> {
    nested.flatten()
}

/// Convert NaiveDate to DateTime<Utc> for SQLX results
pub fn naive_date_to_utc_datetime(date: Option<chrono::NaiveDate>) -> Option<DateTime<Utc>> {
    use chrono::{TimeZone, Utc, NaiveTime};
    date.map(|d| Utc.from_utc_datetime(&d.and_time(NaiveTime::from_hms_opt(0, 0, 0).unwrap())))
}

/// Convert Decimal directly to f64 for non-Option database fields
pub fn convert_decimal_to_f64_direct(decimal: rust_decimal::Decimal) -> f64 {
    decimal.to_f64().unwrap_or(0.0)
}

/// Wrap value in Some() for Option fields
pub fn wrap_in_some<T>(value: T) -> Option<T> {
    Some(value)
}

/// Convert database row fields to proper types for InventoryMovement
pub fn convert_to_movement_type(transaction_type: Option<String>) -> crate::inventory::model::MovementType {
    use crate::inventory::model::MovementType;
    match transaction_type.as_deref() {
        Some("receipt") => MovementType::Receipt,
        Some("shipment") => MovementType::Shipment,
        Some("transfer") => MovementType::Transfer,
        Some("adjustment") => MovementType::Adjustment,
        Some("return") => MovementType::Return,
        Some("damage") => MovementType::Damage,
        Some("loss") => MovementType::Loss,
        Some("found") => MovementType::Found,
        Some("production") => MovementType::Production,
        Some("consumption") => MovementType::Consumption,
        Some("cycle_count") => MovementType::CycleCount,
        Some("physical_count") => MovementType::PhysicalCount,
        _ => MovementType::Adjustment, // Default
    }
}

/// Convert database row fields to proper types for LocationType
pub fn convert_to_location_type(location_type: Option<String>) -> Option<crate::inventory::model::LocationType> {
    use crate::inventory::model::LocationType;
    match location_type.as_deref() {
        Some("warehouse") => Some(LocationType::Warehouse),
        Some("store") => Some(LocationType::Store),
        Some("distribution_center") => Some(LocationType::DistributionCenter),
        Some("supplier") => Some(LocationType::Supplier),
        _ => None,
    }
}

/// Convert database row fields to proper types for ABCClassification
pub fn convert_to_abc_classification(abc: Option<String>) -> Option<crate::inventory::model::ABCClassification> {
    use crate::inventory::model::ABCClassification;
    match abc.as_deref() {
        Some("a") => Some(ABCClassification::A),
        Some("b") => Some(ABCClassification::B),
        Some("c") => Some(ABCClassification::C),
        _ => None,
    }
}

/// Convert database row fields to proper types for MovementVelocity
pub fn convert_to_movement_velocity(velocity: Option<String>) -> Option<crate::inventory::model::MovementVelocity> {
    use crate::inventory::model::MovementVelocity;
    match velocity.as_deref() {
        Some("fast") => Some(MovementVelocity::Fast),
        Some("medium") => Some(MovementVelocity::Medium),
        Some("slow") => Some(MovementVelocity::Slow),
        Some("dead") => Some(MovementVelocity::Dead),
        _ => None,
    }
}

/// Import types for use in conversion functions (currently not used in this file)

/// Convert Option<NaiveDate> to DateTime<Utc> safely
pub fn naivedate_to_datetime_safe(date: Option<chrono::NaiveDate>) -> Option<chrono::DateTime<chrono::Utc>> {
    date.map(|d| d.and_hms_opt(0, 0, 0).unwrap().and_utc())
}

/// Convert Decimal directly to f64 for direct assignments
pub fn decimal_to_f64_direct_safe(decimal: rust_decimal::Decimal) -> f64 {
    decimal.to_f64().unwrap_or(0.0)
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

/// Convert string to LocationType enum
pub fn string_to_location_type(s: String) -> crate::inventory::model::LocationType {
    use crate::inventory::model::LocationType;
    match s.to_lowercase().as_str() {
        "warehouse" => LocationType::Warehouse,
        "store" => LocationType::Store,
        "distribution_center" => LocationType::DistributionCenter,
        "manufacturing_plant" => LocationType::ManufacturingPlant,
        "supplier" => LocationType::Supplier,
        "customer" => LocationType::Customer,
        "transit" => LocationType::Transit,
        "virtual" => LocationType::Virtual,
        _ => LocationType::Warehouse, // Default
    }
}

/// Convert string to ABCClassification enum
pub fn string_to_abc_classification(s: String) -> crate::inventory::model::ABCClassification {
    use crate::inventory::model::ABCClassification;
    match s.to_uppercase().as_str() {
        "A" => ABCClassification::A,
        "B" => ABCClassification::B,
        "C" => ABCClassification::C,
        "X" => ABCClassification::X,
        _ => ABCClassification::C, // Default
    }
}

/// Convert string to MovementVelocity enum
pub fn string_to_movement_velocity(s: String) -> crate::inventory::model::MovementVelocity {
    use crate::inventory::model::MovementVelocity;
    match s.to_lowercase().as_str() {
        "fast" => MovementVelocity::Fast,
        "medium" => MovementVelocity::Medium,
        "slow" => MovementVelocity::Slow,
        "dead" => MovementVelocity::Dead,
        "seasonal" => MovementVelocity::Seasonal,
        _ => MovementVelocity::Medium, // Default
    }
}

/// Convert JSON value to StorageRequirements
pub fn json_to_storage_requirements(json: Option<serde_json::Value>) -> crate::inventory::model::StorageRequirements {
    json.and_then(|v| serde_json::from_value(v).ok())
        .unwrap_or_default()
}

/// ============================================================================
/// SQLX OPTION-UNWRAPPING INFRASTRUCTURE FOR DATABASE QUERIES
/// ============================================================================

/// SQLX-specific conversion helpers to resolve Option<T> → T trait bound issues

/// Convert Option<Option<T>> to Option<T> (flattens nested Options from SQLX)
pub fn flatten_option<T>(nested_option: Option<Option<T>>) -> Option<T> {
    nested_option.flatten()
}

/// Convert Option<Uuid> to Uuid with error handling for required fields
pub fn sqlx_option_uuid_to_uuid(uuid: Option<Uuid>) -> Result<Uuid, String> {
    uuid.ok_or_else(|| "Required UUID field is None from database".to_string())
}

/// Convert Option<String> to String with error handling for required fields
pub fn sqlx_option_string_to_string(string: Option<String>) -> Result<String, String> {
    string.ok_or_else(|| "Required string field is None from database".to_string())
}

/// Convert Option<DateTime<Utc>> to DateTime<Utc> with error handling
pub fn sqlx_option_datetime_to_datetime(datetime: Option<DateTime<Utc>>) -> Result<DateTime<Utc>, String> {
    datetime.ok_or_else(|| "Required datetime field is None from database".to_string())
}

/// Convert Option<i32> to i32 with error handling for required fields
pub fn sqlx_option_i32_to_i32(value: Option<i32>) -> Result<i32, String> {
    value.ok_or_else(|| "Required i32 field is None from database".to_string())
}

/// Convert Option<f64> to f64 with error handling for required fields
pub fn sqlx_option_f64_to_f64(value: Option<f64>) -> Result<f64, String> {
    value.ok_or_else(|| "Required f64 field is None from database".to_string())
}

/// Convert Option<MovementType> to MovementType with error handling
pub fn sqlx_option_movement_type_to_movement_type(movement_type: Option<crate::inventory::model::MovementType>) -> Result<crate::inventory::model::MovementType, String> {
    movement_type.ok_or_else(|| "Required MovementType field is None from database".to_string())
}

/// Convert Option<Vec<String>> to Vec<String> with default empty vec
pub fn sqlx_option_vec_string_to_vec_string(vec: Option<Vec<String>>) -> Vec<String> {
    vec.unwrap_or_default()
}

/// Convert Option<HashMap<String, JsonValue>> to HashMap<String, JsonValue> with default empty map
pub fn sqlx_option_hashmap_to_hashmap(map: Option<HashMap<String, JsonValue>>) -> HashMap<String, JsonValue> {
    map.unwrap_or_default()
}

/// Safe SQLX Option unwrapping with default values
pub fn sqlx_unwrap_or_default<T: Default>(option: Option<T>) -> T {
    option.unwrap_or_default()
}

/// Safe SQLX Option unwrapping with custom default
pub fn sqlx_unwrap_or<T>(option: Option<T>, default: T) -> T {
    option.unwrap_or(default)
}

/// Convert SQLX row Option<Option<T>> nested types to T with defaults
pub fn sqlx_nested_option_to_option<T>(nested: Option<Option<T>>) -> Option<T> {
    nested.flatten()
}

/// Convert SQLX row Option<Option<DateTime<Utc>>> to Option<DateTime<Utc>>
pub fn sqlx_nested_datetime_to_option(nested: Option<Option<DateTime<Utc>>>) -> Option<DateTime<Utc>> {
    nested.flatten()
}

/// Convert SQLX row Option<Option<f64>> to Option<f64>
pub fn sqlx_nested_f64_to_option(nested: Option<Option<f64>>) -> Option<f64> {
    nested.flatten()
}

/// Convert SQLX row Option<Option<Vec<String>>> to Option<Vec<String>>
pub fn sqlx_nested_vec_to_option(nested: Option<Option<Vec<String>>>) -> Option<Vec<String>> {
    nested.flatten()
}

