use clap::{Parser, Subcommand};
use colored::Colorize;
use dialoguer::{Input, theme::ColorfulTheme};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use std::process::{Command, Stdio};

const DEFAULT_INSTALL_DIR: &str = "/opt/pdf-module";
const DEFAULT_VLM_ENDPOINT: &str = "https://open.bigmodel.cn/api/paas/v4/chat/completions";
const DEFAULT_VLM_MODEL: &str = "glm-4v-flash";

#[derive(Parser)]
#[command(name = "pdf-mcp-cli")]
#[command(author = "PDF Module Team")]
#[command(version = "0.1.0")]
#[command(about = "PDF Module MCP 配置管理工具", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    Init,
    
    Config {
        #[arg(short, long)]
        key: Option<String>,
        
        #[arg(short, long)]
        model: Option<String>,
        
        #[arg(short, long)]
        endpoint: Option<String>,
    },
    
    Status,
    
    GenerateConfig {
        #[arg(short, long)]
        output: Option<String>,
    },
    
    Start {
        #[arg(short, long)]
        web: bool,
    },
    
    Stop,
    
    Restart,
    
    Logs {
        #[arg(short, long, default_value = "20")]
        lines: u16,
        
        #[arg(short, long)]
        follow: bool,
    },
    
    Ps,
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
    pid_file: String,
}

impl McpManager {
    fn new(install_dir: Option<String>) -> Self {
        let dir = install_dir.unwrap_or_else(|| DEFAULT_INSTALL_DIR.to_string());
        Self {
            install_dir: dir.clone(),
            env_file: format!("{}/.env.local", dir),
            pid_file: format!("{}/.service.pid", dir),
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
            r#"# PDF Module MCP 配置

VLM_API_KEY={}
VLM_MODEL={}
VLM_ENDPOINT={}

DASHBOARD_PORT={}
DASHBOARD_WEB_DIR={}/web/dist

STORAGE_TYPE=local
STORAGE_LOCAL_DIR={}/data

RUST_LOG={}
"#,
            config.vlm_api_key,
            config.vlm_model,
            config.vlm_endpoint,
            config.dashboard_port,
            self.install_dir,
            self.install_dir,
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
        println!("\n{}", "PDF Module MCP CLI".green().bold());
        println!();
    }
    
    fn check_process(&self, name: &str) -> Option<u32> {
        let output = Command::new("pgrep")
            .args(&["-f", name])
            .output()
            .ok()?;
        
        if output.status.success() {
            let pid_str = String::from_utf8_lossy(&output.stdout);
            pid_str.lines().next()?.trim().parse().ok()
        } else {
            None
        }
    }
    
    fn kill_process(&self, name: &str) -> bool {
        Command::new("pkill")
            .args(&["-f", name])
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
    }
    
    fn save_pid(&self, pid: u32) {
        let _ = fs::write(&self.pid_file, pid.to_string());
    }
    
    fn load_pid(&self) -> Option<u32> {
        let content = fs::read_to_string(&self.pid_file).ok()?;
        content.trim().parse().ok()
    }
    
    fn cmd_init(&self) {
        println!("\n{}", ">>> 初始化配置".cyan().bold());
        
        print!("  {} 创建目录结构...", "→".blue());
        fs::create_dir_all(&self.install_dir).ok();
        fs::create_dir_all(format!("{}/logs", self.install_dir)).ok();
        fs::create_dir_all(format!("{}/data", self.install_dir)).ok();
        fs::create_dir_all(format!("{}/wiki/raw", self.install_dir)).ok();
        println!(" {}", "✓".green());
        
        print!("  {} 创建配置文件...", "→".blue());
        if !Path::new(&self.env_file).exists() {
            let config = EnvConfig::default();
            self.save_config(&config).ok();
            println!(" {}", "✓".green());
        } else {
            println!(" {}", "已存在".blue());
        }
        
        println!("\n{} 初始化完成！", "✓".green());
    }
    
    fn cmd_config(&self, key: Option<String>, model: Option<String>, endpoint: Option<String>) {
        println!("\n{}", ">>> 配置 API".cyan().bold());
        
        let mut config = self.load_config();
        let mut changed = false;
        
        if let Some(k) = key {
            config.vlm_api_key = k;
            changed = true;
            println!("  {} API Key 已设置", "✓".green());
        } else if config.vlm_api_key.is_empty() {
            println!("\n  获取 API Key: https://open.bigmodel.cn/ -> 控制台 -> API Keys\n");
            
            let api_key: String = Input::with_theme(&ColorfulTheme::default())
                .with_prompt("请输入 API Key")
                .interact_text()
                .unwrap();
            
            if !api_key.is_empty() {
                config.vlm_api_key = api_key;
                changed = true;
            }
        }
        
        if let Some(m) = model {
            config.vlm_model = m;
            changed = true;
            println!("  {} 模型: {}", "✓".green(), config.vlm_model);
        }
        
        if let Some(e) = endpoint {
            config.vlm_endpoint = e;
            changed = true;
            println!("  {} 端点已设置", "✓".green());
        }
        
        if changed {
            self.save_config(&config).ok();
            println!("\n{} 配置已保存", "✓".green());
        }
        
        self.show_config_summary(&config);
    }
    
    fn cmd_config_interactive(&self) {
        use std::io::{self, Write};
        
        println!("\n{}", ">>> API 配置".cyan().bold());
        
        loop {
            let config = self.load_config();
            
            println!("\n  {}", "当前配置:".yellow());
            if config.vlm_api_key.is_empty() {
                println!("    {} API Key: {}", "✗".red(), "未配置".red());
            } else {
                let masked = format!("{}****", &config.vlm_api_key[..8.min(config.vlm_api_key.len())]);
                println!("    {} API Key: {}", "✓".green(), masked);
            }
            println!("    {} 模型: {}", "→".blue(), config.vlm_model);
            
            println!("\n  {} 配置 API Key", "1".cyan());
            println!("  {} 配置模型", "2".cyan());
            println!("  {} 返回", "0".cyan());
            print!("\n  选择: ");
            io::stdout().flush().unwrap();
            
            let mut input = String::new();
            io::stdin().read_line(&mut input).unwrap();
            
            match input.trim() {
                "1" => {
                    println!("\n  获取 API Key: https://open.bigmodel.cn/");
                    print!("  输入 API Key: ");
                    io::stdout().flush().unwrap();
                    
                    let mut key = String::new();
                    io::stdin().read_line(&mut key).unwrap();
                    
                    if !key.trim().is_empty() {
                        let mut config = self.load_config();
                        config.vlm_api_key = key.trim().to_string();
                        self.save_config(&config).ok();
                        println!("  {} 已保存", "✓".green());
                    }
                }
                "2" => {
                    println!("\n  {} glm-4v-flash (推荐)", "1".cyan());
                    println!("  {} glm-4v-plus", "2".cyan());
                    print!("  选择 [1]: ");
                    io::stdout().flush().unwrap();
                    
                    let mut m = String::new();
                    io::stdin().read_line(&mut m).unwrap();
                    
                    let model = match m.trim() {
                        "2" => "glm-4v-plus",
                        _ => "glm-4v-flash",
                    };
                    
                    let mut config = self.load_config();
                    config.vlm_model = model.to_string();
                    self.save_config(&config).ok();
                    println!("  {} 模型: {}", "✓".green(), model);
                }
                "0" | "" => break,
                _ => {}
            }
        }
    }
    
    fn cmd_status(&self) {
        println!("\n{}", ">>> 服务状态".cyan().bold());
        
        let config = self.load_config();
        
        println!("\n  {}", "配置:".yellow());
        if config.vlm_api_key.is_empty() {
            println!("    {} API Key: {}", "✗".red(), "未配置");
        } else {
            let masked = format!("{}****", &config.vlm_api_key[..8.min(config.vlm_api_key.len())]);
            println!("    {} API Key: {}", "✓".green(), masked);
        }
        println!("    {} 模型: {}", "→".blue(), config.vlm_model);
        println!("    {} 端口: {}", "→".blue(), config.dashboard_port);
        
        println!("\n  {}", "进程:".yellow());
        
        if let Some(pid) = self.check_process("pdf-mcp.*dashboard") {
            println!("    {} Dashboard 服务运行中 (PID: {})", "✓".green(), pid);
            println!("      访问: http://localhost:{}", config.dashboard_port);
        } else {
            println!("    {} Dashboard 服务未运行", "○".blue());
        }
        
        if let Some(pid) = self.check_process("serve.*dist") {
            println!("    {} Web 前端运行中 (PID: {})", "✓".green(), pid);
        } else {
            println!("    {} Web 前端未运行", "○".blue());
        }
    }
    
    fn cmd_ps(&self) {
        println!("\n{}", ">>> 进程列表".cyan().bold());
        
        println!("\n  {:<8} {:<20} {}", "PID".cyan(), "名称".cyan(), "状态".cyan());
        println!("  {}", "-".repeat(40));
        
        let processes = vec![
            ("pdf-mcp.*dashboard", "Dashboard API"),
            ("serve.*dist", "Web 前端"),
            ("pdf-mcp$", "MCP Server"),
        ];
        
        let mut found = false;
        for (pattern, name) in processes {
            if let Some(pid) = self.check_process(pattern) {
                println!("  {:<8} {:<20} {}", pid.to_string().white(), name, "运行中".green());
                found = true;
            }
        }
        
        if !found {
            println!("  {}", "无运行中的进程".blue());
        }
    }
    
    fn cmd_generate_config(&self, output: Option<String>) {
        println!("\n{}", ">>> 生成客户端配置".cyan().bold());
        
        let mcp_config = serde_json::json!({
            "mcpServers": {
                "pdf-module": {
                    "command": format!("{}/pdf-mcp", self.install_dir),
                    "env": {
                        "VLM_API_KEY": "",
                        "VLM_MODEL": DEFAULT_VLM_MODEL,
                        "VLM_ENDPOINT": DEFAULT_VLM_ENDPOINT
                    }
                }
            }
        });
        
        let config_str = serde_json::to_string_pretty(&mcp_config).unwrap();
        
        if let Some(out_path) = output {
            match fs::write(&out_path, &config_str) {
                Ok(_) => println!("  {} 已保存到: {}", "✓".green(), out_path),
                Err(e) => println!("  {} 写入失败: {}", "✗".red(), e),
            }
        } else {
            println!("\n{}", config_str);
        }
    }
    
    fn cmd_start(&self, web: bool) {
        println!("\n{}", ">>> 启动服务".cyan().bold());
        
        let config = self.load_config();
        
        if !web {
            println!("  {} MCP 服务按需启动，无需手动启动", "ℹ".blue());
            println!("  使用 --web 参数启动 Dashboard");
            return;
        }
        
        let dashboard_binary = format!("{}/pdf-mcp", self.install_dir);
        let web_dist = format!("{}/web/dist", self.install_dir);
        
        if !Path::new(&dashboard_binary).exists() {
            println!("  {} pdf-mcp 不存在", "✗".red());
            return;
        }
        
        if !Path::new(&web_dist).exists() {
            println!("  {} Web 前端不存在", "✗".red());
            return;
        }
        
        if self.check_process("pdf-mcp.*dashboard").is_some() {
            println!("  {} Dashboard 已在运行", "ℹ".blue());
        } else {
            print!("  {} 启动 Dashboard...", "→".blue());
            
            let result = Command::new(&dashboard_binary)
                .args(&["dashboard", "--port", &config.dashboard_port.to_string()])
                .current_dir(&self.install_dir)
                .env("VLM_API_KEY", &config.vlm_api_key)
                .env("VLM_MODEL", &config.vlm_model)
                .env("VLM_ENDPOINT", &config.vlm_endpoint)
                .env("DASHBOARD_PORT", config.dashboard_port.to_string())
                .env("DASHBOARD_WEB_DIR", format!("{}/web/dist", self.install_dir))
                .env("RUST_LOG", &config.rust_log)
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn();
            
            match result {
                Ok(child) => {
                    self.save_pid(child.id());
                    std::thread::sleep(std::time::Duration::from_millis(500));
                    
                    if self.check_process("pdf-mcp.*dashboard").is_some() {
                        println!(" {}", "✓".green());
                    } else {
                        println!(" {}", "✗ 启动失败".red());
                        return;
                    }
                }
                Err(e) => {
                    println!(" {} {}", "✗".red(), e);
                    return;
                }
            }
        }
        
        if self.check_process("serve.*dist").is_some() {
            println!("  {} Web 前端已在运行", "ℹ".blue());
        } else {
            print!("  {} 启动 Web 前端...", "→".blue());
            
            let web_dir = format!("{}/web", self.install_dir);
            let result = Command::new("npx")
                .args(&["serve", "dist", "-p", "8080", "-s"])
                .current_dir(&web_dir)
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn();
            
            match result {
                Ok(_) => {
                    std::thread::sleep(std::time::Duration::from_millis(500));
                    
                    if self.check_process("serve.*dist").is_some() {
                        println!(" {}", "✓".green());
                    } else {
                        println!(" {}", "✗ 启动失败".red());
                    }
                }
                Err(e) => {
                    println!(" {} {}", "✗".red(), e);
                }
            }
        }
        
        println!("\n  {} 访问地址: http://localhost:8080", "→".blue());
        println!("  {} API 地址: http://localhost:{}", "→".blue(), config.dashboard_port);
    }
    
    fn cmd_stop(&self) {
        println!("\n{}", ">>> 停止服务".cyan().bold());
        
        print!("  {} 停止 Dashboard...", "→".blue());
        if self.kill_process("pdf-mcp.*dashboard") {
            println!(" {}", "✓".green());
        } else {
            println!(" {}", "未运行".blue());
        }
        
        print!("  {} 停止 Web 前端...", "→".blue());
        if self.kill_process("serve.*dist") {
            println!(" {}", "✓".green());
        } else {
            println!(" {}", "未运行".blue());
        }
        
        let _ = fs::remove_file(&self.pid_file);
    }
    
    fn cmd_restart(&self) {
        self.cmd_stop();
        std::thread::sleep(std::time::Duration::from_millis(500));
        self.cmd_start(true);
    }
    
    fn cmd_logs(&self, lines: u16, follow: bool) {
        let log_file = format!("{}/logs/latest.log", self.install_dir);
        
        if !Path::new(&log_file).exists() {
            println!("  {} 日志文件不存在", "✗".red());
            return;
        }
        
        if follow {
            let _ = Command::new("tail")
                .args(&["-f", "-n", &lines.to_string(), &log_file])
                .status();
        } else {
            let _ = Command::new("tail")
                .args(&["-n", &lines.to_string(), &log_file])
                .status();
        }
    }
    
    fn show_config_summary(&self, config: &EnvConfig) {
        println!("\n  {}", "配置摘要:".yellow());
        if config.vlm_api_key.is_empty() {
            println!("    {} API Key: {}", "✗".red(), "未配置");
        } else {
            let masked = format!("{}****", &config.vlm_api_key[..8.min(config.vlm_api_key.len())]);
            println!("    {} API Key: {}", "✓".green(), masked);
        }
        println!("    {} 模型: {}", "→".blue(), config.vlm_model);
        println!("    {} 端点: {}", "→".blue(), config.vlm_endpoint);
    }
}

fn main() {
    let cli = Cli::parse();
    let manager = McpManager::new(None);
    
    if cli.command.is_none() {
        manager.show_banner();
        interactive_menu(&manager);
        return;
    }
    
    manager.show_banner();
    
    match cli.command {
        None => {}
        Some(Commands::Init) => manager.cmd_init(),
        Some(Commands::Config { key, model, endpoint }) => {
            manager.cmd_config(key, model, endpoint)
        }
        Some(Commands::Status) => manager.cmd_status(),
        Some(Commands::GenerateConfig { output }) => {
            manager.cmd_generate_config(output)
        }
        Some(Commands::Start { web }) => manager.cmd_start(web),
        Some(Commands::Stop) => manager.cmd_stop(),
        Some(Commands::Restart) => manager.cmd_restart(),
        Some(Commands::Logs { lines, follow }) => manager.cmd_logs(lines, follow),
        Some(Commands::Ps) => manager.cmd_ps(),
    }
}

fn interactive_menu(manager: &McpManager) {
    use std::io::{self, Write};
    
    loop {
        let config = manager.load_config();
        
        println!("\n  {}", "主菜单".cyan().bold());
        println!("  {}", "─".repeat(30));
        
        if config.vlm_api_key.is_empty() {
            println!("  {} 配置 API (未配置)", "1".cyan());
        } else {
            println!("  {} 配置 API (已配置)", "1".cyan());
        }
        
        println!("  {} 查看状态", "2".cyan());
        println!("  {} 启动服务", "3".cyan());
        println!("  {} 停止服务", "4".cyan());
        println!("  {} 重启服务", "5".cyan());
        println!("  {} 查看进程", "6".cyan());
        println!("  {} 查看日志", "7".cyan());
        println!("  {} 生成客户端配置", "8".cyan());
        println!("  {} 退出", "0".cyan());
        
        print!("\n  选择: ");
        io::stdout().flush().unwrap();
        
        let mut input = String::new();
        if io::stdin().read_line(&mut input).is_err() {
            break;
        }
        
        match input.trim() {
            "1" => manager.cmd_config_interactive(),
            "2" => manager.cmd_status(),
            "3" => manager.cmd_start(true),
            "4" => manager.cmd_stop(),
            "5" => manager.cmd_restart(),
            "6" => manager.cmd_ps(),
            "7" => {
                print!("  行数 [20]: ");
                io::stdout().flush().unwrap();
                let mut lines = String::new();
                io::stdin().read_line(&mut lines).unwrap();
                let n: u16 = lines.trim().parse().unwrap_or(20);
                manager.cmd_logs(n, false);
            }
            "8" => manager.cmd_generate_config(None),
            "0" | "q" | "quit" | "exit" => {
                println!("\n  再见！\n");
                break;
            }
            _ => {}
        }
    }
}
