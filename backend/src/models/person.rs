use serde::{Deserialize, Serialize};
use chrono::NaiveDate;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Person {
    pub first_name: String,
    pub last_name: String,
    pub full_name: String,
    pub middle_name: Option<String>,
    pub maiden_name: Option<String>,
    pub nick_name: Option<String>,
    pub born: Option<String>,
    pub deceased: Option<String>,
    pub info: Option<String>,
}