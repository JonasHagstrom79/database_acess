// This application demonstrates how to use Neo4j graph database with Rust.
// It creates Employee nodes and performs basic CRUD operations.
//
// To run the demo:
//
// 1. Build and run the Neo4j Docker container:
//       docker build -t neo4j-image .
//       docker run --name neo4j-container -d -p 7474:7474 -p 7687:7687 neo4j-image
//
// 2. Run the Rust application:
//       cargo run
//
// 3. Clean up Docker resources:
//       docker container rm -f neo4j-container
//       docker image rm -f neo4j-image

use neo4rs::*;
use std::error::Error;
mod app_schema_generator;
use app_schema_generator::generate_app_puml;
mod error;
use error::AppError;

// Define the Employee struct to represent employee data
// This struct maps directly to Neo4j node properties
struct Employee {
    employee_id: i32,
    name: String,
    salary: f64,
    region: String
}

// Implementation of Employee methods
impl Employee {
    // Display employee information in a formatted way
    fn print(&self) {
        println!("{} {} {} {}", self.employee_id, self.name, self.salary, self.region);
    }
}

// Main async function using tokio runtime
#[tokio::main]
async fn main() -> Result<(), AppError> {
    // Set up Neo4j connection configuration
    let config = ConfigBuilder::default()
        .uri("bolt://localhost:7687")
        .user("neo4j")
        .password("password")
        .db("neo4j")  // Use default database
        .build()?;

    // Create connection to Neo4j database
    let graph = Graph::connect(config).await?;

    // Clean up: Remove any existing Employee nodes
    graph.run(query("MATCH (e:Employee) DELETE e")).await?;

    // Create test data: Vector of Employee instances
    let employees = vec![
        Employee { employee_id: 1, name: String::from("Andy"),  salary: 25_000.0, region: String::from("South Wales") },
        Employee { employee_id: 2, name: String::from("Jayne"), salary: 35_000.0, region: String::from("South Wales") },
        Employee { employee_id: 3, name: String::from("Emily"), salary: 45_000.0, region: String::from("Scotland") },
        Employee { employee_id: 4, name: String::from("Tom"),   salary: 55_000.0, region: String::from("London") }
    ];

    // Insert employees into Neo4j database
    for employee in &employees {
        // Create Cypher query with parameterized values for safety
        let query = query(
            "CREATE (e:Employee {employee_id: $id, name: $name, salary: $salary, region: $region})"
        )
            .param("id", employee.employee_id)
            .param("name", &*employee.name)     // Convert String to &str for Neo4j
            .param("salary", employee.salary)
            .param("region", &*employee.region); // Convert String to &str for Neo4j

        // Execute the creation query
        graph.run(query).await?;
    }

    // Retrieve all employees from database, ordered by ID
    let mut result = graph.execute(
        query("MATCH (e:Employee) RETURN e.employee_id, e.name, e.salary, e.region ORDER BY e.employee_id")
    ).await?;

    // Process query results and display each employee
    while let Some(row) = result.next().await? {
        let employee = Employee {
            employee_id: row.get::<i64>("e.employee_id").unwrap() as i32, // Convert Neo4j's i64 to i32
            name: row.get::<String>("e.name").unwrap(),
            salary: row.get::<f64>("e.salary").unwrap(),
            region: row.get::<String>("e.region").unwrap(),
        };
        employee.print();
    }

    // Generate PlantUML diagram of application structure
    generate_app_puml()?;

    Ok(())
}