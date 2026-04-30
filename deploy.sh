#!/bin/bash

set -e

INSTALL_DIR="/opt/pdf-module"
REPO_OWNER="smile9493"
REPO_NAME="rsut_pdf_mcp"
API_URL="https://api.github.com/repos/${REPO_OWNER}/${REPO_NAME}/releases/latest"

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m'

print_banner() {
    echo -e "${BLUE}"
    echo "██████╗  ██████╗ ██╗     ██╗     ██╗███╗   ██╗ ██████╗ "
    echo "██╔══██╗██╔═══██╗██║     ██║     ██║████╗  ██║██╔════╝ "
    echo "██████╔╝██║   ██║██║     ██║     ██║██╔██╗ ██║██║  ███╗"
    echo "██╔═══╝ ██║   ██║██║     ██║     ██║██║╚██╗██║██║   ██║"
    echo "██║     ╚██████╔╝███████╗███████╗██║██║ ╚████║╚██████╔╝"
    echo "╚═╝      ╚═════╝ ╚══════╝╚══════╝╚═╝╚═╝  ╚═══╝ ╚═════╝ "
    echo -e "${NC}"
    echo -e "${GREEN}PDF Module MCP - 自动部署脚本${NC}"
    echo -e "${CYAN}版本: latest (预编译二进制)${NC}"
    echo ""
}

check_root() {
    if [[ $EUID -ne 0 ]]; then
        echo -e "${RED}错误：此脚本需要 root 权限${NC}"
        echo "请使用: sudo $0"
        exit 1
    fi
}

detect_architecture() {
    ARCH=$(uname -m)
    OS=$(uname -s)
    
    case $OS in
        Linux)
            case $ARCH in
                x86_64)
                    BINARY_NAME="pdf-mcp-linux-x64.tar.gz"
                    ;;
                aarch64|arm64)
                    BINARY_NAME="pdf-mcp-linux-arm64.tar.gz"
                    ;;
                *)
                    echo -e "${RED}不支持的架构: $ARCH${NC}"
                    exit 1
                    ;;
            esac
            ;;
        Darwin)
            case $ARCH in
                x86_64)
                    BINARY_NAME="pdf-mcp-macos-x64.tar.gz"
                    ;;
                arm64)
                    BINARY_NAME="pdf-mcp-macos-arm64.tar.gz"
                    ;;
                *)
                    echo -e "${RED}不支持的架构: $ARCH${NC}"
                    exit 1
                    ;;
            esac
            ;;
        *)
            echo -e "${RED}不支持的操作系统: $OS${NC}"
            exit 1
            ;;
    esac
    
    echo -e "${GREEN}检测到系统: $OS $ARCH${NC}"
    echo -e "${GREEN}二进制包: $BINARY_NAME${NC}"
}

install_curl() {
    if ! command -v curl &> /dev/null; then
        echo -e "${YELLOW}安装 curl...${NC}"
        if command -v apt-get &> /dev/null; then
            apt-get update && apt-get install -y curl
        elif command -v yum &> /dev/null; then
            yum install -y curl
        elif command -v dnf &> /dev/null; then
            dnf install -y curl
        fi
    fi
}

get_latest_version() {
    echo -e "${YELLOW}[1/5] 获取最新版本...${NC}"
    
    VERSION=$(curl -s $API_URL | grep '"tag_name":' | sed -E 's/.*"([^"]+)".*/\1/')
    
    if [[ -z "$VERSION" ]]; then
        echo -e "${RED}无法获取最新版本，使用默认版本 v0.1.1${NC}"
        VERSION="v0.1.1"
    fi
    
    echo -e "${GREEN}最新版本: $VERSION${NC}"
}

download_binaries() {
    echo -e "${YELLOW}[2/5] 下载预编译二进制...${NC}"
    
    mkdir -p "$INSTALL_DIR"
    cd "$INSTALL_DIR"
    
    DOWNLOAD_URL="https://github.com/${REPO_OWNER}/${REPO_NAME}/releases/download/${VERSION}/${BINARY_NAME}"
    echo -e "${CYAN}  下载: $BINARY_NAME${NC}"
    curl -fsSL -o "pdf-mcp.tar.gz" "$DOWNLOAD_URL"
    
    echo -e "${CYAN}  下载: web-dist.tar.gz${NC}"
    curl -fsSL -o "web-dist.tar.gz" "https://github.com/${REPO_OWNER}/${REPO_NAME}/releases/download/${VERSION}/web-dist.tar.gz"
    
    echo -e "${GREEN}✓ 二进制下载完成${NC}"
}

extract_binaries() {
    echo -e "${YELLOW}[3/5] 解压文件...${NC}"
    
    cd "$INSTALL_DIR"
    
    if [[ -f "pdf-mcp.tar.gz" ]]; then
        tar -xzf pdf-mcp.tar.gz
        rm pdf-mcp.tar.gz
        echo -e "${GREEN}✓ pdf-mcp 解压完成${NC}"
    fi
    
    if [[ -f "web-dist.tar.gz" ]]; then
        mkdir -p web
        tar -xzf web-dist.tar.gz -C web
        rm web-dist.tar.gz
        echo -e "${GREEN}✓ Web 前端解压完成${NC}"
    fi
    
    chmod +x "$INSTALL_DIR/pdf-mcp" 2>/dev/null || true
    chmod +x "$INSTALL_DIR/pdf-mcp-cli" 2>/dev/null || true
    
    echo -e "${GREEN}✓ 文件解压完成${NC}"
}

setup_directories() {
    echo -e "${YELLOW}[4/5] 创建目录结构...${NC}"
    
    mkdir -p "$INSTALL_DIR/logs"
    mkdir -p "$INSTALL_DIR/wiki/raw"
    mkdir -p "$INSTALL_DIR/wiki/wiki"
    mkdir -p "$INSTALL_DIR/wiki/scheme"
    mkdir -p "$INSTALL_DIR/data"
    
    echo -e "${GREEN}✓ 目录结构创建完成${NC}"
}

setup_config() {
    echo -e "${YELLOW}[5/5] 配置环境...${NC}"
    
    ENV_FILE="$INSTALL_DIR/.env.local"
    
    if [[ ! -f "$ENV_FILE" ]]; then
        cat > "$ENV_FILE" << 'EOF'
# PDF Module MCP 环境变量配置

# VLM (Visual Language Model) 配置 - GLM 智谱 AI
VLM_API_KEY=
VLM_MODEL=glm-4v-flash
VLM_ENDPOINT=https://open.bigmodel.cn/api/paas/v4/chat/completions

# Dashboard 配置
DASHBOARD_PORT=8000
DASHBOARD_WEB_DIR=/opt/pdf-module/web/dist

# 存储配置
STORAGE_TYPE=local
STORAGE_LOCAL_DIR=/opt/pdf-module/data

# 日志配置
RUST_LOG=info
EOF
        echo -e "${GREEN}✓ 配置文件已创建: $ENV_FILE${NC}"
        echo -e "${YELLOW}请编辑配置文件设置 VLM_API_KEY${NC}"
    else
        echo -e "${GREEN}✓ 配置文件已存在: $ENV_FILE${NC}"
    fi
    
    if [[ ! -f "$INSTALL_DIR/pdf-mcp-cli" ]]; then
        CLI_BIN="$INSTALL_DIR/pdf-mcp-cli"
        cat > "$CLI_BIN" << EOF
#!/bin/bash
exec "$INSTALL_DIR/pdf-mcp" "\$@"
EOF
        chmod +x "$CLI_BIN"
        echo -e "${GREEN}✓ CLI 快捷方式已创建${NC}"
    else
        echo -e "${GREEN}✓ CLI 工具已就绪${NC}"
    fi
}

create_systemd_service() {
    echo -e "${YELLOW}创建 systemd 服务...${NC}"
    
    if command -v systemctl &> /dev/null; then
        SERVICE_FILE="/etc/systemd/system/pdf-mcp.service"
        cat > "$SERVICE_FILE" << EOF
[Unit]
Description=PDF Module MCP Service
After=network.target

[Service]
Type=simple
User=root
WorkingDirectory=$INSTALL_DIR
EnvironmentFile=$INSTALL_DIR/.env.local
ExecStart=$INSTALL_DIR/pdf-mcp dashboard --port \${DASHBOARD_PORT:-8000}
Restart=on-failure
RestartSec=5

[Install]
WantedBy=multi-user.target
EOF
        
        systemctl daemon-reload
        systemctl enable pdf-mcp
        echo -e "${GREEN}✓ systemd 服务已创建并启用${NC}"
    else
        echo -e "${YELLOW}  systemctl 未找到，跳过服务创建${NC}"
    fi
}

print_success() {
    echo ""
    echo -e "${GREEN}========================================${NC}"
    echo -e "${GREEN}部署完成！${NC}"
    echo -e "${GREEN}========================================${NC}"
    echo ""
    echo -e "${CYAN}快速开始:${NC}"
    echo ""
    echo -e "  ${YELLOW}交互式配置 (推荐):${NC}"
    echo "     $INSTALL_DIR/pdf-mcp-cli"
    echo ""
    echo -e "  ${YELLOW}命令行方式:${NC}"
    echo "     $INSTALL_DIR/pdf-mcp-cli config      # 配置 API Key"
    echo "     $INSTALL_DIR/pdf-mcp-cli status      # 查看状态"
    echo "     $INSTALL_DIR/pdf-mcp-cli start --web # 启动服务"
    echo ""
    echo -e "  ${YELLOW}访问 Web 界面:${NC}"
    echo "     http://localhost:8000"
    echo ""
    echo -e "${BLUE}配置文件: $INSTALL_DIR/.env.local${NC}"
    echo -e "${BLUE}安装目录: $INSTALL_DIR${NC}"
    echo ""
    echo -e "${YELLOW}提示: 运行 pdf-mcp-cli 进入交互式配置菜单${NC}"
    echo ""
}

main() {
    print_banner
    check_root
    install_curl
    detect_architecture
    get_latest_version
    download_binaries
    extract_binaries
    setup_directories
    setup_config
    create_systemd_service
    print_success
}

main "$@"
