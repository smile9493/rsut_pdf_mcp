# PDF MCP Module

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/Rust-1.83%2B-orange.svg)](https://www.rust-lang.org/)
[![Vue](https://img.shields.io/badge/Vue-3.5%2B-green.svg)](https://vuejs.org/)
[![CI](https://github.com/smile9493/rsut_pdf_mcp/actions/workflows/build.yml/badge.svg)](https://github.com/smile9493/rsut_pdf_mcp/actions)
[![Release](https://img.shields.io/github/v/release/smile9493/rsut_pdf_mcp)](https://github.com/smile9493/rsut_pdf_mcp/releases)

** PDF 提取 MCP 管道** — 零拷贝mmap、智能质量探测、Wiki自动化、双模态工具集。

基于**唯物辩证法与截拳道**工程哲学，将项目从臃肿的通用服务端重塑为极致精简、物理确定的 **AI Agent 专用感知器官**。

***

## 🎯 核心特性

### ✨ 宗师级重构亮点

- 🚀 **物理顺应**: mmap零拷贝 + Arena分配器，顺应OS Page Cache
- 🧠 **智能感知**: 自动质量探测 + VLM热切换，扫描件自动识别
- 📚 **Wiki自动化**: 三级存储结构 + 自动索引生成
- 🔧 **双模态工具**: 服务端构建 + 本地投影两种模式

### 🥋 截拳道工程哲学

| 哲学原则     | 工程实现                 | 性能收益           |
| -------- | -------------------- | -------------- |
| **寸劲发力** | Arena分配器 $O(1)$ 批量销毁 | 微秒级内存回收        |
| **截击之道** | FFI防波堤 + 质量探测拦截      | 防止C++ Segfault |
| **物理顺应** | mmap零拷贝顺应Page Cache  | 零内存拷贝          |
| **虚实结合** | Pdfium本地 + VLM云端     | 确定性 + 智能增强     |

***

## 架构

```
┌─────────────────────────────────────────────────────────────┐
│                  AI Agent (Cursor/Claude Desktop)           │
──────────────────────────┬──────────────────────────────────┘
                         │ JSON-RPC over stdio
                         ▼
┌─────────────────────────────────────────────────────────────┐
│                     pdf-mcp (二进制客户端)                   │
│  Windows: pdf-mcp.exe  │  Linux/macOS: pdf-mcp             │
├─────────────────────────────────────────────────────────────┤
│  MCP Tools (6个核心工具):                                    │
│  • extract_text              - 提取纯文本                    │
│  • extract_structured        - 提取结构化数据                │
│  • get_page_count            - 获取页数                      │
│  • search_keywords           - 关键词搜索                    │
│  • extrude_to_server_wiki    - 服务端Wiki构建 ⭐ NEW         │
│  • extrude_to_agent_payload  - 本地Wiki投影 ⭐ NEW           │
└──────────────────────────┬──────────────────────────────────┘
                         │
    ─────────────────────┴──────────────────┐
    ▼                                        ▼
┌───────────────────┐              ┌───────────────────┐
│   MmapPdfLoader   │              │   QualityProbe    │
│   零拷贝加载       │              │   智能质量探测     │
│   物理顺应         │              │   扫描件识别       │
└─────────┬─────────┘              └─────────┬─────────┘
          │                                  │
          ▼                                  ▼
┌───────────────────┐              ┌───────────────────┐
│   PdfiumEngine    │              │   VlmGateway      │
│   本地确定性提取    │              │   条件升级         │
│   FFI 防波堤       │              │   GPT-4o/Claude   │
└───────────────────┘              └───────────────────┘

┌─────────────────────────────────────────────────────────────┐
│                   Wiki自动化系统 ⭐ NEW                      │
├─────────────────────────────────────────────────────────────┤
│  wiki/                                                       │
│  ├── raw/           # 物理提取产物 (.raw.md + YAML元数据)    │
│  ├── wiki/          # 精炼后的实体页面                       │
│  ├── scheme/        # 强类型约束文件                         │
│  └── MAP.md         # 全局语义地图 (自动生成)                │
└─────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────┐
│                   Web Dashboard (Vue 3)                     │
├─────────────────────────────────────────────────────────────┤
│  /              │ 首页 (状态、工具概览、快速操作)             │
│  /extract       │ 文本提取                                    │
│  /search        │ 关键词搜索                                  │
│  /batch         │ 批量处理                                    │
│  /mcp-tools     │ 工具测试                                    │
│  /settings      │ MCP 配置 (服务器 + VLM API Key)            │
└─────────────────────────────────────────────────────────────┘
```

***

## 🚀 快速开始

### 方式一：Docker Compose（推荐）

```yaml
version: "3.8"

services:
  pdf-mcp:
    image: smile9493/pdf-mcp:0.1.1
    container_name: pdf-mcp
    restart: unless-stopped
    ports:
      - "8000:8000"   # Dashboard Web UI
      - "8001:8001"   # MCP SSE (可选)
    volumes:
      - ./data:/app/data
      - ./wiki:/app/wiki
      - /path/to/pdfs:/pdfs:ro
    environment:
      - RUST_LOG=info
      - STORAGE_TYPE=local
      - STORAGE_LOCAL_DIR=/app/data
      - DASHBOARD_WEB_DIR=/app/web/dist
      - DASHBOARD_PORT=8000
      - VLM_API_KEY=${VLM_API_KEY:-}
      - VLM_MODEL=${VLM_MODEL:-gpt-4o}
    stdin_open: true
    tty: true
```

### 方式二：二进制文件

从 [Releases](https://github.com/smile9493/rsut_pdf_mcp/releases) 下载对应平台版本。

### 方式三：从源码构建

```bash
git clone https://github.com/smile9493/rsut_pdf_mcp.git
cd rsut_pdf_mcp/pdf-module-rs

cargo build --release --bin pdf-mcp
```

***

## 📚 MCP 工具详解

### 基础工具

| 工具                   | 说明      | 参数                                         |
| -------------------- | ------- | ------------------------------------------ |
| `extract_text`       | 提取纯文本   | `file_path`                                |
| `extract_structured` | 提取结构化数据 | `file_path`                                |
| `get_page_count`     | 获取页数    | `file_path`                                |
| `search_keywords`    | 关键词搜索   | `file_path`, `keywords`, `case_sensitive?` |

### ⭐ 新增：Wiki自动化工具

#### 1. `extrude_to_server_wiki` - 服务端构建模式

**功能**: 在服务器端闭环完成提取、落盘与索引更新

**参数**:

- `file_path`: PDF文件绝对路径
- `wiki_base_path`: Wiki存储基础目录（默认: `./wiki`）

**返回示例**:

```json
{
  "status": "success",
  "raw_path": "/opt/wiki/raw/a1b2c3d4.raw.md",
  "map_path": "/opt/wiki/MAP.md",
  "page_count": 42,
  "message": "PDF extracted and saved to wiki"
}
```

**使用场景**: 大工程文档库、多设备共享大脑

#### 2. `extrude_to_agent_payload` - 本地投影模式

**功能**: 服务器仅进行计算，将Markdown报文发还给Agent

**参数**:

- `file_path`: PDF文件绝对路径

**返回示例**:

```markdown
---
source_file: /path/to/document.pdf
file_hash: a1b2c3d4
extraction_time: 2026-04-30T12:00:00Z
page_count: 42
quality_score: 0.85
extraction_method: pdfium
---

# PDF Extraction Complete

Your PDF has been successfully extracted...

[完整提取内容]
```

**使用场景**: iOS/Windows本地Wiki构建

***

## 🧠 智能质量探测

系统会自动分析PDF质量并选择最佳提取方法：

```
PDF → 质量探测 → 文本密度分析
  ├─ Digital (text_density > 0.3) → Pdfium 本地提取
  ├─ Scanned (无字体，有图像) → VLM 多模态增强
  └─ LowQuality (text_density < 0.1) → 混合模式
```

**质量指标**:

- 文本密度计算
- 字体检测
- 图像检测
- 置信度评分

***

## 📂 Wiki自动化系统

### 三级存储结构

```
wiki/
├── raw/                    # 物理提取产物
│   └── [hash].raw.md      # 带YAML元数据的原始提取
├── wiki/                   # 精炼后的实体页面
├── scheme/                 # 强类型约束文件
└── MAP.md                  # 全局语义地图 (自动生成)
```

### YAML元数据示例

```yaml
---
source_file: /path/to/document.pdf
file_hash: a1b2c3d4
extraction_time: 2026-04-30T12:00:00Z
page_count: 42
quality_score: 0.85
extraction_method: pdfium
---

# Extracted Content

[PDF文本内容...]
```

### MAP.md自动索引

```markdown
# PDF Knowledge Map

## Raw Extractions

- [a1b2c3d4.raw.md](raw/a1b2c3d4.raw.md)
- [e5f6g7h8.raw.md](raw/e5f6g7h8.raw.md)

**Total documents**: 2
```

***

## ⚙️ Agent 集成配置

### Cursor 配置

编辑 `~/.cursor/mcp.json`:

```json
{
  "mcpServers": {
    "pdf-module": {
      "command": "/usr/local/bin/pdf-mcp",
      "env": {
        "VLM_API_KEY": "sk-xxx",
        "VLM_MODEL": "gpt-4o"
      }
    }
  }
}
```

### Claude Desktop 配置

编辑 `~/Library/Application Support/Claude/claude_desktop_config.json`:

```json
{
  "mcpServers": {
    "pdf-module": {
      "command": "/usr/local/bin/pdf-mcp",
      "env": {
        "PDFIUM_LIB_PATH": "/opt/pdf-mcp/lib/",
        "VLM_API_KEY": "sk-xxx"
      }
    }
  }
}
```

***

## 🔧 环境变量

| 变量                    | 说明        | 默认值                                          |
| --------------------- | --------- | -------------------------------------------- |
| `VLM_API_KEY`         | VLM API密钥 | -                                            |
| `VLM_ENDPOINT`        | VLM端点URL  | `https://api.openai.com/v1/chat/completions` |
| `VLM_MODEL`           | 模型名称      | `gpt-4o`                                     |
| `VLM_TIMEOUT_SECS`    | 请求超时      | `30`                                         |
| `VLM_MAX_CONCURRENCY` | 最大并发      | `5`                                          |
| `PDFIUM_LIB_PATH`     | Pdfium库路径 | -                                            |
| `DASHBOARD_PORT`      | Dashboard端口 | `8000`                                       |
| `DASHBOARD_WEB_DIR`   | 前端静态文件目录 | `./web/dist`                                  |
| `STORAGE_TYPE`        | 存储类型      | `local`                                       |
| `STORAGE_LOCAL_DIR`   | 本地存储目录   | `./data`                                      |
| `CACHE_ENABLED`       | 缓存开关      | `true`                                        |
| `CACHE_MAX_SIZE`      | 缓存上限      | `1000`                                        |

***

## 📊 性能指标

### 启动时间

- **< 1ms**: 无HTTP服务开销，纯stdio通信

### 内存占用

- **零拷贝**: mmap直接映射，无堆拷贝
- **Arena分配**: 请求级批量销毁

### 提取速度

- **Pdfium本地**: 确定性高性能
- **VLM云端**: 智能增强扫描件

***

## 🧪 测试覆盖

```bash
running 24 tests
test result: ok. 24 passed; 0 failed; 0 ignored
```

**测试覆盖**:

- ✅ mmap加载器
- ✅ 质量探测
- ✅ Wiki存储
- ✅ VLM管道
- ✅ 错误处理

***

## 📁 项目结构

```
pdf-module-rs/
├── crates/
│   ├── pdf-common/           # error + dto + config
│   ├── pdf-core/             # 核心引擎
│   │   ├── mmap_loader.rs    # 零拷贝加载器 ⭐
│   │   ├── quality_probe.rs  # 质量探测器 ⭐
│   │   ├── wiki.rs           # Wiki自动化 ⭐
│   │   └── engine/           # Pdfium引擎
│   ├── pdf-mcp/              # MCP stdio 入口
│   ├── pdf-dashboard/        # Dashboard Web 服务 ⭐
│   └── vlm-visual-gateway/   # VLM条件升级

web/
├── src/
│   ├── views/                # Vue组件
│   ├── composables/          # 组合式函数
│   └── locales/              # 国际化

Dockerfile                    # 统一构建镜像 (前端+后端)
docker-compose.yml            # 编排配置
```

***

## 📝 版本历史

### v0.1.1 - Dashboard集成 (2026-04-30)

**新增功能**:

- ✨ Dashboard Web 服务 (pdf-dashboard)
- ✨ 统一 Docker 镜像 (前端+后端+MCP)
- ✨ 深色模式 (Slate 色系)

**界面优化**:

- 🎨 统一深色模式 Slate 色系
- 🎨 12列栅格布局 (5:7 输入输出比)
- 🎨 精简顶部导航栏
- 🎨 左侧栏紧凑布局

**Bug修复**:

- 🐛 仪表盘工具数量显示不准确
- 🐛 左侧栏底部空白区域
- 🐛 MCP工具页统计与仪表盘重合

### v0.3.0 - 宗师级重构 (2026-04-30)

**新增功能**:

- ✨ mmap零拷贝加载器
- ✨ 智能质量探测系统
- ✨ Wiki自动化系统
- ✨ 双模态MCP工具

**性能优化**:

- 🚀 物理顺应：mmap + Arena
- 🚀 FFI防波堤：catch\_unwind
- 🚀 启动时间：< 1ms

**架构改进**:

- 🏗️ 三级存储结构
- 🏗️ 自动索引生成
- 🏗️ 双模态工具集

***

## 🤝 贡献指南

欢迎贡献！请查看 [CONTRIBUTING.md](CONTRIBUTING.md)

***

## 📄 License

[MIT](LICENSE)

***

## 🙏 致谢

本项目受以下工程哲学启发：

- **唯物辩证法**: 矛盾驱动演化，量变到质变
- **截拳道**: 以无法为有法，以无限为有限
- **机械同情**: 软件顺应硬件，共振而非优化

***

**Made with ❤️ by the rsut\_pdf\_mcp team**
