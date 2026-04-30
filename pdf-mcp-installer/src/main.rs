use clap::{Parser, Subcommand};
use colored::Colorize;
use dialoguer::{Input, theme::ColorfulTheme};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use std::process::Command;

const DEFAULT_INSTALL_DIR: &str = "/opt/pdf-module";
const DEFAULT_VLM_ENDPOINT: &str = "https://open.bigmodel.cn/api/paas/v4/chat/completions";
const DEFAULT_VLM_MODEL: &str = "glm-4v-flash";

#[derive(Parser)]
#[command(name = "pdf-mcp")]
#[command(author = "PDF Module Team")]
#[command(version = "0.1.0")]
#[command(about = "PDF Module MCP 配置管理工具", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// 初始化配置
    Init,
    
    /// 配置 GLM API
    Config {
        #[arg(short, long)]
        key: Option<String>,
        
        #[arg(short, long)]
        model: Option<String>,
        
        #[arg(short, long)]
        endpoint: Option<String>,
    },
    
    /// 查看服务端配置
    Status,
    
    /// 生成客户端配置
    GenerateConfig {
        #[arg(short, long)]
        output: Option<String>,
        
        #[arg(short, long)]
        server_url: Option<String>,
    },
    
    /// 启动服务
    Start {
        #[arg(short, long)]
        web: bool,
    },
    
    /// 停止服务
    Stop,
    
    /// 查看日志
    Logs {
        #[arg(short, long, default_value = "20")]
        lines: u16,
        
        #[arg(short, long)]
        follow: bool,
    },
    
    /// 进入交互式菜单
    Interactive,
}

#[derive(Serialize, Deserialize, Debug)]
struct EnvConfig {
    vlm_api_key: String,
    vlm_model: String,
    vlm_endpoint: String,
    dashboard_port: u16,
    rust_log: String,
}

impl Default for EnvConfig {
    fn default() -> Self {
        Self {
            vlm_api_key: String::new(),
            vlm_model: DEFAULT_VLM_MODEL.to_string(),
            vlm_endpoint: DEFAULT_VLM_ENDPOINT.to_string(),
            dashboard_port: 8000,
            rust_log: "info".to_string(),
        }
    }
}

struct McpManager {
    install_dir: String,
    env_file: String,
}

impl McpManager {
    fn new(install_dir: Option<String>) -> Self {
        let dir = install_dir.unwrap_or_else(|| DEFAULT_INSTALL_DIR.to_string());
        Self {
            install_dir: dir.clone(),
            env_file: format!("{}/.env.local", dir),
        }
    }
    
    fn load_config(&self) -> EnvConfig {
        if !Path::new(&self.env_file).exists() {
            return EnvConfig::default();
        }
        
        let content = fs::read_to_string(&self.env_file).unwrap_or_default();
        let mut config = EnvConfig::default();
        
        for line in content.lines() {
            let line = line.trim();
            if line.starts_with('#') || line.is_empty() {
                continue;
            }
            
            if let Some((key, value)) = line.split_once('=') {
                match key.trim() {
                    "VLM_API_KEY" => config.vlm_api_key = value.trim().to_string(),
                    "VLM_MODEL" => config.vlm_model = value.trim().to_string(),
                    "VLM_ENDPOINT" => config.vlm_endpoint = value.trim().to_string(),
                    "DASHBOARD_PORT" => {
                        config.dashboard_port = value.trim().parse().unwrap_or(8000)
                    }
                    "RUST_LOG" => config.rust_log = value.trim().to_string(),
                    _ => {}
                }
            }
        }
        
        config
    }
    
    fn save_config(&self, config: &EnvConfig) -> std::io::Result<()> {
        let content = format!(
            r#"# PDF Module MCP 环境变量配置

# VLM (Visual Language Model) 配置 - GLM 智谱 AI
VLM_API_KEY={}
VLM_MODEL={}
VLM_ENDPOINT={}

# Dashboard 配置
DASHBOARD_PORT={}

# 日志配置
RUST_LOG={}
"#,
            config.vlm_api_key,
            config.vlm_model,
            config.vlm_endpoint,
            config.dashboard_port,
            config.rust_log,
        );
        
        fs::write(&self.env_file, content)
    }
    
    fn show_banner(&self) {
        println!("\n{}", "██████╗  ██████╗ ██╗     ██╗     ██╗███╗   ██╗ ██████╗ ".cyan().bold());
        println!("{}", "██╔══██╗██╔═══██╗██║     ██║     ██║████╗  ██║██╔════╝ ".cyan().bold());
        println!("{}", "██████╔╝██║   ██║██║     ██║     ██║██╔██╗ ██║██║  ███╗".cyan().bold());
        println!("{}", "██╔═══╝ ██║   ██║██║     ██║     ██║██║╚██╗██║██║   ██║".cyan().bold());
        println!("{}", "██║     ╚██████╔╝███████╗███████╗██║██║ ╚████║╚██████╔╝".cyan().bold());
        println!("{}", "╚═╝      ╚═════╝ ╚══════╝╚══════╝╚═╝╚═╝  ╚═══╝ ╚═════╝ ".cyan().bold());
        println!("\n{}", "PDF Module MCP - 配置管理工具".green().bold());
        println!("{}", format!("安装目录：{}", self.install_dir).blue());
        println!();
    }
    
    fn cmd_init(&self) {
        println!("\n{}", ">>> 初始化配置".cyan().bold());
        
        println!("\n{} 创建目录结构:", "→".blue());
        fs::create_dir_all(&self.install_dir).ok();
        println!("  {} {}", "✓".green(), self.install_dir);
        
        fs::create_dir_all(format!("{}/logs", self.install_dir)).ok();
        println!("  {} {}/logs", "✓".green(), self.install_dir);
        
        println!("\n{} 配置文件:", "→".blue());
        if !Path::new(&self.env_file).exists() {
            let config = EnvConfig::default();
            self.save_config(&config).ok();
            println!("  {} 创建：{}", "✓".green(), self.env_file);
        } else {
            println!("  {} 已存在：{}", "ℹ".blue(), self.env_file);
        }
        
        println!("\n{} 初始化完成！", "✓".green());
        println!("\n{} 快速开始:", "📝".yellow());
        println!("  1. 配置 API Key: pdf-mcp config");
        println!("  2. 查看配置状态: pdf-mcp status");
        println!("  3. 启动服务: pdf-mcp start --web");
    }
    
    fn cmd_config(&self, key: Option<String>, model: Option<String>, endpoint: Option<String>) {
        println!("\n{}", ">>> 配置 GLM API".cyan().bold());
        
        let mut config = self.load_config();
        let mut changed = false;
        
        if let Some(k) = key {
            config.vlm_api_key = k;
            changed = true;
            println!("{} VLM_API_KEY 已设置", "✓".green());
        } else if config.vlm_api_key.is_empty() {
            println!("\n{}", "获取 GLM API Key:".yellow());
            println!("  1. 访问：https://open.bigmodel.cn/");
            println!("  2. 注册/登录账号");
            println!("  3. 进入控制台 -> API Keys");
            println!("  4. 创建新的 API Key\n");
            
            let api_key: String = Input::with_theme(&ColorfulTheme::default())
                .with_prompt("请输入你的 GLM API Key")
                .interact_text()
                .unwrap();
            
            if !api_key.is_empty() {
                config.vlm_api_key = api_key;
                changed = true;
                println!("{} VLM_API_KEY 已设置", "✓".green());
            }
        }
        
        if let Some(m) = model {
            config.vlm_model = m;
            changed = true;
            println!("{} VLM_MODEL 已设置为：{}", "✓".green(), config.vlm_model);
        }
        
        if let Some(e) = endpoint {
            config.vlm_endpoint = e;
            changed = true;
            println!("{} VLM_ENDPOINT 已设置", "✓".green());
        }
        
        if changed {
            self.save_config(&config).ok();
            println!("\n{} 配置已保存", "✓".green());
        }
        
        self.show_config(&config);
    }
    
    fn cmd_config_interactive(&self) {
        use std::io::{self, Write};
        
        println!("\n{}", ">>> GLM API 配置".cyan().bold());
        
        loop {
            let config = self.load_config();
            
            println!("\n{}", "当前配置:".yellow());
            if config.vlm_api_key.is_empty() {
                println!("  {} VLM_API_KEY: {}", "✗".red(), "未配置".red());
            } else {
                let masked = format!("{}****", &config.vlm_api_key[..8.min(config.vlm_api_key.len())]);
                println!("  {} VLM_API_KEY: {}", "✓".green(), masked.cyan());
            }
            println!("  {} VLM_MODEL: {}", "→".blue(), config.vlm_model.cyan());
            println!("  {} VLM_ENDPOINT: {}", "→".blue(), config.vlm_endpoint.cyan());
            
            let left_border = format!("{}", "║ ".blue());
            let right_border = format!(" {}", "║".blue());
            
            println!("\n{}", format!("╔════════════════════════════════════════════════════════════════╗").blue().bold());
            println!("{} {:^56} {}", &left_border, "配置选项", &right_border);
            println!("{}", format!("╠════════════════════════════════════════════════════════════════╣").blue());
            println!("{} {:>2}. 配置 API Key{}", &left_border, "1", &right_border);
            println!("{} {:>2}. 配置模型{}", &left_border, "2", &right_border);
            println!("{} {:>2}. 配置端点{}", &left_border, "3", &right_border);
            println!("{} {:>2}. 返回主菜单{}", &left_border, "0", &right_border);
            println!("{}", format!("╚════════════════════════════════════════════════════════════════╝").blue());
            print!("\n{} 请选择 (0-3): ", format!("{}", "->".blue().bold()));
            io::stdout().flush().unwrap();
            
            let mut input = String::new();
            match io::stdin().read_line(&mut input) {
                Ok(0) | Err(_) => return,
                Ok(_) => {}
            }
            let choice = input.trim().to_string();
            
            match choice.as_str() {
                "1" => {
                    println!("\n{}", "配置 API Key:".cyan().bold());
                    println!("获取 API Key: https://open.bigmodel.cn/ -> 控制台 -> API Keys\n");
                    print!("请输入 GLM API Key: ");
                    io::stdout().flush().unwrap();
                    
                    let mut api_key = String::new();
                    io::stdin().read_line(&mut api_key).unwrap();
                    let api_key = api_key.trim();
                    
                    if !api_key.is_empty() {
                        let mut config = self.load_config();
                        config.vlm_api_key = api_key.to_string();
                        self.save_config(&config).ok();
                        println!("{} API Key 已保存", "✓".green());
                    }
                }
                "2" => {
                    println!("\n{}", "配置模型:".cyan().bold());
                    println!("可选模型:");
                    println!("  1. glm-4v-flash (推荐，免费额度)");
                    println!("  2. glm-4v-plus (付费，高质量)");
                    println!("  3. cogvm-2 (开源模型)\n");
                    print!("请选择模型编号 [1]: ");
                    io::stdout().flush().unwrap();
                    
                    let mut model_choice = String::new();
                    io::stdin().read_line(&mut model_choice).unwrap();
                    
                    let model = match model_choice.trim() {
                        "" | "1" => "glm-4v-flash",
                        "2" => "glm-4v-plus",
                        "3" => "cogvm-2",
                        other => other,
                    };
                    
                    let mut config = self.load_config();
                    config.vlm_model = model.to_string();
                    self.save_config(&config).ok();
                    println!("{} 模型已设置为: {}", "✓".green(), model);
                }
                "3" => {
                    println!("\n{}", "配置 API 端点:".cyan().bold());
                    println!("默认: https://open.bigmodel.cn/api/paas/v4/chat/completions\n");
                    print!("请输入 API 端点: ");
                    io::stdout().flush().unwrap();
                    
                    let mut endpoint = String::new();
                    io::stdin().read_line(&mut endpoint).unwrap();
                    let endpoint = endpoint.trim();
                    
                    let endpoint = if endpoint.is_empty() {
                        DEFAULT_VLM_ENDPOINT
                    } else {
                        endpoint
                    };
                    
                    let mut config = self.load_config();
                    config.vlm_endpoint = endpoint.to_string();
                    self.save_config(&config).ok();
                    println!("{} API 端点已保存", "✓".green());
                }
                "0" => break,
                _ => println!("{} 无效选择", "✗".red()),
            }
        }
    }
    
    fn cmd_status(&self) {
        println!("\n{}", ">>> 服务端配置状态".cyan().bold());
        
        let config = self.load_config();
        self.show_config(&config);
        
        println!("\n{}", "服务状态:".yellow());
        
        let mcp_binary = format!("{}/pdf-mcp", self.install_dir);
        if Path::new(&mcp_binary).exists() {
            println!("  {} MCP 服务：已安装 (按需启动)", "✓".green());
        } else {
            println!("  {} MCP 服务：未安装", "✗".red());
        }
        
        let dashboard_binary = format!("{}/pdf-dashboard", self.install_dir);
        if Path::new(&dashboard_binary).exists() {
            println!("  {} Dashboard 服务：已安装", "✓".green());
            println!("    访问地址：http://localhost:{}", config.dashboard_port);
        } else {
            println!("  {} Dashboard 服务：未安装", "ℹ".blue());
        }
        
        println!("\n{}", "配置检查:".yellow());
        if config.vlm_api_key.is_empty() {
            println!("  {} VLM API Key：未配置 (仅本地模式)", "ℹ".blue());
        } else {
            println!("  {} VLM API Key：已配置", "✓".green());
        }
    }
    
    fn cmd_generate_config(&self, output: Option<String>, server_url: Option<String>) {
        println!("\n{}", ">>> 生成客户端配置".cyan().bold());
        
        let url = server_url.unwrap_or_else(|| "http://localhost:8000".to_string());
        
        let mcp_config = serde_json::json!({
            "mcpServers": {
                "pdf-module": {
                    "command": format!("{}/pdf-mcp", self.install_dir),
                    "env": {
                        "VLM_API_KEY": "YOUR_API_KEY_HERE",
                        "VLM_MODEL": DEFAULT_VLM_MODEL,
                        "VLM_ENDPOINT": DEFAULT_VLM_ENDPOINT
                    }
                }
            }
        });
        
        let config_str = serde_json::to_string_pretty(&mcp_config).unwrap();
        
        if let Some(out_path) = output {
            match fs::write(&out_path, &config_str) {
                Ok(_) => {
                    println!("{} 配置文件已生成：{}", "✓".green(), out_path.cyan());
                    println!("\n{}", config_str);
                }
                Err(e) => {
                    println!("{} 写入失败：{}", "✗".red(), e);
                }
            }
        } else {
            println!("{} 客户端 MCP 配置：\n", "→".blue());
            println!("{}", config_str);
            println!("\n{} 使用说明：", "→".blue());
            println!("  1. 将配置添加到 MCP 客户端配置文件");
            println!("  2. API Key 建议配置在服务端 {}", self.install_dir.cyan());
        }
    }
    
    fn cmd_start(&self, web: bool) {
        println!("\n{}", ">>> 启动服务".cyan().bold());
        
        let config = self.load_config();
        
        if web {
            let dashboard_binary = format!("{}/pdf-dashboard", self.install_dir);
            let web_dist = format!("{}/web/dist", self.install_dir);
            
            // 检查 Dashboard API
            if !Path::new(&dashboard_binary).exists() {
                println!("{} Dashboard API 未安装", "✗".red());
                println!("  提示：请先编译或下载 pdf-dashboard");
                return;
            }
            
            // 检查 Web 前端
            if !Path::new(&web_dist).exists() {
                println!("{} Web 前端未构建", "✗".red());
                println!("  提示：请先构建 Web 前端 (cd web && npm run build)");
                return;
            }
            
            println!("{} 启动 Dashboard API (端口 {})...", "→".blue(), config.dashboard_port);
            
            // 启动 Dashboard API
            Command::new(&dashboard_binary)
                .current_dir(&self.install_dir)
                .env("DASHBOARD_PORT", config.dashboard_port.to_string())
                .env("RUST_LOG", &config.rust_log)
                .spawn()
                .ok();
            
            println!("{} Dashboard API 已启动", "✓".green());
            
            // 启动 Web 前端
            let web_port = 8080;
            println!("\n{} 启动 Web 前端 (端口 {})...", "→".blue(), web_port);
            
            let web_dir = format!("{}/web", self.install_dir);
            Command::new("npx")
                .args(&["serve", "dist", "-p", &web_port.to_string()])
                .current_dir(&web_dir)
                .spawn()
                .ok();
            
            println!("{} Web 前端已启动", "✓".green());
            
            println!("\n{}", "========================================".cyan().bold());
            println!("{} 服务已启动！", "✓".green());
            println!("{}", "========================================".cyan().bold());
            println!("\n{} 访问地址：", "→".blue().bold());
            println!("  Web 界面: http://localhost:{}", web_port);
            println!("  Dashboard API: http://localhost:{}/api/*", config.dashboard_port);
            println!("\n{} 停止服务：", "→".blue());
            println!("  pdf-mcp stop");
            println!("  或按 Ctrl+C 退出此程序（服务将继续运行）");
            println!();
            
            // 保持进程运行
            println!("{} 服务正在后台运行，按 Ctrl+C 退出 CLI", "ℹ".blue());
            loop {
                std::thread::sleep(std::time::Duration::from_secs(1));
            }
        } else {
            println!("{} MCP 服务按需启动，无需手动启动", "ℹ".blue());
            println!("  客户端连接时会自动启动 MCP 服务");
            println!("\n{} 如需启动 Web 界面，请使用:", "→".blue());
            println!("  pdf-mcp start --web");
        }
    }
    
    fn cmd_stop(&self) {
        println!("\n{}", ">>> 停止服务".cyan().bold());
        
        println!("{} 停止 Dashboard API...", "→".blue());
        let stopped_dashboard = Command::new("pkill")
            .args(&["-f", "pdf-dashboard"])
            .status()
            .map(|s| s.success())
            .unwrap_or(false);
        
        if stopped_dashboard {
            println!("{} Dashboard API 已停止", "✓".green());
        } else {
            println!("{} Dashboard API 未运行", "ℹ".blue());
        }
        
        println!("{} 停止 Web 前端...", "→".blue());
        let stopped_web = Command::new("pkill")
            .args(&["-f", "serve dist"])
            .status()
            .map(|s| s.success())
            .unwrap_or(false);
        
        if stopped_web {
            println!("{} Web 前端已停止", "✓".green());
        } else {
            println!("{} Web 前端未运行", "ℹ".blue());
        }
    }
    
    fn cmd_logs(&self, lines: u16, follow: bool) {
        println!("\n{}", ">>> 查看日志".cyan().bold());
        
        let log_file = format!("{}/logs/latest.log", self.install_dir);
        
        if !Path::new(&log_file).exists() {
            println!("{} 日志文件不存在：{}", "✗".red(), log_file);
            return;
        }
        
        if follow {
            println!("{} 实时日志 (按 Ctrl+C 退出)", "→".blue());
            Command::new("tail")
                .args(&["-f", "-n", &lines.to_string(), &log_file])
                .status()
                .ok();
        } else {
            Command::new("tail")
                .args(&["-n", &lines.to_string(), &log_file])
                .status()
                .ok();
        }
    }
    
    fn show_config(&self, config: &EnvConfig) {
        println!("\n{}", "配置信息:".yellow());
        
        if config.vlm_api_key.is_empty() {
            println!("  {} VLM_API_KEY: {}", "✗".red(), "未配置".red());
        } else {
            let masked = format!("{}****", &config.vlm_api_key[..8.min(config.vlm_api_key.len())]);
            println!("  {} VLM_API_KEY: {}", "✓".green(), masked.cyan());
        }
        
        println!("  {} VLM_MODEL: {}", "→".blue(), config.vlm_model.cyan());
        println!("  {} VLM_ENDPOINT: {}", "→".blue(), config.vlm_endpoint.cyan());
        println!("  {} DASHBOARD_PORT: {}", "→".blue(), config.dashboard_port.to_string().cyan());
        println!("  {} RUST_LOG: {}", "→".blue(), config.rust_log.cyan());
    }
}

fn main() {
    let cli = Cli::parse();
    let manager = McpManager::new(None);
    
    if std::env::args().len() == 1 {
        manager.show_banner();
        interactive_menu(&manager);
        return;
    }
    
    manager.show_banner();
    
    match cli.command {
        Commands::Init => manager.cmd_init(),
        Commands::Config { key, model, endpoint } => {
            manager.cmd_config(key, model, endpoint)
        }
        Commands::Status => manager.cmd_status(),
        Commands::GenerateConfig { output, server_url } => {
            manager.cmd_generate_config(output, server_url)
        }
        Commands::Start { web } => manager.cmd_start(web),
        Commands::Stop => manager.cmd_stop(),
        Commands::Logs { lines, follow } => manager.cmd_logs(lines, follow),
        Commands::Interactive => {
            interactive_menu(&manager);
        }
    }
}

fn interactive_menu(manager: &McpManager) {
    use std::io::{self, Write};
    
    loop {
        let left_border = format!("{}", "║ ".blue());
        let right_border = format!(" {}", "║".blue());
        
        println!("\n{}", format!("╔════════════════════════════════════════════════════════════════╗").blue().bold());
        println!("{} {:^56} {}", &left_border, "PDF Module MCP 配置管理", &right_border);
        println!("{}", format!("╠════════════════════════════════════════════════════════════════╣").blue());
        println!("{} {:>2}. 初始化配置{}", &left_border, "1", &right_border);
        println!("{} {:>2}. 配置 GLM API{}", &left_border, "2", &right_border);
        println!("{} {:>2}. 查看服务端配置{}", &left_border, "3", &right_border);
        println!("{} {:>2}. 生成客户端配置{}", &left_border, "4", &right_border);
        println!("{} {:>2}. 启动服务{}", &left_border, "5", &right_border);
        println!("{} {:>2}. 停止服务{}", &left_border, "6", &right_border);
        println!("{} {:>2}. 查看日志{}", &left_border, "7", &right_border);
        println!("{} {:>2}. 退出{}", &left_border, "0", &right_border);
        println!("{}", format!("╚════════════════════════════════════════════════════════════════╝").blue());
        print!("\n{} 请选择 (0-7): ", format!("{}", "->".blue().bold()));
        io::stdout().flush().unwrap();
        
        let mut input = String::new();
        match io::stdin().read_line(&mut input) {
            Ok(0) | Err(_) => {
                println!("\n{} 退出\n", "ℹ".blue());
                break;
            }
            Ok(_) => {}
        }
        let choice = input.trim();
        
        if choice.is_empty() {
            continue;
        }
        
        match choice {
            "1" => {
                println!();
                manager.cmd_init();
            }
            "2" => {
                println!();
                manager.cmd_config_interactive();
            }
            "3" => {
                println!();
                manager.cmd_status();
            }
            "4" => {
                println!();
                manager.cmd_generate_config(None, None);
            }
            "5" => {
                println!();
                println!("{} 启动选项:", "→".blue());
                println!("  1. 启动 Dashboard (Web界面)");
                println!("  0. 返回");
                print!("\n{} 请选择: ", "->".blue());
                io::stdout().flush().unwrap();
                
                let mut start_choice = String::new();
                io::stdin().read_line(&mut start_choice).unwrap();
                
                match start_choice.trim() {
                    "1" => manager.cmd_start(true),
                    _ => continue,
                }
            }
            "6" => {
                println!();
                manager.cmd_stop();
            }
            "7" => {
                println!();
                print!("{} 查看最近多少行？[20]: ", "→".blue());
                io::stdout().flush().unwrap();
                let mut lines_input = String::new();
                io::stdin().read_line(&mut lines_input).unwrap();
                let lines: u16 = lines_input.trim().parse().unwrap_or(20);
                manager.cmd_logs(lines, false);
            }
            "0" => {
                println!("\n{} 退出\n", "✓".green());
                break;
            }
            _ => {
                println!("{} 无效选择", "✗".red());
            }
        }
        
        if choice != "0" && choice != "5" {
            println!("\n{} 按回车键继续...", "→".blue());
            let mut pause = String::new();
            io::stdin().read_line(&mut pause).unwrap();
        }
    }
}
