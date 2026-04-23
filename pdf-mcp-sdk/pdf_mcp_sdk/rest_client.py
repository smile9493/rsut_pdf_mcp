"""
REST API Client for PDF Module
Alternative client using REST API instead of MCP protocol
"""

import json
import requests
from typing import Optional, List, Dict, Any, BinaryIO
from pathlib import Path

from .models import (
    ExtractionResult,
    StructuredResult,
    PageMetadata,
    AdapterInfo,
    CacheStats,
    PDFInfo,
)
from .exceptions import (
    PDFMCPError,
    ConnectionError,
    ToolExecutionError,
)


class PDFRestClient:
    """
    REST API client for PDF Module.
    
    Communicates via HTTP REST API endpoints.
    This is simpler than MCP but provides the same functionality.
    
    Example:
        client = PDFRestClient("http://localhost:8000")
        
        # Extract text from file
        with open("test.pdf", "rb") as f:
            result = client.extract_text(f, adapter="lopdf")
        print(result.text)
        
        # Or from path
        result = client.extract_text_from_path("/path/to/file.pdf")
        print(result.text)
    """
    
    def __init__(
        self,
        base_url: str = "http://localhost:8000",
        timeout: float = 30.0,
        headers: Optional[Dict[str, str]] = None,
    ):
        """
        Initialize REST client.
        
        Args:
            base_url: Base URL of REST API server (default: http://localhost:8000)
            timeout: Request timeout in seconds
            headers: Additional headers
        """
        self.base_url = base_url.rstrip("/")
        self.timeout = timeout
        self.headers = headers or {}
        self._session = requests.Session()
    
    def _post_file(
        self,
        endpoint: str,
        file: BinaryIO,
        filename: str = "document.pdf",
        **params,
    ) -> Any:
        """Post a file to an endpoint."""
        files = {"file": (filename, file, "application/pdf")}
        data = {k: v for k, v in params.items() if v is not None}
        
        try:
            response = self._session.post(
                f"{self.base_url}{endpoint}",
                files=files,
                data=data,
                headers=self.headers,
                timeout=self.timeout,
            )
            response.raise_for_status()
            return response
        except requests.exceptions.ConnectionError as e:
            raise ConnectionError(f"Failed to connect to REST API at {self.base_url}: {e}")
        except requests.exceptions.HTTPError as e:
            raise ToolExecutionError(endpoint, f"HTTP error: {e}")
    
    def health_check(self) -> bool:
        """Check if server is healthy."""
        try:
            response = self._session.get(
                f"{self.base_url}/api/v1/x2text/health",
                timeout=5,
            )
            return response.status_code == 200
        except:
            return False
    
    def extract_text(
        self,
        file: BinaryIO,
        filename: str = "document.pdf",
        adapter: str = "lopdf",
    ) -> ExtractionResult:
        """
        Extract text from a PDF file.
        
        Args:
            file: File-like object (opened in binary mode)
            filename: Filename for the upload
            adapter: Extraction engine (lopdf, pdf-extract, pdfium)
            
        Returns:
            ExtractionResult with text content
        """
        response = self._post_file(
            "/api/v1/x2text/extract",
            file,
            filename,
            adapter=adapter,
        )
        
        text = response.text
        
        # Try to get page count from info
        file.seek(0)
        try:
            info_response = self._post_file(
                "/api/v1/x2text/info",
                file,
                filename,
            )
            info = info_response.json()
            page_count = info.get("page_count", 0)
        except:
            page_count = 0
        
        return ExtractionResult(
            text=text,
            page_count=page_count,
            adapter=adapter,
            file_path=filename,
        )
    
    def extract_text_from_path(
        self,
        file_path: str,
        adapter: str = "lopdf",
    ) -> ExtractionResult:
        """
        Extract text from a file path.
        
        Args:
            file_path: Path to the PDF file
            adapter: Extraction engine
            
        Returns:
            ExtractionResult
        """
        path = Path(file_path)
        with open(path, "rb") as f:
            return self.extract_text(f, path.name, adapter)
    
    def extract_structured(
        self,
        file: BinaryIO,
        filename: str = "document.pdf",
        adapter: str = "lopdf",
    ) -> StructuredResult:
        """
        Extract structured data from PDF.
        
        Args:
            file: File-like object
            filename: Filename
            adapter: Extraction engine
            
        Returns:
            StructuredResult with page-by-page data
        """
        response = self._post_file(
            "/api/v1/x2text/extract-json",
            file,
            filename,
            adapter=adapter,
        )
        
        data = response.json()
        
        pages = []
        for page_data in data.get("pages", []):
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
            file_path=filename,
            metadata=data.get("metadata", {}),
        )
    
    def extract_structured_from_path(
        self,
        file_path: str,
        adapter: str = "lopdf",
    ) -> StructuredResult:
        """Extract structured data from file path."""
        path = Path(file_path)
        with open(path, "rb") as f:
            return self.extract_structured(f, path.name, adapter)
    
    def get_info(
        self,
        file: BinaryIO,
        filename: str = "document.pdf",
    ) -> PDFInfo:
        """
        Get PDF file information.
        
        Args:
            file: File-like object
            filename: Filename
            
        Returns:
            PDFInfo with metadata
        """
        response = self._post_file(
            "/api/v1/x2text/info",
            file,
            filename,
        )
        
        data = response.json()
        
        return PDFInfo(
            file_path=filename,
            page_count=data.get("page_count", 0),
            file_size_bytes=data.get("file_size", 0),
            is_encrypted=data.get("is_encrypted", False),
            is_linear=data.get("is_linear", False),
            metadata=data.get("metadata", {}),
        )
    
    def list_adapters(self) -> List[AdapterInfo]:
        """List available extraction engines."""
        try:
            response = self._session.get(
                f"{self.base_url}/api/v1/x2text/adapters",
                timeout=self.timeout,
            )
            response.raise_for_status()
            data = response.json()
            
            return [
                AdapterInfo(
                    id=a.get("id", ""),
                    name=a.get("name", ""),
                    description=a.get("description", ""),
                )
                for a in data
            ]
        except Exception as e:
            raise ConnectionError(f"Failed to list adapters: {e}")
    
    def get_cache_stats(self) -> CacheStats:
        """Get cache statistics."""
        try:
            response = self._session.get(
                f"{self.base_url}/api/v1/x2text/cache/stats",
                timeout=self.timeout,
            )
            response.raise_for_status()
            data = response.json()
            
            return CacheStats(
                size=data.get("size", 0),
                max_size=data.get("max_size", 0),
                hits=data.get("hits", 0),
                misses=data.get("misses", 0),
                hit_rate=data.get("hit_rate", 0.0),
            )
        except Exception as e:
            raise ConnectionError(f"Failed to get cache stats: {e}")
    
    def __enter__(self):
        return self
    
    def __exit__(self, exc_type, exc_val, exc_tb):
        self.close()
    
    def close(self):
        """Close the client."""
        self._session.close()
