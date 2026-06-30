"""Configuration, read from environment variables with defaults."""
import os
from dataclasses import dataclass


@dataclass(frozen=True)
class Config:
    channels_dir: str = os.getenv("CHANNELS_DIR", "/channels")
    host: str = os.getenv("MCP_HOST", "0.0.0.0")
    port: int = int(os.getenv("MCP_PORT", "8766"))


config = Config()
