# PDF MCP Client SDK

Python SDK for interacting with PDF Module MCP Server.

## Installation

```bash
# Basic installation
pip install pdf-mcp-sdk

# With async support
pip install pdf-mcp-sdk[async]
```

## Quick Start

### MCP Client (Recommended for Agent Integration)

```python
from pdf_mcp_sdk import PDFMCPClient

# Connect to MCP server
client = PDFMCPClient("http://localhost:8001")

# Extract text
result = client.extract_text("/path/to/file.pdf")
print(f"Extracted {len(result.text)} characters from {result.page_count} pages")

# Search keywords
search = client.search_keywords("/path/to/file.pdf", ["contract", "agreement"])
print(f"Found {search.total_matches} matches")

# List available engines
adapters = client.list_adapters()
for adapter in adapters:
    print(f"- {adapter.id}: {adapter.description}")

# Get cache stats
stats = client.get_cache_stats()
print(f"Cache hit rate: {stats.hit_rate:.2%}")

client.close()
```

### Async Client

```python
import asyncio
from pdf_mcp_sdk import AsyncPDFMCPClient

async def main():
    async with AsyncPDFMCPClient("http://localhost:8001") as client:
        result = await client.extract_text("/path/to/file.pdf")
        print(result.text)

asyncio.run(main())
```

### REST API Client

```python
from pdf_mcp_sdk import PDFRestClient

with PDFRestClient("http://localhost:8000") as client:
    # From file path
    result = client.extract_text_from_path("/path/to/file.pdf")
    print(result.text)
    
    # From file object
    with open("test.pdf", "rb") as f:
        result = client.extract_text(f)
        print(result.text)
```

## Cursor/Claude Desktop Integration

### Cursor Configuration

Create or edit `.cursor/mcp.json`:

```json
{
  "mcpServers": {
    "pdf-module": {
      "command": "docker",
      "args": ["exec", "-i", "pdf-mcp-server", "pdf-mcp", "serve", "--transport", "stdio"]
    }
  }
}
```

Or with local binary:

```json
{
  "mcpServers": {
    "pdf-module": {
      "command": "/path/to/pdf-mcp",
      "args": ["serve", "--transport", "stdio"]
    }
  }
}
```

### Claude Desktop Configuration

Edit `~/Library/Application Support/Claude/claude_desktop_config.json` (macOS):

```json
{
  "mcpServers": {
    "pdf-module": {
      "command": "docker",
      "args": ["exec", "-i", "pdf-mcp-server", "pdf-mcp", "serve", "--transport", "stdio"]
    }
  }
}
```

After configuration, restart Cursor/Claude and the PDF tools will be automatically available.

## Available Tools

| Tool | Description |
|------|-------------|
| `extract_text` | Extract text content from PDF |
| `extract_structured` | Extract structured data with page info |
| `get_page_count` | Get number of pages |
| `search_keywords` | Search for keywords |
| `extract_keywords` | Auto-extract top keywords |
| `list_adapters` | List extraction engines |
| `cache_stats` | Get cache statistics |

## Extraction Engines

| Engine | ID | Best For |
|--------|-----|----------|
| Lopdf | `lopdf` | General purpose, layout-aware |
| PDF Extract | `pdf-extract` | Fast text extraction |
| PDFium | `pdfium` | High compatibility |

## Error Handling

```python
from pdf_mcp_sdk import PDFMCPClient, PDFMCPError, ConnectionError

client = PDFMCPClient("http://localhost:8001")

try:
    result = client.extract_text("/path/to/file.pdf")
except ConnectionError as e:
    print(f"Failed to connect: {e}")
except PDFMCPError as e:
    print(f"Error: {e}")
```

## API Reference

### PDFMCPClient

```python
class PDFMCPClient:
    def __init__(self, base_url: str = "http://localhost:8001", timeout: float = 30.0)
    
    def extract_text(self, file_path: str, adapter: str = "pdf-extract") -> ExtractionResult
    def extract_structured(self, file_path: str, adapter: str = "pdf-extract") -> StructuredResult
    def get_page_count(self, file_path: str) -> int
    def search_keywords(self, file_path: str, keywords: List[str], case_sensitive: bool = False) -> KeywordSearchResult
    def extract_keywords(self, file_path: str, top_n: int = 10) -> List[KeywordFrequency]
    def list_adapters() -> List[AdapterInfo]
    def get_cache_stats() -> CacheStats
```

### Models

```python
@dataclass
class ExtractionResult:
    text: str
    page_count: int
    adapter: str
    file_path: str

@dataclass
class KeywordSearchResult:
    keywords: List[str]
    matches: List[KeywordMatch]
    total_matches: int
    pages_with_matches: List[int]

@dataclass
class AdapterInfo:
    id: str
    name: str
    description: str
```

## License

MIT License
