//! Neo4j Repository Implementation
//!
//! This module provides the core functionality for interacting with the Neo4j graph database,
//! including CRUD operations for persons and relationship management in a genealogy context.

use neo4rs::*;
use std::error::Error;
use chrono::NaiveDate;
use crate::models::{Person, Relationship};

/// Repository struct for Neo4j database operations
pub struct Neo4jRepository {
    graph: Graph,
}

impl Neo4jRepository {
    /// Creates a new Neo4j repository instance
    ///
    /// # Arguments
    /// * `uri` - Connection URI for the Neo4j database
    ///
    /// # Returns
    /// * `Result<Self, Box<dyn Error>>` - Repository instance or error
    pub async fn new(uri: String) -> std::result::Result<Self, Box<dyn Error>> {
        let config = ConfigBuilder::default()
            .uri(uri)
            .user("neo4j")
            .password("password")
            .db("neo4j")
            .build()?;

        let graph = Graph::connect(config).await?;
        Ok(Self { graph })
    }

    /// Returns a reference to the underlying graph connection
    pub async fn get_graph(&self) -> &Graph {
        &self.graph
    }

    /// Creates a new person in the database
    ///
    /// # Arguments
    /// * `person` - Person struct containing the individual's information
    ///
    /// # Returns
    /// * `Result<String, Box<dyn Error>>` - The ID of the created person or error
    pub async fn create_person(&self, person: &Person) -> Result<String> {
        let query = query(
            "CREATE (p:Person {
                first_name: $first_name,
                middle_name: $middle_name,
                last_name: $last_name,
                maiden_name: $maiden_name,
                nick_name: $nick_name,
                full_name: $full_name,
                born: $born,
                deceased: $deceased,
                info: $info
            }) RETURN id(p)"
        )
            .param("first_name", &person.first_name)
            .param("middle_name", &person.middle_name)
            .param("last_name", &person.last_name)
            .param("maiden_name", &person.maiden_name)
            .param("nick_name", &person.nick_name)
            .param("full_name", &person.full_name)
            .param("born", &person.born.map(|d| d.to_string()))
            .param("deceased", &person.deceased.map(|d| d.to_string()))
            .param("info", &person.info);

        let mut result = self.graph.execute(query).await?;
        let row = result.next().await?.unwrap();
        let id: i64 = row.get("id(p)").unwrap();

        Ok(id.to_string())
    }

    /// Retrieves a person by their ID
    ///
    /// # Arguments
    /// * `id` - The unique identifier of the person
    ///
    /// # Returns
    /// * `Result<Option<Person>, Box<dyn Error>>` - The person if found, None if not found, or error
    pub async fn get_person(&self, id: &str) -> Result<Option<Person>> {
        let query = query(
            "MATCH (p:Person) WHERE id(p) = $id
             RETURN p.first_name, p.middle_name, p.last_name, p.maiden_name,
                    p.nick_name, p.full_name, p.born, p.deceased, p.info"
        )
            .param("id", id);

        let mut result = self.graph.execute(query).await?;

        if let Some(row) = result.next().await? {
            Ok(Some(Person {
                first_name: row.get("p.first_name").unwrap(),
                middle_name: row.get("p.middle_name").unwrap(),
                last_name: row.get("p.last_name").unwrap(),
                maiden_name: row.get("p.maiden_name").unwrap(),
                nick_name: row.get("p.nick_name").unwrap(),
                full_name: row.get("p.full_name").unwrap(),
                born: row.get::<String>("p.born")
                    .map(|d| NaiveDate::parse_from_str(&d, "%Y-%m-%d").unwrap()),
                deceased: row.get::<String>("p.deceased")
                    .map(|d| NaiveDate::parse_from_str(&d, "%Y-%m-%d").unwrap()),
                info: row.get("p.info").unwrap(),
            }))
        } else {
            Ok(None)
        }
    }

    /// Adds a relationship between two persons
    ///
    /// # Arguments
    /// * `rel` - The relationship to be created (Parent or Marriage)
    ///
    /// # Returns
    /// * `Result<(), Box<dyn Error>>` - Success or error
    pub async fn add_relationship(&self, rel: &Relationship) -> Result<()> {
        match rel {
            Relationship::Parent { from_id, to_id } => {
                let query = query(
                    "MATCH (parent:Person), (child:Person)
                     WHERE id(parent) = $from_id AND id(child) = $to_id
                     CREATE (parent)-[:PARENT_OF]->(child)"
                )
                    .param("from_id", from_id)
                    .param("to_id", to_id);

                self.graph.execute(query).await?;
            },
            Relationship::Marriage { from_id, to_id, wedding_date, divorce_date } => {
                let query = query(
                    "MATCH (p1:Person), (p2:Person)
                     WHERE id(p1) = $from_id AND id(p2) = $to_id
                     CREATE (p1)-[:MARRIED_TO {
                         wedding_date: $wedding_date,
                         divorce_date: $divorce_date
                     }]->(p2)"
                )
                    .param("from_id", from_id)
                    .param("to_id", to_id)
                    .param("wedding_date", wedding_date.map(|d| d.to_string()))
                    .param("divorce_date", divorce_date.map(|d| d.to_string()));

                self.graph.execute(query).await?;
            }
        }
        Ok(())
    }

    /// Retrieves the family tree for a person
    ///
    /// # Arguments
    /// * `person_id` - ID of the person whose family tree to retrieve
    ///
    /// # Returns
    /// * `Result<Vec<Person>, Box<dyn Error>>` - List of related persons or error
    pub async fn get_family_tree(&self, person_id: &str) -> Result<Vec<Person>> {
        let query = query(
            "MATCH (p:Person)-[*1..3]-(relative:Person)
             WHERE id(p) = $person_id
             RETURN DISTINCT relative"
        )
            .param("person_id", person_id);

        let mut result = self.graph.execute(query).await?;
        let mut relatives = Vec::new();

        while let Some(row) = result.next().await? {
            // TODO: Implement conversion from row to Person
        }

        Ok(relatives)
    }
}