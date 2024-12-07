# Build backend
FROM rust:1.70 as builder
WORKDIR /usr/src/app
COPY . .
RUN cargo build --release

# Runtime
FROM debian:bullseye-slim
COPY --from=builder /usr/src/app/target/release/database_access /usr/local/bin/
CMD ["database_access"]

# Neo4j setup
FROM neo4j:5.12.0

ENV NEO4J_AUTH=neo4j/password
ENV NEO4J_PLUGINS='["graph-data-science", "apoc"]'

VOLUME ["/data", "/logs"]

EXPOSE 7474 7687