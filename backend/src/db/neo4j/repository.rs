use neo4rs::*;
use std::error::Error;
use crate::models::{Person, Relationship};
use std::fs::File;
use csv::Reader;

pub struct Neo4jRepository {
    graph: Graph,
}

impl Neo4jRepository {
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

    pub async fn get_graph(&self) -> &Graph {
        &self.graph
    }

    // CRUD-operationer för Person
    pub async fn create_person(&self, person: &Person) -> Result<String> {
        let query = query(
            "CREATE (p:Person {
            firstName: $firstName,
            middleName: $middleName,
            lastName: $lastName,
            maidenName: $maidenName,
            nickName: $nickName,
            fullName: $fullName,
            born: $born,
            deceased: $deceased,
            info: $info
        }) RETURN id(p)"
        )
            .param("firstName", person.first_name.as_str())
            .param("lastName", person.last_name.as_str())
            .param("fullName", person.full_name.as_str())
            .param("middleName", person.middle_name.as_deref().unwrap_or(""))
            .param("maidenName", person.maiden_name.as_deref().unwrap_or(""))
            .param("nickName", person.nick_name.as_deref().unwrap_or(""))
            .param("born", person.born.as_deref().unwrap_or(""))
            .param("deceased", person.deceased.as_deref().unwrap_or(""))
            .param("info", person.info.as_deref().unwrap_or(""));

        let mut result = self.graph.execute(query).await?;
        let row = result.next().await?.unwrap();
        let id: i64 = row.get("id(p)").unwrap();

        Ok(id.to_string())
    }

    pub async fn get_person(&self, id: &str) -> Result<Option<Person>> {
        let query = query(
            "MATCH (p:Person) WHERE id(p) = $id 
         RETURN p.firstName, p.middleName, p.lastName, p.maidenName,
                p.nickName, p.fullName, p.born, p.deceased, p.info"
        )
            .param("id", id);

        let mut result = self.graph.execute(query).await?;

        if let Some(row) = result.next().await? {
            Ok(Some(Person {
                first_name: row.get("p.firstName").unwrap_or_default(),
                middle_name: row.get("p.middleName"),
                last_name: row.get("p.lastName").unwrap_or_default(),
                maiden_name: row.get("p.maidenName"),
                nick_name: row.get("p.nickName"),
                full_name: row.get("p.fullName").unwrap_or_default(),
                born: row.get("p.born"),
                deceased: row.get("p.deceased"),
                info: row.get("p.info"),
            }))
        } else {
            Ok(None)
        }
    }

    // Relationshantering
    pub async fn add_relationship(&self, rel: &Relationship) -> Result<()> {
        match rel {
            Relationship::Parent { from_id, to_id } => {
                let query = query(
                    "MATCH (parent:Person), (child:Person)
                 WHERE parent.id = $from_id AND child.id = $to_id
                 CREATE (parent)-[:PARENT_OF]->(child)"
                )
                    .param("from_id", from_id.as_str())
                    .param("to_id", to_id.as_str());

                self.graph.execute(query).await?;
            },
            Relationship::Marriage { from_id, to_id, wedding_date, divorce_date } => {
                let query = query(
                    "MATCH (p1:Person), (p2:Person)
                 WHERE p1.id = $from_id AND p2.id = $to_id
                 CREATE (p1)-[:MARRIED_TO {
                     wedding_date: $wedding_date,
                     divorce_date: $divorce_date
                 }]->(p2)"
                )
                    .param("from_id", from_id.as_str())
                    .param("to_id", to_id.as_str())
                    .param("wedding_date", wedding_date.map(|d| d.to_string()).unwrap_or_default())
                    .param("divorce_date", divorce_date.map(|d| d.to_string()).unwrap_or_default());

                self.graph.execute(query).await?;
            }
        }
        Ok(())
    }

    // Hämta släktträd
    pub async fn get_family_tree(&self, person_id: &str) -> Result<Vec<Person>> {
        let query = query(
            "MATCH (p:Person)-[*1..3]-(relative:Person)
             WHERE p.id = $person_id
             RETURN DISTINCT relative"
        )
        .param("person_id", person_id);

        let mut result = self.graph.execute(query).await?;
        let relatives = Vec::new();

        while let Some(_row) = result.next().await? {
            // TODO: Implementera konvertering från row till Person
        }

        Ok(relatives)
    }

    /// Imports data from a Cypher script file
    pub async fn import_from_file(&self, file_path: &str) -> Result<()> {
        let content = std::fs::read_to_string(file_path)?;
        
        // Split into individual statements
        for statement in content.split(';') {
            if !statement.trim().is_empty() {
                self.graph.execute(query(statement)).await?;
            }
        }
        
        Ok(())
    }

    /// Imports data from a .dump file
    pub async fn import_from_dump(&self, dump_path: &str) -> Result<()> {
        let output = std::process::Command::new("neo4j-admin")
            .arg("load")
            .arg("--from")
            .arg(dump_path)
            .output()?;

        if !output.status.success() {
            return Err(neo4rs::Error::from(std::io::Error::new(
                std::io::ErrorKind::Other,
                String::from_utf8_lossy(&output.stderr).to_string()
            )));
        }

        Ok(())
    }

    /// Imports person data from a CSV file
    /// 
    /// Expected CSV format:
    /// first_name,middle_name,last_name,maiden_name,nick_name,full_name,born,deceased,info
    pub async fn import_from_csv(&self, file_path: &str) -> Result<()> {
        let file = File::open(file_path).map_err(|e| neo4rs::Error::from(std::io::Error::new(
            std::io::ErrorKind::Other,
            e.to_string()
        )))?;
        let mut rdr = Reader::from_reader(file);

        for result in rdr.records() {
            let record = result.map_err(|e| neo4rs::Error::from(std::io::Error::new(
                std::io::ErrorKind::Other,
                e.to_string()
            )))?;

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
            })"
            )
                .param("first_name", &record[0])
                .param("middle_name", if record[1].is_empty() { "" } else { &record[1] })
                .param("last_name", &record[2])
                .param("maiden_name", if record[3].is_empty() { "" } else { &record[3] })
                .param("nick_name", if record[4].is_empty() { "" } else { &record[4] })
                .param("full_name", &record[5])
                .param("born", if record[6].is_empty() { "" } else { &record[6] })
                .param("deceased", if record[7].is_empty() { "" } else { &record[7] })
                .param("info", if record[8].is_empty() { "" } else { &record[8] });

            self.graph.execute(query).await?;
        }

        Ok(())
    }

    /// Imports relationship data from a CSV file
    /// 
    /// Expected CSV format:
    /// relationship_type,from_id,to_id,wedding_date,divorce_date
    pub async fn import_relationships_from_csv(&self, file_path: &str) -> Result<()> {
        let file = File::open(file_path).map_err(|e| neo4rs::Error::from(std::io::Error::new(
            std::io::ErrorKind::Other,
            e.to_string()
        )))?;
        let mut rdr = Reader::from_reader(file);

        for result in rdr.records() {
            let record = result.map_err(|e| neo4rs::Error::from(std::io::Error::new(
                std::io::ErrorKind::Other,
                e.to_string()
            )))?;

            match &record[0] {
                "PARENT" => {
                    let query = query(
                        "MATCH (parent:Person), (child:Person)
             WHERE id(parent) = $from_id AND id(child) = $to_id
             CREATE (parent)-[:PARENT_OF]->(child)"
                    )
                        .param("from_id", &record[1])
                        .param("to_id", &record[2]);

                    self.graph.execute(query).await?;
                },
                "MARRIAGE" => {
                    // ... resten av koden
                },
                _ => println!("Unknown relationship type: {}", record[0].to_string())
            }
        }
        
        Ok(())
    }

    pub async fn get_all_persons(&self) -> Result<Vec<Person>> {
        let query = query("MATCH (p:Person) RETURN p");
        let mut result = self.graph.execute(query).await?;
        let mut persons = Vec::new();

        while let Some(row) = result.next().await? {
            let person = Person {
                first_name: row.get("p.firstName").unwrap_or_default(),
                middle_name: row.get("p.middleName"),
                last_name: row.get("p.lastName").unwrap_or_default(),
                maiden_name: row.get("p.maidenName"),
                nick_name: row.get("p.nickName"),
                full_name: row.get("p.fullName").unwrap_or_default(),
                born: row.get("p.born"),
                deceased: row.get("p.deceased"),
                info: row.get("p.info"),
            };
            persons.push(person);
        }

        Ok(persons)
    }
}