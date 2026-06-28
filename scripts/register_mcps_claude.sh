!# /bin/bash


# Doc-retrieval MCP (port 8000)
claude mcp add --transport http --scope project doc-retrieval http://localhost:8000/mcp
# Jira bridge MCP (port 8001)
claude mcp add --transport http --scope project jira-bridge http://localhost:8001/mcp
# COde Graph MCP (port 8765)
claude mcp add \
  --transport http \
  --scope project \
  code-graph \
  http://localhost:8765/mcp