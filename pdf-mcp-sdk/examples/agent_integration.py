#!/usr/bin/env python3
"""
Example: Using PDF MCP Client in an AI Agent
"""

from typing import List, Dict, Any
from pdf_mcp_sdk import PDFMCPClient, ExtractionResult


class PDFAgent:
    """
    Example AI Agent that uses PDF MCP tools.
    
    This demonstrates how an agent can integrate PDF processing
    capabilities using the MCP client SDK.
    """
    
    def __init__(self, mcp_url: str = "http://localhost:8001"):
        self.pdf_client = PDFMCPClient(mcp_url)
        self.conversation_history: List[Dict[str, str]] = []
    
    def process_query(self, query: str, pdf_path: str = None) -> str:
        """
        Process a user query about a PDF document.
        
        Args:
            query: User's question
            pdf_path: Path to PDF file (optional)
            
        Returns:
            Agent's response
        """
        # If query is about PDF content
        if pdf_path and self._is_pdf_query(query):
            return self._handle_pdf_query(query, pdf_path)
        
        return "I can help you analyze PDF documents. Please provide a PDF file path."
    
    def _is_pdf_query(self, query: str) -> bool:
        """Check if query is about PDF content"""
        keywords = ["pdf", "document", "page", "text", "content", "search", "find"]
        return any(kw in query.lower() for kw in keywords)
    
    def _handle_pdf_query(self, query: str, pdf_path: str) -> str:
        """Handle PDF-related queries"""
        
        # Get basic info
        page_count = self.pdf_client.get_page_count(pdf_path)
        
        # Determine what the user wants
        query_lower = query.lower()
        
        if "summarize" in query_lower or "summary" in query_lower:
            return self._summarize_pdf(pdf_path, page_count)
        
        elif "search" in query_lower or "find" in query_lower:
            # Extract search terms from query
            terms = self._extract_search_terms(query)
            return self._search_in_pdf(pdf_path, terms)
        
        elif "keyword" in query_lower:
            return self._extract_keywords(pdf_path)
        
        elif "page" in query_lower:
            return f"The document has {page_count} pages."
        
        else:
            # Default: extract and show preview
            return self._show_preview(pdf_path, page_count)
    
    def _summarize_pdf(self, pdf_path: str, page_count: int) -> str:
        """Generate a summary of the PDF"""
        result = self.pdf_client.extract_text(pdf_path)
        
        # Simple summary (in real agent, this would use LLM)
        text = result.text
        word_count = len(text.split())
        
        summary = f"""PDF Summary:
- File: {pdf_path}
- Pages: {page_count}
- Word count: ~{word_count}
- Characters: {len(text)}

Preview (first 500 chars):
{text[:500]}...
"""
        return summary
    
    def _search_in_pdf(self, pdf_path: str, terms: List[str]) -> str:
        """Search for terms in PDF"""
        result = self.pdf_client.search_keywords(pdf_path, terms)
        
        response = f"Search results for {terms}:\n"
        response += f"- Total matches: {result.total_matches}\n"
        response += f"- Pages with matches: {result.pages_with_matches}\n\n"
        
        for match in result.matches[:5]:  # Show first 5 matches
            response += f"Page {match.page_number}: ...{match.text}...\n"
        
        if result.total_matches > 5:
            response += f"\n... and {result.total_matches - 5} more matches"
        
        return response
    
    def _extract_keywords(self, pdf_path: str) -> str:
        """Extract top keywords from PDF"""
        keywords = self.pdf_client.extract_keywords(pdf_path, top_n=10)
        
        response = "Top keywords:\n"
        for kw in keywords:
            response += f"- {kw.keyword}: {kw.count} occurrences\n"
        
        return response
    
    def _show_preview(self, pdf_path: str, page_count: int) -> str:
        """Show a preview of the PDF"""
        result = self.pdf_client.extract_text(pdf_path)
        
        return f"""PDF Preview:
- File: {pdf_path}
- Pages: {page_count}

Content preview:
{result.text[:1000]}...
"""
    
    def _extract_search_terms(self, query: str) -> List[str]:
        """Extract search terms from query (simple implementation)"""
        # In a real agent, this would use NLP or LLM
        words = query.split()
        terms = [w for w in words if len(w) > 3 and w.isalpha()]
        return terms[:5]  # Max 5 terms
    
    def close(self):
        """Clean up resources"""
        self.pdf_client.close()


# Example usage
if __name__ == "__main__":
    agent = PDFAgent()
    
    try:
        # Example queries
        pdf_path = "/app/data/test.pdf"
        
        print("Query: How many pages?")
        print(agent.process_query("How many pages?", pdf_path))
        
        print("\nQuery: Summarize the document")
        print(agent.process_query("Summarize the document", pdf_path))
        
        print("\nQuery: Search for MarkDown")
        print(agent.process_query("Search for MarkDown", pdf_path))
        
    finally:
        agent.close()
