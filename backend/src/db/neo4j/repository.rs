use neo4rs::*;
use std::error::Error;

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
}