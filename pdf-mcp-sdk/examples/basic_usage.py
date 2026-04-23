#!/usr/bin/env python3
"""
Basic usage examples for PDF MCP Client SDK
"""

from pdf_mcp_sdk import PDFMCPClient, PDFRestClient

# ==================== MCP Client Examples ====================

def mcp_client_example():
    """Using MCP client (recommended for Agent integration)"""
    
    # Create client
    client = PDFMCPClient("http://localhost:8001")
    
    try:
        # Initialize and check connection
        info = client.initialize()
        print(f"Connected to: {info.get('serverInfo', {}).get('name', 'Unknown')}")
        
        # List available tools
        tools = client.list_tools()
        print(f"\nAvailable tools: {len(tools)}")
        for tool in tools:
            print(f"  - {tool['name']}: {tool['description']}")
        
        # List extraction engines
        adapters = client.list_adapters()
        print(f"\nAvailable adapters:")
        for adapter in adapters:
            print(f"  - {adapter.id}: {adapter.description}")
        
        # Extract text
        file_path = "/app/data/test.pdf"
        print(f"\nExtracting text from: {file_path}")
        result = client.extract_text(file_path, adapter="pdf-extract")
        print(f"  Pages: {result.page_count}")
        print(f"  Text length: {len(result.text)} characters")
        print(f"  Preview: {result.text[:200]}...")
        
        # Get page count
        pages = client.get_page_count(file_path)
        print(f"\nPage count: {pages}")
        
        # Search keywords
        keywords = ["MarkDown", "标题", "链接"]
        print(f"\nSearching for keywords: {keywords}")
        search_result = client.search_keywords(file_path, keywords)
        print(f"  Total matches: {search_result.total_matches}")
        print(f"  Pages with matches: {search_result.pages_with_matches}")
        
        # Extract keywords
        print(f"\nExtracting top keywords...")
        top_keywords = client.extract_keywords(file_path, top_n=10)
        for kw in top_keywords:
            print(f"  - {kw.keyword}: {kw.count}")
        
        # Cache stats
        stats = client.get_cache_stats()
        print(f"\nCache statistics:")
        print(f"  Size: {stats.size}/{stats.max_size}")
        print(f"  Hits: {stats.hits}, Misses: {stats.misses}")
        print(f"  Hit rate: {stats.hit_rate:.2%}")
        
    finally:
        client.close()


def context_manager_example():
    """Using context manager for automatic cleanup"""
    
    with PDFMCPClient("http://localhost:8001") as client:
        result = client.extract_text("/app/data/test.pdf")
        print(f"Extracted {len(result.text)} characters")


# ==================== REST Client Examples ====================

def rest_client_example():
    """Using REST API client"""
    
    with PDFRestClient("http://localhost:8000") as client:
        # Health check
        if client.health_check():
            print("Server is healthy")
        
        # Extract from file path
        result = client.extract_text_from_path("/path/to/file.pdf")
        print(f"Extracted: {len(result.text)} characters")
        
        # Extract from file object
        with open("test.pdf", "rb") as f:
            result = client.extract_text(f, adapter="lopdf")
            print(f"Extracted: {len(result.text)} characters")


# ==================== Async Client Examples ====================

async def async_client_example():
    """Using async client"""
    
    from pdf_mcp_sdk import AsyncPDFMCPClient
    
    async with AsyncPDFMCPClient("http://localhost:8001") as client:
        # Extract text
        result = await client.extract_text("/app/data/test.pdf")
        print(f"Extracted: {len(result.text)} characters")
        
        # Search keywords
        search = await client.search_keywords("/app/data/test.pdf", ["test"])
        print(f"Found: {search.total_matches} matches")


if __name__ == "__main__":
    print("=" * 60)
    print("PDF MCP Client SDK - Basic Usage Examples")
    print("=" * 60)
    
    # Run MCP client example
    print("\n### MCP Client Example ###\n")
    try:
        mcp_client_example()
    except Exception as e:
        print(f"Error: {e}")
    
    print("\n" + "=" * 60)
