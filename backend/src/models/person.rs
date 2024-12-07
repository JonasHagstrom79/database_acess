use serde::{Deserialize, Serialize};
use chrono::NaiveDate;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Person {
    pub first_name: String,
    pub middle_name: Option<String>,
    pub last_name: String,
    pub maiden_name: Option<String>,
    pub nick_name: Option<String>,
    pub full_name: String,
    pub born: Option<NaiveDate>,
    pub deceased: Option<NaiveDate>,
    pub info: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Relationship {
    Parent {
        from_id: String,
        to_id: String
    },
    Marriage {
        from_id: String,
        to_id: String,
        wedding_date: Option<NaiveDate>,
        divorce_date: Option<NaiveDate>
    }
}