#!/usr/bin/env python3
"""
PDF Module 客户端示例
演示如何通过REST API调用远程PDF Module服务
"""

import requests
import json
from pathlib import Path

class PDFModuleClient:
    """PDF Module REST API 客户端"""

    def __init__(self, host="localhost", port=8000):
        """
        初始化客户端

        Args:
            host: 服务器IP地址
            port: REST API端口
        """
        self.base_url = f"http://{host}:{port}/api/v1/x2text"

    def health_check(self):
        """健康检查"""
        try:
            response = requests.get(f"{self.base_url}/health")
            return response.status_code == 200
        except:
            return False

    def extract_text(self, pdf_path, adapter="auto"):
        """
        提取PDF文本

        Args:
            pdf_path: PDF文件路径
            adapter: 提取引擎 (auto/lopdf/pdf-extract/pdfium)

        Returns:
            dict: 提取结果
        """
        with open(pdf_path, 'rb') as f:
            response = requests.post(
                f"{self.base_url}/extract",
                files={'file': f},
                params={'adapter': adapter}
            )
        return response.json()

    def search_keywords(self, pdf_path, keywords, case_sensitive=False):
        """
        搜索关键词

        Args:
            pdf_path: PDF文件路径
            keywords: 关键词列表
            case_sensitive: 是否区分大小写

        Returns:
            dict: 搜索结果
        """
        with open(pdf_path, 'rb') as f:
            response = requests.post(
                f"{self.base_url}/search",
                files={'file': f},
                data={
                    'keywords': ','.join(keywords),
                    'case_sensitive': str(case_sensitive).lower()
                }
            )
        return response.json()

    def extract_keywords(self, pdf_path, top_n=10):
        """
        提取高频关键词

        Args:
            pdf_path: PDF文件路径
            top_n: 返回前N个关键词

        Returns:
            dict: 关键词结果
        """
        with open(pdf_path, 'rb') as f:
            response = requests.post(
                f"{self.base_url}/keywords",
                files={'file': f},
                params={'top_n': top_n}
            )
        return response.json()

    def get_page_count(self, pdf_path):
        """
        获取PDF页数

        Args:
            pdf_path: PDF文件路径

        Returns:
            int: 页数
        """
        with open(pdf_path, 'rb') as f:
            response = requests.post(
                f"{self.base_url}/page-count",
                files={'file': f}
            )
        return response.json().get('page_count', 0)

    def list_adapters(self):
        """列出可用的提取引擎"""
        response = requests.get(f"{self.base_url}/adapters")
        return response.json()


def main():
    """使用示例"""

    # 创建客户端 - 连接到远程服务器
    # 将 YOUR_SERVER_IP 替换为你的服务器IP
    client = PDFModuleClient(host="YOUR_SERVER_IP", port=8000)

    # 健康检查
    if client.health_check():
        print("✅ 服务器连接正常")
    else:
        print("❌ 服务器连接失败")
        return

    # 列出可用引擎
    print("\n📋 可用引擎:")
    adapters = client.list_adapters()
    for adapter in adapters:
        print(f"  - {adapter['name']}: {adapter['description']}")

    # PDF文件路径
    pdf_file = "example.pdf"

    if not Path(pdf_file).exists():
        print(f"\n⚠️  文件不存在: {pdf_file}")
        print("请将 example.pdf 放在当前目录")
        return

    # 提取文本
    print(f"\n📄 提取文本: {pdf_file}")
    result = client.extract_text(pdf_file)
    print(f"  页数: {result.get('page_count', 0)}")
    print(f"  字符数: {len(result.get('text', ''))}")
    print(f"  引擎: {result.get('adapter', 'unknown')}")
    print(f"\n  前200字符:")
    print(f"  {result.get('text', '')[:200]}...")

    # 搜索关键词
    print(f"\n🔍 搜索关键词: ['PDF', '文档']")
    matches = client.search_keywords(pdf_file, ['PDF', '文档'])
    print(f"  找到 {matches.get('total_matches', 0)} 个匹配")
    for match in matches.get('matches', [])[:3]:
        print(f"  - 页码 {match['page']}: {match['keyword']}")

    # 提取关键词
    print(f"\n🏷️  提取高频关键词:")
    keywords = client.extract_keywords(pdf_file, top_n=5)
    for kw in keywords.get('keywords', []):
        print(f"  - {kw['keyword']}: {kw['count']} 次")

    # 获取页数
    print(f"\n📊 页数统计:")
    page_count = client.get_page_count(pdf_file)
    print(f"  总页数: {page_count}")


if __name__ == "__main__":
    main()
