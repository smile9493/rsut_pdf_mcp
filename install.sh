#!/bin/bash

set -e

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m'

INSTALL_DIR="/opt/pdf-module"
REPO_URL="https://github.com/smile9493/rsut_pdf_mcp"
CLI_DIR="$INSTALL_DIR/pdf-mcp-installer"

print_banner() {
    echo -e "${CYAN}"
    cat << 'EOF'
██████╗  ██████╗ ██╗     ██╗     ██╗███╗   ██╗ ██████╗ 
██╔══██╗██╔═══██╗██║     ██║     ██║████╗  ██║██╔════╝ 
██████╔╝██║   ██║██║     ██║     ██║██╔██╗ ██║██║  ███╗
██╔═══╝ ██║   ██║██║     ██║     ██║██║╚██╗██║██║   ██║
██║     ╚██████╔╝███████╗███████╗██║██║ ╚████║╚██████╔╝
╚═╝      ╚═════╝ ╚══════╝╚══════╝╚═╝╚═╝  ╚═══╝ ╚═════╝ 
EOF
    echo -e "${NC}"
    echo -e "${GREEN}PDF Module MCP - 一键安装脚本${NC}"
    echo -e "${BLUE}版本: latest${NC}"
    echo ""
}

check_root() {
    if [[ $EUID -ne 0 ]]; then
        echo -e "${RED}错误：此脚本需要 root 权限${NC}"
        echo "请使用: sudo bash $0"
        exit 1
    fi
}

detect_os() {
    if [[ -f /etc/os-release ]]; then
        . /etc/os-release
        OS=$ID
        VER=$VERSION_ID
    elif type lsb_release >/dev/null 2>&1; then
        OS=$(lsb_release -si)
        VER=$(lsb_release -sr)
    else
        OS=$(uname -s)
        VER=$(uname -r)
    fi
    
    echo -e "${GREEN}检测到操作系统: $OS $VER${NC}"
    
    case $OS in
        ubuntu|debian)
            PKG_MANAGER="apt"
            PKG_UPDATE="apt-get update"
            PKG_INSTALL="apt-get install -y"
            ;;
        centos|rhel|fedora)
            PKG_MANAGER="yum"
            PKG_UPDATE="yum makecache"
            PKG_INSTALL="yum install -y"
            ;;
        *)
            echo -e "${YELLOW}未知的包管理器，将尝试使用 apt${NC}"
            PKG_MANAGER="apt"
            PKG_UPDATE="apt-get update"
            PKG_INSTALL="apt-get install -y"
            ;;
    esac
}

install_dependencies() {
    echo -e "${YELLOW}[1/6] 安装依赖...${NC}"
    
    $PKG_UPDATE
    $PKG_INSTALL curl wget git build-essential pkg-config libssl-dev
    
    echo -e "${GREEN}✓ 依赖安装完成${NC}"
}

install_rust() {
    echo -e "${YELLOW}[2/6] 检查 Rust...${NC}"
    
    if ! command -v rustc &> /dev/null; then
        echo -e "${YELLOW}安装 Rust...${NC}"
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
        source "$HOME/.cargo/env"
        echo -e "${GREEN}✓ Rust 安装完成${NC}"
    else
        echo -e "${GREEN}✓ Rust 已安装: $(rustc --version)${NC}"
    fi
}

install_nodejs() {
    echo -e "${YELLOW}[3/6] 检查 Node.js...${NC}"
    
    if ! command -v node &> /dev/null; then
        echo -e "${YELLOW}安装 Node.js 20.x...${NC}"
        curl -fsSL https://deb.nodesource.com/setup_20.x | bash -
        $PKG_INSTALL nodejs
        echo -e "${GREEN}✓ Node.js 安装完成: $(node --version)${NC}"
    else
        echo -e "${GREEN}✓ Node.js 已安装: $(node --version)${NC}"
    fi
}

clone_repository() {
    echo -e "${YELLOW}[4/6] 克隆项目...${NC}"
    
    if [[ -d "$INSTALL_DIR" ]]; then
        echo -e "${YELLOW}目录已存在，更新代码...${NC}"
        cd "$INSTALL_DIR"
        git pull
    else
        git clone "$REPO_URL" "$INSTALL_DIR"
    fi
    
    echo -e "${GREEN}✓ 项目克隆完成${NC}"
}

build_project() {
    echo -e "${YELLOW}[5/6] 构建项目...${NC}"
    
    cd "$INSTALL_DIR"
    
    # 构建 CLI 工具
    echo -e "${CYAN}  构建 CLI 工具...${NC}"
    cd "$CLI_DIR"
    cargo build --release
    
    # 构建 Dashboard
    echo -e "${CYAN}  构建 Dashboard 服务...${NC}"
    cd "$INSTALL_DIR/pdf-module-rs"
    cargo build --release --bin pdf-mcp --bin pdf-dashboard
    
    # 构建 Web 前端
    echo -e "${CYAN}  构建 Web 前端...${NC}"
    cd "$INSTALL_DIR/web"
    npm install
    npm run build
    
    # 复制二进制文件
    cp "$INSTALL_DIR/pdf-module-rs/target/release/pdf-mcp" "$INSTALL_DIR/"
    cp "$INSTALL_DIR/pdf-module-rs/target/release/pdf-dashboard" "$INSTALL_DIR/"
    chmod +x "$INSTALL_DIR/pdf-mcp" "$INSTALL_DIR/pdf-dashboard"
    
    echo -e "${GREEN}✓ 项目构建完成${NC}"
}

setup_config() {
    echo -e "${YELLOW}[6/6] 配置环境...${NC}"
    
    # 创建目录
    mkdir -p "$INSTALL_DIR/logs"
    mkdir -p "$INSTALL_DIR/wiki/raw"
    mkdir -p "$INSTALL_DIR/wiki/wiki"
    mkdir -p "$INSTALL_DIR/wiki/scheme"
    
    # 创建配置文件
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
        echo -e "${GREEN}✓ 配置文件已创建${NC}"
    else
        echo -e "${GREEN}✓ 配置文件已存在${NC}"
    fi
    
    # 创建 CLI 快捷方式
    CLI_BIN="$INSTALL_DIR/pdf-mcp-cli"
    cat > "$CLI_BIN" << EOF
#!/bin/bash
cd $CLI_DIR
./target/release/pdf-mcp "\$@"
EOF
    chmod +x "$CLI_BIN"
    
    echo -e "${GREEN}✓ 环境配置完成${NC}"
}

print_success() {
    echo ""
    echo -e "${GREEN}========================================${NC}"
    echo -e "${GREEN}安装完成！${NC}"
    echo -e "${GREEN}========================================${NC}"
    echo ""
    echo -e "${CYAN}快速开始:${NC}"
    echo ""
    echo -e "  ${YELLOW}1. 配置 API Key:${NC}"
    echo "     $CLI_BIN config"
    echo ""
    echo -e "  ${YELLOW}2. 查看状态:${NC}"
    echo "     $CLI_BIN status"
    echo ""
    echo -e "  ${YELLOW}3. 启动服务:${NC}"
    echo "     $CLI_BIN start --web"
    echo ""
    echo -e "  ${YELLOW}4. 访问 Web 界面:${NC}"
    echo "     http://localhost:8080"
    echo ""
    echo -e "${BLUE}配置文件: $INSTALL_DIR/.env.local${NC}"
    echo -e "${BLUE}安装目录: $INSTALL_DIR${NC}"
    echo ""
    echo -e "${YELLOW}提示: 请先配置 VLM_API_KEY${NC}"
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
    build_project
    setup_config
    print_success
}

main "$@"
