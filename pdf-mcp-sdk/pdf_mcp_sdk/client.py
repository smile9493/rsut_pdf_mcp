"""
Synchronous MCP Client for PDF Module
Communicates with MCP server via HTTP/SSE transport
"""

import json
import requests
from typing import Optional, List, Dict, Any, Union
from contextlib import contextmanager

from .models import (
    ExtractionResult,
    StructuredResult,
    PageMetadata,
    KeywordSearchResult,
    KeywordMatch,
    KeywordFrequency,
    AdapterInfo,
    CacheStats,
)
from .exceptions import (
    PDFMCPError,
    ConnectionError,
    ToolExecutionError,
    InvalidParameterError,
)


class PDFMCPClient:
    """
    Synchronous client for PDF Module MCP Server.
    
    Communicates via HTTP POST to the MCP server's /message endpoint.
    
    Example:
        client = PDFMCPClient("http://localhost:8001")
        
        # Extract text
        result = client.extract_text("/path/to/file.pdf")
        print(result.text)
        
        # Search keywords
        search_result = client.search_keywords("/path/to/file.pdf", ["keyword1", "keyword2"])
        print(f"Found {search_result.total_matches} matches")
    """
    
    def __init__(
        self,
        base_url: str = "http://localhost:8001",
        timeout: float = 30.0,
        headers: Optional[Dict[str, str]] = None,
    ):
        """
        Initialize MCP client.
        
        Args:
            base_url: Base URL of MCP server (default: http://localhost:8001)
            timeout: Request timeout in seconds (default: 30)
            headers: Additional headers to send with requests
        """
        self.base_url = base_url.rstrip("/")
        self.timeout = timeout
        self.headers = headers or {}
        self._request_id = 0
        self._session = requests.Session()
        
    def _next_id(self) -> int:
        """Generate next request ID"""
        self._request_id += 1
        return self._request_id
    
    def _call_tool(self, tool_name: str, arguments: Dict[str, Any]) -> Any:
        """
        Call an MCP tool and return the result.
        
        Args:
            tool_name: Name of the tool to call
            arguments: Tool arguments
            
        Returns:
            Parsed JSON result from the tool
            
        Raises:
            ToolExecutionError: If tool execution fails
            ConnectionError: If connection to server fails
        """
        request_id = self._next_id()
        payload = {
            "jsonrpc": "2.0",
            "id": request_id,
            "method": "tools/call",
            "params": {
                "name": tool_name,
                "arguments": arguments,
            }
        }
        
        try:
            response = self._session.post(
                f"{self.base_url}/message",
                json=payload,
                headers={"Content-Type": "application/json", **self.headers},
                timeout=self.timeout,
            )
            response.raise_for_status()
        except requests.exceptions.ConnectionError as e:
            raise ConnectionError(f"Failed to connect to MCP server at {self.base_url}: {e}")
        except requests.exceptions.Timeout as e:
            raise ConnectionError(f"Request timed out after {self.timeout}s: {e}")
        except requests.exceptions.HTTPError as e:
            raise ConnectionError(f"HTTP error: {e}")
        
        result = response.json()
        
        # Check for JSON-RPC error
        if "error" in result:
            error = result["error"]
            raise ToolExecutionError(
                tool_name,
                error.get("message", "Unknown error"),
                error,
            )
        
        # Extract result from MCP response format
        if "result" in result and "content" in result["result"]:
            content = result["result"]["content"]
            if content and len(content) > 0:
                text = content[0].get("text", "")
                if text:
                    try:
                        return json.loads(text)
                    except json.JSONDecodeError:
                        return text
        
        return None
    
    def initialize(self) -> Dict[str, Any]:
        """
        Initialize connection to MCP server.
        
        Returns:
            Server capabilities and info
        """
        request_id = self._next_id()
        payload = {
            "jsonrpc": "2.0",
            "id": request_id,
            "method": "initialize",
            "params": {},
        }
        
        response = self._session.post(
            f"{self.base_url}/message",
            json=payload,
            headers={"Content-Type": "application/json", **self.headers},
            timeout=self.timeout,
        )
        result = response.json()
        return result.get("result", {})
    
    def list_tools(self) -> List[Dict[str, Any]]:
        """
        List all available MCP tools.
        
        Returns:
            List of tool definitions
        """
        request_id = self._next_id()
        payload = {
            "jsonrpc": "2.0",
            "id": request_id,
            "method": "tools/list",
            "params": {},
        }
        
        response = self._session.post(
            f"{self.base_url}/message",
            json=payload,
            headers={"Content-Type": "application/json", **self.headers},
            timeout=self.timeout,
        )
        result = response.json()
        return result.get("result", {}).get("tools", [])
    
    # ==================== PDF Operations ====================
    
    def extract_text(
        self,
        file_path: str,
        adapter: str = "pdf-extract",
    ) -> ExtractionResult:
        """
        Extract text content from a PDF file.
        
        Args:
            file_path: Absolute path to the PDF file
            adapter: Extraction engine (lopdf, pdf-extract, pdfium)
            
        Returns:
            ExtractionResult with extracted text
        """
        result = self._call_tool("extract_text", {
            "file_path": file_path,
            "adapter": adapter,
        })
        
        text = result if isinstance(result, str) else result.get("text", "") if result else ""
        
        # Get page count
        page_count = self.get_page_count(file_path)
        
        return ExtractionResult(
            text=text,
            page_count=page_count,
            adapter=adapter,
            file_path=file_path,
        )
    
    def extract_structured(
        self,
        file_path: str,
        adapter: str = "pdf-extract",
        enable_highlight: bool = False,
    ) -> StructuredResult:
        """
        Extract structured data with page info and positions.
        
        Args:
            file_path: Absolute path to the PDF file
            adapter: Extraction engine
            enable_highlight: Include highlight metadata
            
        Returns:
            StructuredResult with page-by-page data
        """
        result = self._call_tool("extract_structured", {
            "file_path": file_path,
            "adapter": adapter,
            "enable_highlight": enable_highlight,
        })
        
        pages = []
        for page_data in result.get("pages", []):
            pages.append(PageMetadata(
                page_number=page_data.get("page_number", 0),
                text=page_data.get("text", ""),
                bbox=page_data.get("bbox"),
                lines=page_data.get("lines", []),
            ))
        
        return StructuredResult(
            pages=pages,
            total_pages=len(pages),
            adapter=adapter,
            file_path=file_path,
            metadata=result.get("metadata", {}),
        )
    
    def get_page_count(self, file_path: str) -> int:
        """
        Get the number of pages in a PDF file.
        
        Args:
            file_path: Absolute path to the PDF file
            
        Returns:
            Number of pages
        """
        result = self._call_tool("get_page_count", {
            "file_path": file_path,
        })
        return int(result) if result else 0
    
    def search_keywords(
        self,
        file_path: str,
        keywords: List[str],
        case_sensitive: bool = False,
        context_length: int = 50,
    ) -> KeywordSearchResult:
        """
        Search for keywords in a PDF file.
        
        Args:
            file_path: Absolute path to the PDF file
            keywords: List of keywords to search
            case_sensitive: Case sensitive search
            context_length: Context characters around match
            
        Returns:
            KeywordSearchResult with matches
        """
        result = self._call_tool("search_keywords", {
            "file_path": file_path,
            "keywords": keywords,
            "case_sensitive": case_sensitive,
        })
        
        matches = []
        for match_data in result.get("matches", []):
            matches.append(KeywordMatch(
                keyword=match_data.get("keyword", ""),
                page_number=match_data.get("page_number", 0),
                text=match_data.get("text", ""),
                bbox=tuple(match_data["bbox"]) if match_data.get("bbox") else None,
                start_index=match_data.get("start_index", 0),
                end_index=match_data.get("end_index", 0),
                confidence=match_data.get("confidence", 1.0),
            ))
        
        return KeywordSearchResult(
            keywords=keywords,
            matches=matches,
            total_matches=result.get("total_matches", 0),
            pages_with_matches=result.get("pages_with_matches", []),
        )
    
    def extract_keywords(
        self,
        file_path: str,
        top_n: int = 10,
        min_length: int = 2,
        max_length: int = 20,
    ) -> List[KeywordFrequency]:
        """
        Auto-extract top keywords by frequency.
        
        Args:
            file_path: Absolute path to the PDF file
            top_n: Number of top keywords to return
            min_length: Minimum keyword length
            max_length: Maximum keyword length
            
        Returns:
            List of KeywordFrequency
        """
        result = self._call_tool("extract_keywords", {
            "file_path": file_path,
            "top_n": top_n,
        })
        
        keywords = []
        for item in result if isinstance(result, list) else []:
            if isinstance(item, list) and len(item) >= 2:
                keywords.append(KeywordFrequency(keyword=item[0], count=item[1]))
            elif isinstance(item, dict):
                keywords.append(KeywordFrequency(
                    keyword=item.get("keyword", ""),
                    count=item.get("count", 0),
                ))
        
        return keywords
    
    def list_adapters(self) -> List[AdapterInfo]:
        """
        List available PDF extraction engines.
        
        Returns:
            List of AdapterInfo
        """
        result = self._call_tool("list_adapters", {})
        
        adapters = []
        for adapter_data in result if isinstance(result, list) else []:
            adapters.append(AdapterInfo(
                id=adapter_data.get("id", ""),
                name=adapter_data.get("name", ""),
                description=adapter_data.get("description", ""),
            ))
        
        return adapters
    
    def get_cache_stats(self) -> CacheStats:
        """
        Get cache statistics.
        
        Returns:
            CacheStats with hit/miss info
        """
        result = self._call_tool("cache_stats", {})
        
        return CacheStats(
            size=result.get("size", 0),
            max_size=result.get("max_size", 0),
            hits=result.get("hits", 0),
            misses=result.get("misses", 0),
            hit_rate=result.get("hit_rate", 0.0),
        )
    
    def clear_cache(self) -> bool:
        """Clear the extraction cache."""
        # This would need to be implemented on the server side
        raise NotImplementedError("clear_cache not yet implemented on server")
    
    # ==================== Context Manager Support ====================
    
    def __enter__(self):
        return self
    
    def __exit__(self, exc_type, exc_val, exc_tb):
        self.close()
    
    def close(self):
        """Close the client and release resources."""
        self._session.close()
