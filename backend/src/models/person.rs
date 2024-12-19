use serde::{Deserialize, Serialize};
use chrono::NaiveDateTime;

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Person {
    pub info: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct PersonCreate {
    pub first_name: String,
    pub last_name: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct RelationCreate {
    pub related_person_id: String,
    pub relation_type: RelationType,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum RelationType {
    FAR,
    MOR,
    SON,
    DOTTER,
}

impl PersonCreate {

}