import structlog
from .server import mcp

structlog.configure(
    processors=[
        structlog.processors.TimeStamper(fmt="iso"),
        structlog.processors.add_log_level,
        structlog.processors.JSONRenderer(),
    ]
)

if __name__ == "__main__":
    mcp.run(transport="streamable-http")
