#!/bin/bash

set -e

INSTALL_DIR="/opt/pdf-module"
REPO_URL="https://github.com/smile9493/rsut_pdf_mcp.git"
CLI_DIR="$INSTALL_DIR/pdf-mcp-installer"

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
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
    echo ""
}

check_root() {
    if [[ $EUID -ne 0 ]]; then
        echo -e "${RED}错误：此脚本需要 root 权限${NC}"
        echo "请使用: sudo $0"
        exit 1
    fi
}

detect_os() {
    if [[ "$OSTYPE" == "linux-gnu"* ]]; then
        OS="linux"
        if command -v apt-get &> /dev/null; then
            PKG_MANAGER="apt"
        elif command -v yum &> /dev/null; then
            PKG_MANAGER="yum"
        elif command -v dnf &> /dev/null; then
            PKG_MANAGER="dnf"
        else
            PKG_MANAGER="unknown"
        fi
    elif [[ "$OSTYPE" == "darwin"* ]]; then
        OS="macos"
        PKG_MANAGER="brew"
    else
        echo -e "${RED}不支持的操作系统: $OSTYPE${NC}"
        exit 1
    fi
    echo -e "${GREEN}检测到操作系统: $OS${NC}"
    echo -e "${GREEN}包管理器: $PKG_MANAGER${NC}"
}

install_dependencies() {
    echo -e "${YELLOW}安装依赖...${NC}"
    
    if [[ "$OS" == "linux" ]]; then
        if [[ "$PKG_MANAGER" == "apt" ]]; then
            apt-get update
            apt-get install -y curl git build-essential pkg-config libssl-dev
        elif [[ "$PKG_MANAGER" == "yum" ]] || [[ "$PKG_MANAGER" == "dnf" ]]; then
            $PKG_MANAGER install -y curl git gcc gcc-c++ make openssl-devel
        fi
    elif [[ "$OS" == "macos" ]]; then
        if ! command -v brew &> /dev/null; then
            echo -e "${YELLOW}安装 Homebrew...${NC}"
            /bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
        fi
        brew install curl git
    fi
    
    echo -e "${GREEN}依赖安装完成${NC}"
}

install_rust() {
    if ! command -v rustc &> /dev/null; then
        echo -e "${YELLOW}安装 Rust...${NC}"
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
        source "$HOME/.cargo/env"
        echo -e "${GREEN}Rust 安装完成${NC}"
    else
        echo -e "${GREEN}Rust 已安装: $(rustc --version)${NC}"
    fi
}

install_nodejs() {
    if ! command -v node &> /dev/null; then
        echo -e "${YELLOW}安装 Node.js...${NC}"
        
        if [[ "$OS" == "linux" ]]; then
            curl -fsSL https://deb.nodesource.com/setup_20.x | bash -
            apt-get install -y nodejs
        elif [[ "$OS" == "macos" ]]; then
            brew install node
        fi
        
        echo -e "${GREEN}Node.js 安装完成: $(node --version)${NC}"
    else
        echo -e "${GREEN}Node.js 已安装: $(node --version)${NC}"
    fi
}

clone_repository() {
    echo -e "${YELLOW}克隆项目...${NC}"
    
    if [[ -d "$INSTALL_DIR" ]]; then
        echo -e "${YELLOW}目录已存在: $INSTALL_DIR${NC}"
        read -p "是否删除并重新克隆？(y/n): " -n 1 -r
        echo
        if [[ $REPLY =~ ^[Yy]$ ]]; then
            rm -rf "$INSTALL_DIR"
        else
            echo -e "${YELLOW}使用现有目录${NC}"
            return
        fi
    fi
    
    mkdir -p "$INSTALL_DIR"
    git clone "$REPO_URL" "$INSTALL_DIR"
    echo -e "${GREEN}项目克隆完成${NC}"
}

build_cli() {
    echo -e "${YELLOW}构建 CLI 工具...${NC}"
    
    cd "$CLI_DIR"
    cargo build --release
    
    echo -e "${GREEN}CLI 工具构建完成${NC}"
    echo -e "${GREEN}位置: $CLI_DIR/target/release/pdf-mcp${NC}"
}

build_dashboard() {
    echo -e "${YELLOW}构建 Dashboard 服务...${NC}"
    
    cd "$INSTALL_DIR/pdf-module-rs"
    cargo build --release --bin pdf-dashboard
    
    cp target/release/pdf-dashboard "$INSTALL_DIR/"
    chmod +x "$INSTALL_DIR/pdf-dashboard"
    
    echo -e "${GREEN}Dashboard 服务构建完成${NC}"
}

build_web() {
    echo -e "${YELLOW}构建 Web 前端...${NC}"
    
    cd "$INSTALL_DIR/web"
    npm install
    npm run build
    
    echo -e "${GREEN}Web 前端构建完成${NC}"
}

copy_binaries() {
    echo -e "${YELLOW}复制二进制文件...${NC}"
    
    if [[ -f "$INSTALL_DIR/pdf-module-rs/target/release/pdf-mcp" ]]; then
        cp "$INSTALL_DIR/pdf-module-rs/target/release/pdf-mcp" "$INSTALL_DIR/"
        chmod +x "$INSTALL_DIR/pdf-mcp"
        echo -e "${GREEN}pdf-mcp 已复制${NC}"
    fi
    
    echo -e "${GREEN}二进制文件复制完成${NC}"
}

create_directories() {
    echo -e "${YELLOW}创建目录结构...${NC}"
    
    mkdir -p "$INSTALL_DIR/logs"
    mkdir -p "$INSTALL_DIR/wiki/raw"
    mkdir -p "$INSTALL_DIR/wiki/wiki"
    mkdir -p "$INSTALL_DIR/wiki/scheme"
    
    echo -e "${GREEN}目录结构创建完成${NC}"
}

create_env_file() {
    echo -e "${YELLOW}创建配置文件...${NC}"
    
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

# 日志配置
RUST_LOG=info
EOF
        echo -e "${GREEN}配置文件已创建: $ENV_FILE${NC}"
        echo -e "${YELLOW}请编辑配置文件设置 VLM_API_KEY${NC}"
    else
        echo -e "${GREEN}配置文件已存在: $ENV_FILE${NC}"
    fi
}

print_success() {
    echo ""
    echo -e "${GREEN}========================================${NC}"
    echo -e "${GREEN}部署完成！${NC}"
    echo -e "${GREEN}========================================${NC}"
    echo ""
    echo -e "${BLUE}快速开始:${NC}"
    echo ""
    echo "1. 配置 API Key:"
    echo "   cd $CLI_DIR"
    echo "   ./target/release/pdf-mcp config"
    echo ""
    echo "2. 查看状态:"
    echo "   ./target/release/pdf-mcp status"
    echo ""
    echo "3. 启动服务:"
    echo "   ./target/release/pdf-mcp start --web"
    echo ""
    echo "4. 访问 Web 界面:"
    echo "   http://localhost:8080"
    echo ""
    echo -e "${YELLOW}提示: API Key 请配置在服务端 $INSTALL_DIR/.env.local${NC}"
    echo ""
}

main() {
    print_banner
    check_root
    detect_os
    install_dependencies
    install_rust
    install_nodejs
    clone_repository
    build_cli
    build_dashboard
    build_web
    copy_binaries
    create_directories
    create_env_file
    print_success
}

main "$@"
