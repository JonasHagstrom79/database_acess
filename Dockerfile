FROM neo4j:5.12.0

ENV NEO4J_AUTH=neo4j/password
ENV NEO4J_PLUGINS='["graph-data-science", "apoc"]'

VOLUME ["/data", "/logs"]

EXPOSE 7474 7687
