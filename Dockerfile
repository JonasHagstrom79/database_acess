# Pull MySQL from Docker Hub (if not already in local Docker registry).
FROM neo4j:5.12.0

# MySQL will run on port 3306 within the container.
EXPOSE 3306
ENV NEO4J_AUTH=neo4j/password
ENV NEO4J_PLUGINS=["graph-data-scinece", "apoc"]

# Set an environment variable, which MySQL will look for.
EXPOSE 7474 7687
