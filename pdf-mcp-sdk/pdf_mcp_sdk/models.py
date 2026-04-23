"""
Data models for PDF MCP Client SDK
"""

from dataclasses import dataclass, field
from typing import List, Optional, Dict, Any


@dataclass
class PageMetadata:
    """Metadata for a single PDF page"""
    page_number: int
    text: str
    bbox: Optional[List[float]] = None
    lines: List[Dict[str, Any]] = field(default_factory=list)


@dataclass
class ExtractionResult:
    """Result of text extraction"""
    text: str
    page_count: int
    adapter: str
    file_path: str
    processing_time_ms: Optional[float] = None


@dataclass
class StructuredResult:
    """Result of structured extraction"""
    pages: List[PageMetadata]
    total_pages: int
    adapter: str
    file_path: str
    metadata: Dict[str, Any] = field(default_factory=dict)


@dataclass
class KeywordMatch:
    """A single keyword match"""
    keyword: str
    page_number: int
    text: str
    bbox: Optional[tuple] = None
    start_index: int = 0
    end_index: int = 0
    confidence: float = 1.0


@dataclass
class KeywordSearchResult:
    """Result of keyword search"""
    keywords: List[str]
    matches: List[KeywordMatch]
    total_matches: int
    pages_with_matches: List[int]


@dataclass
class KeywordFrequency:
    """Keyword with its frequency"""
    keyword: str
    count: int


@dataclass
class AdapterInfo:
    """Information about a PDF extraction adapter"""
    id: str
    name: str
    description: str


@dataclass
class CacheStats:
    """Cache statistics"""
    size: int
    max_size: int
    hits: int
    misses: int
    hit_rate: float


@dataclass
class PDFInfo:
    """PDF file information"""
    file_path: str
    page_count: int
    file_size_bytes: int
    is_encrypted: bool = False
    is_linear: bool = False
    metadata: Dict[str, Any] = field(default_factory=dict)
