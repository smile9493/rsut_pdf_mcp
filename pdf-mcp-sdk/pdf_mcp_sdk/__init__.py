"""
PDF MCP Client SDK
A Python SDK for interacting with PDF Module MCP Server

Provides both MCP (Model Context Protocol) and REST API clients.
"""

from .client import PDFMCPClient
from .rest_client import PDFRestClient
from .models import (
    ExtractionResult,
    StructuredResult,
    PageMetadata,
    KeywordSearchResult,
    KeywordMatch,
    AdapterInfo,
    CacheStats,
)
from .exceptions import (
    PDFMCPError,
    ConnectionError,
    ToolExecutionError,
    FileNotFoundError,
    InvalidParameterError,
)

# Async client is optional (requires aiohttp)
try:
    from .async_client import AsyncPDFMCPClient
except ImportError:
    AsyncPDFMCPClient = None

__version__ = "0.1.0"
__all__ = [
    # Clients
    "PDFMCPClient",
    "AsyncPDFMCPClient",
    "PDFRestClient",
    # Models
    "ExtractionResult",
    "StructuredResult",
    "PageMetadata",
    "KeywordSearchResult",
    "KeywordMatch",
    "AdapterInfo",
    "CacheStats",
    # Exceptions
    "PDFMCPError",
    "ConnectionError",
    "ToolExecutionError",
    "FileNotFoundError",
    "InvalidParameterError",
]
