// Test compilation of master-data types without SQLx dependencies
use erp_master_data::types::*;
use uuid::Uuid;

fn main() {
    let id = Uuid::new_v4();
    println!("Test compile successful with UUID: {}", id);
}