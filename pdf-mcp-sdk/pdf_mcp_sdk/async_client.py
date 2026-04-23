"""
Asynchronous MCP Client for PDF Module
"""

import json
import aiohttp
from typing import Optional, List, Dict, Any

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
)


class AsyncPDFMCPClient:
    """
    Asynchronous client for PDF Module MCP Server.
    
    Example:
        async with AsyncPDFMCPClient("http://localhost:8001") as client:
            result = await client.extract_text("/path/to/file.pdf")
            print(result.text)
    """
    
    def __init__(
        self,
        base_url: str = "http://localhost:8001",
        timeout: float = 30.0,
        headers: Optional[Dict[str, str]] = None,
    ):
        self.base_url = base_url.rstrip("/")
        self.timeout = aiohttp.ClientTimeout(total=timeout)
        self.headers = headers or {}
        self._request_id = 0
        self._session: Optional[aiohttp.ClientSession] = None
    
    def _next_id(self) -> int:
        self._request_id += 1
        return self._request_id
    
    async def _get_session(self) -> aiohttp.ClientSession:
        if self._session is None or self._session.closed:
            self._session = aiohttp.ClientSession(timeout=self.timeout)
        return self._session
    
    async def _call_tool(self, tool_name: str, arguments: Dict[str, Any]) -> Any:
        request_id = self._next_id()
        payload = {
            "jsonrpc": "2.0",
            "id": request_id,
            "method": "tools/call",
            "params": {"name": tool_name, "arguments": arguments},
        }
        
        session = await self._get_session()
        
        try:
            async with session.post(
                f"{self.base_url}/message",
                json=payload,
                headers={"Content-Type": "application/json", **self.headers},
            ) as response:
                response.raise_for_status()
                result = await response.json()
        except aiohttp.ClientError as e:
            raise ConnectionError(f"Connection error: {e}")
        
        if "error" in result:
            error = result["error"]
            raise ToolExecutionError(tool_name, error.get("message", "Unknown error"), error)
        
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
    
    async def extract_text(self, file_path: str, adapter: str = "pdf-extract") -> ExtractionResult:
        result = self._call_tool("extract_text", {"file_path": file_path, "adapter": adapter})
        page_count = self.get_page_count(file_path)
        
        result_text = await result
        pages = await page_count
        
        text = result_text if isinstance(result_text, str) else result_text.get("text", "") if result_text else ""
        
        return ExtractionResult(
            text=text,
            page_count=pages,
            adapter=adapter,
            file_path=file_path,
        )
    
    async def get_page_count(self, file_path: str) -> int:
        result = await self._call_tool("get_page_count", {"file_path": file_path})
        return int(result) if result else 0
    
    async def search_keywords(
        self,
        file_path: str,
        keywords: List[str],
        case_sensitive: bool = False,
    ) -> KeywordSearchResult:
        result = await self._call_tool("search_keywords", {
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
            ))
        
        return KeywordSearchResult(
            keywords=keywords,
            matches=matches,
            total_matches=result.get("total_matches", 0),
            pages_with_matches=result.get("pages_with_matches", []),
        )
    
    async def list_adapters(self) -> List[AdapterInfo]:
        result = await self._call_tool("list_adapters", {})
        return [
            AdapterInfo(
                id=a.get("id", ""),
                name=a.get("name", ""),
                description=a.get("description", ""),
            )
            for a in result if isinstance(result, list)
        ]
    
    async def get_cache_stats(self) -> CacheStats:
        result = await self._call_tool("cache_stats", {})
        return CacheStats(
            size=result.get("size", 0),
            max_size=result.get("max_size", 0),
            hits=result.get("hits", 0),
            misses=result.get("misses", 0),
            hit_rate=result.get("hit_rate", 0.0),
        )
    
    async def close(self):
        if self._session and not self._session.closed:
            await self._session.close()
    
    async def __aenter__(self):
        return self
    
    async def __aexit__(self, exc_type, exc_val, exc_tb):
        await self.close()
