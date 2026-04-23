"""
Custom exceptions for PDF MCP Client SDK
"""


class PDFMCPError(Exception):
    """Base exception for all PDF MCP errors"""
    
    def __init__(self, message: str, details: dict = None):
        super().__init__(message)
        self.message = message
        self.details = details or {}
    
    def __str__(self):
        if self.details:
            return f"{self.message} - Details: {self.details}"
        return self.message


class ConnectionError(PDFMCPError):
    """Raised when connection to MCP server fails"""
    pass


class ToolExecutionError(PDFMCPError):
    """Raised when tool execution fails"""
    
    def __init__(self, tool_name: str, message: str, details: dict = None):
        super().__init__(message, details)
        self.tool_name = tool_name


class FileNotFoundError(PDFMCPError):
    """Raised when PDF file is not found or inaccessible"""
    
    def __init__(self, file_path: str, message: str = None):
        super().__init__(message or f"File not found: {file_path}")
        self.file_path = file_path


class InvalidParameterError(PDFMCPError):
    """Raised when invalid parameters are provided"""
    
    def __init__(self, param_name: str, value: any, message: str = None):
        super().__init__(message or f"Invalid parameter '{param_name}': {value}")
        self.param_name = param_name
        self.value = value


class TimeoutError(PDFMCPError):
    """Raised when operation times out"""
    pass


class AuthenticationError(PDFMCPError):
    """Raised when authentication fails"""
    pass
