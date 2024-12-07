use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Employee {
    pub employee_id: i32,
    pub name: String,
    pub salary: f64,
    pub region: String,
}