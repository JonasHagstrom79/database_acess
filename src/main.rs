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

// Struct representing an employee with basic information
struct Employee {
    employee_id: i32,
    name: String,
    salary: f64,
    region: String
}

// Implementation block for Employee struct
impl Employee {
    // Print employee details in a formatted way
    fn print(&self) {
        println!("{} {} {} {}", self.employee_id, self.name, self.salary, self.region);
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // Configure Neo4j connection
    let uri = "bolt://localhost:7687";
    let config = ConfigBuilder::default()
        .uri(uri)
        .user("neo4j")
        .password("password")
        .db("neo4j")  // Use default database
        .build()?;

    // Establish connection to Neo4j
    let graph = Graph::connect(config).await?;

    // Clear any existing Employee nodes to avoid duplicates
    graph.run(query("MATCH (e:Employee) DELETE e")).await?;

    // Create a vector of sample employees
    let employees = vec![
        Employee { employee_id: 1, name: String::from("Andy"),  salary: 25_000.0, region: String::from("South Wales") },
        Employee { employee_id: 2, name: String::from("Jayne"), salary: 35_000.0, region: String::from("South Wales") },
        Employee { employee_id: 3, name: String::from("Emily"), salary: 45_000.0, region: String::from("Scotland") },
        Employee { employee_id: 4, name: String::from("Tom"),   salary: 55_000.0, region: String::from("London") }
    ];

    // Create Employee nodes in Neo4j
    for employee in &employees {
        // Construct Cypher query with parameters
        let query = query(
            "CREATE (e:Employee {employee_id: $id, name: $name, salary: $salary, region: $region})"
        )
            .param("id", employee.employee_id)
            .param("name", &*employee.name)     // Dereference String to str
            .param("salary", employee.salary)
            .param("region", &*employee.region); // Dereference String to str

        // Execute the query
        graph.run(query).await?;
    }

    // Retrieve all employees from Neo4j
    let mut result = graph.execute(
        query("MATCH (e:Employee) RETURN e.employee_id, e.name, e.salary, e.region ORDER BY e.employee_id")
    ).await?;

    // Process and print each employee
    while let Some(row) = result.next().await? {
        let employee = Employee {
            // Neo4j stores integers as i64, so we convert to i32
            employee_id: row.get::<i64>("e.employee_id").unwrap() as i32,
            name: row.get::<String>("e.name").unwrap(),
            salary: row.get::<f64>("e.salary").unwrap(),
            region: row.get::<String>("e.region").unwrap(),
        };
        employee.print();
    }

    Ok(())
}