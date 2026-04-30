use clap::{Parser, Subcommand};
use colored::Colorize;
use dialoguer::{theme::ColorfulTheme, Input};
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

    DownloadWeb {
        #[arg(short, long)]
        version: Option<String>,
    },
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
        println!(
            "\n{}",
            "██████╗  ██████╗ ██╗     ██╗     ██╗███╗   ██╗ ██████╗ "
                .cyan()
                .bold()
        );
        println!(
            "{}",
            "██╔══██╗██╔═══██╗██║     ██║     ██║████╗  ██║██╔════╝ "
                .cyan()
                .bold()
        );
        println!(
            "{}",
            "██████╔╝██║   ██║██║     ██║     ██║██╔██╗ ██║██║  ███╗"
                .cyan()
                .bold()
        );
        println!(
            "{}",
            "██╔═══╝ ██║   ██║██║     ██║     ██║██║╚██╗██║██║   ██║"
                .cyan()
                .bold()
        );
        println!(
            "{}",
            "██║     ╚██████╔╝███████╗███████╗██║██║ ╚████║╚██████╔╝"
                .cyan()
                .bold()
        );
        println!(
            "{}",
            "╚═╝      ╚═════╝ ╚══════╝╚══════╝╚═╝╚═╝  ╚═══╝ ╚═════╝ "
                .cyan()
                .bold()
        );
        println!("\n{}", "PDF Module MCP CLI".green().bold());
        println!();
    }

    fn check_process(&self, name: &str) -> Option<u32> {
        let output = Command::new("pgrep").args(["-f", name]).output().ok()?;

        if output.status.success() {
            let pid_str = String::from_utf8_lossy(&output.stdout);
            // 返回第一个匹配的PID
            pid_str.lines().next()?.trim().parse().ok()
        } else {
            None
        }
    }

    fn check_mcp_server(&self) -> Option<u32> {
        // MCP Server: 匹配 pdf-mcp 但排除 pdf-dashboard 和临时进程
        let output = Command::new("ps")
            .args(["aux"])
            .output()
            .ok()?;

        let output_str = String::from_utf8_lossy(&output.stdout);
        
        for line in output_str.lines() {
            if line.contains("/opt/pdf-module/pdf-mcp") 
                && !line.contains("pdf-dashboard")
                && !line.contains("--version")
                && !line.contains("--help")
                && !line.contains("dashboard") {
                // 提取PID（第二列）
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() > 1 {
                    if let Ok(pid) = parts[1].parse::<u32>() {
                        return Some(pid);
                    }
                }
            }
        }
        None
    }

    fn check_dashboard(&self) -> Option<u32> {
        // Dashboard API: 精确匹配 pdf-dashboard
        self.check_process("/opt/pdf-module/pdf-dashboard")
    }

    fn check_web_frontend(&self) -> Option<u32> {
        // Web Frontend: 精确匹配 /opt/pdf-module/web 目录下的 serve 进程
        let output = Command::new("ps")
            .args(["aux"])
            .output()
            .ok()?;

        let output_str = String::from_utf8_lossy(&output.stdout);
        let web_dir = format!("{}/web", self.install_dir);
        
        for line in output_str.lines() {
            // 精确匹配：必须在安装目录的 web 目录下运行 serve
            if line.contains(&web_dir) 
                && line.contains("serve") 
                && line.contains("8080") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() > 1 {
                    if let Ok(pid) = parts[1].parse::<u32>() {
                        return Some(pid);
                    }
                }
            }
        }
        None
    }

    fn check_port(&self, port: u16) -> bool {
        Command::new("ss")
            .args(["-tlnp"])
            .output()
            .map(|output| {
                let output_str = String::from_utf8_lossy(&output.stdout);
                output_str.contains(&format!(":{}", port))
            })
            .unwrap_or(false)
    }

    fn save_pid(&self, pid: u32) {
        let _ = fs::write(&self.pid_file, pid.to_string());
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
                let masked = format!(
                    "{}****",
                    &config.vlm_api_key[..8.min(config.vlm_api_key.len())]
                );
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
            println!("    {} API Key: 未配置", "✗".red());
        } else {
            let masked = format!(
                "{}****",
                &config.vlm_api_key[..8.min(config.vlm_api_key.len())]
            );
            println!("    {} API Key: {}", "✓".green(), masked);
        }
        println!("    {} 模型: {}", "→".blue(), config.vlm_model);
        println!("    {} 端口: {}", "→".blue(), config.dashboard_port);

        println!("\n  {}", "进程:".yellow());

        // MCP Server (即用即停)
        if let Some(pid) = self.check_mcp_server() {
            println!("    {} MCP Server 运行中 (PID: {})", "✓".green(), pid);
            println!("      说明: MCP服务即用即停，无需手动管理");
        } else {
            println!("    {} MCP Server 未运行 (按需启动)", "○".blue());
        }

        // Dashboard API
        if let Some(pid) = self.check_dashboard() {
            println!("    {} Dashboard API 运行中 (PID: {})", "✓".green(), pid);
            println!("      访问: http://localhost:{}", config.dashboard_port);
        } else {
            println!("    {} Dashboard API 未运行", "○".blue());
        }

        // Web Frontend
        if let Some(pid) = self.check_web_frontend() {
            println!("    {} Web 前端运行中 (PID: {}, 端口: 8080)", "✓".green(), pid);
        } else if self.check_port(8080) {
            println!("    {} Web 前端运行中 (端口: 8080)", "✓".green());
        } else {
            println!("    {} Web 前端未运行", "○".blue());
        }
    }

    fn cmd_ps(&self) {
        println!("\n{}", ">>> 进程列表".cyan().bold());

        println!(
            "\n  {:<8} {:<20} {}",
            "PID".cyan(),
            "名称".cyan(),
            "状态".cyan()
        );
        println!("  {}", "-".repeat(40));

        let processes = vec![
            ("MCP Server", self.check_mcp_server()),
            ("Dashboard API", self.check_dashboard()),
            ("Web Frontend", self.check_web_frontend()),
        ];

        let mut found = false;
        for (name, pid_opt) in processes {
            if let Some(pid) = pid_opt {
                println!(
                    "  {:<8} {:<20} {}",
                    pid.to_string().white(),
                    name,
                    "运行中".green()
                );
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
            println!("  {} MCP Server 按需启动，无需手动管理", "ℹ".blue());
            println!("  使用 --web 参数启动 Dashboard 和 Web 前端");
            return;
        }

        // 检查Dashboard二进制
        let dashboard_binary = format!("{}/pdf-dashboard", self.install_dir);
        if !Path::new(&dashboard_binary).exists() {
            println!("  {} pdf-dashboard 不存在", "✗".red());
            println!("  {} 请检查安装是否完整", "ℹ".blue());
            return;
        }

        // 检查Web前端
        let web_dist = format!("{}/web/dist", self.install_dir);
        if !Path::new(&web_dist).exists() {
            println!("  {} Web 前端不存在", "✗".red());
            println!("  {} 请先下载 Web 前端:", "ℹ".blue());
            println!("     pdf-mcp-cli download-web");
            println!("  或使用交互式菜单选择下载");
            return;
        }

        // 启动Dashboard API
        if self.check_dashboard().is_some() {
            println!("  {} Dashboard API 已在运行", "ℹ".blue());
        } else {
            print!("  {} 启动 Dashboard API...", "→".blue());

            let pdfium_lib = format!("{}/lib/libpdfium.so", self.install_dir);
            let lib_dir = format!("{}/lib", self.install_dir);

            let result = Command::new(&dashboard_binary)
                .args(["--port", &config.dashboard_port.to_string()])
                .current_dir(&self.install_dir)
                .env("PDFIUM_LIB_PATH", &pdfium_lib)
                .env("LD_LIBRARY_PATH", &lib_dir)
                .env("VLM_API_KEY", &config.vlm_api_key)
                .env("VLM_MODEL", &config.vlm_model)
                .env("VLM_ENDPOINT", &config.vlm_endpoint)
                .env("RUST_LOG", &config.rust_log)
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn();

            match result {
                Ok(child) => {
                    self.save_pid(child.id());
                    std::thread::sleep(std::time::Duration::from_millis(1000));

                    if self.check_dashboard().is_some() {
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

        if self.check_port(8080) {
            println!("  {} Web 前端已在运行", "ℹ".blue());
        } else {
            print!("  {} 启动 Web 前端...", "→".blue());

            let web_dir = format!("{}/web", self.install_dir);
            let result = Command::new("npx")
                .args(["serve", "dist", "-p", "8080", "-s"])
                .current_dir(&web_dir)
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn();

            match result {
                Ok(_) => {
                    std::thread::sleep(std::time::Duration::from_millis(3000));

                    if self.check_port(8080) {
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
        println!(
            "  {} API 地址: http://localhost:{}",
            "→".blue(),
            config.dashboard_port
        );
    }

    fn cmd_stop(&self) {
        println!("\n{}", ">>> 停止服务".cyan().bold());

        print!("  {} 停止 Dashboard API...", "→".blue());
        if let Some(pid) = self.check_dashboard() {
            let _ = Command::new("kill")
                .args([&pid.to_string()])
                .status();
            std::thread::sleep(std::time::Duration::from_millis(500));
            
            if self.check_dashboard().is_none() {
                println!(" {}", "✓".green());
            } else {
                println!(" {}", "✗ 停止失败".red());
            }
        } else {
            println!(" {}", "未运行".blue());
        }

        print!("  {} 停止 Web 前端...", "→".blue());
        if let Some(pid) = self.check_web_frontend() {
            let _ = Command::new("kill")
                .args([&pid.to_string()])
                .status();
            std::thread::sleep(std::time::Duration::from_millis(500));
            
            if self.check_web_frontend().is_none() && !self.check_port(8080) {
                println!(" {}", "✓".green());
            } else {
                println!(" {}", "✗ 停止失败".red());
            }
        } else if self.check_port(8080) {
            // 端口被其他进程占用，不要误杀！
            println!(" {}", "端口 8080 被其他进程占用，请手动检查".yellow());
            println!("    提示: 使用 'ss -tlnp | grep 8080' 查看占用进程");
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

    fn cmd_download_web(&self, version: Option<String>) {
        println!("\n{}", ">>> 下载 Web 前端".cyan().bold());

        let web_dist = format!("{}/web/dist", self.install_dir);
        
        if Path::new(&web_dist).exists() {
            println!("  {} Web 前端已存在", "ℹ".blue());
            print!("  {} 是否重新下载? (y/N): ", "?".yellow());
            use std::io::{self, BufRead};
            let stdin = io::stdin();
            let input = stdin.lock().lines().next().unwrap_or(Ok(String::new())).unwrap_or_default();
            
            if !input.to_lowercase().starts_with('y') {
                println!("  {} 已取消", "ℹ".blue());
                return;
            }
        }

        let version = version.unwrap_or_else(|| {
            print!("  {} 获取最新版本...", "→".blue());
            let output = Command::new("curl")
                .args(["-s", "https://api.github.com/repos/smile9493/rsut_pdf_mcp/releases/latest"])
                .output();
            
            match output {
                Ok(output) => {
                    let response = String::from_utf8_lossy(&output.stdout);
                    for line in response.lines() {
                        if line.contains("\"tag_name\":") {
                            let version = line.split(':').nth(1).unwrap_or("\"v0.1.3\"")
                                .trim().trim_matches(',').trim_matches('"').to_string();
                            println!(" {}", version.green());
                            return version;
                        }
                    }
                }
                Err(_) => {}
            }
            println!(" {}", "v0.1.3 (默认)".blue());
            "v0.1.3".to_string()
        });

        let download_url = format!(
            "https://github.com/smile9493/rsut_pdf_mcp/releases/download/{}/web-dist.tar.gz",
            version
        );

        print!("  {} 下载 Web 前端...", "→".blue());
        
        let web_dir = format!("{}/web", self.install_dir);
        let temp_file = format!("{}/web-dist.tar.gz", web_dir);

        fs::create_dir_all(&web_dir).ok();

        let result = Command::new("curl")
            .args(["-fsSL", "-o", &temp_file, &download_url])
            .status();

        match result {
            Ok(status) if status.success() => {
                println!(" {}", "✓".green());
                
                print!("  {} 解压文件...", "→".blue());
                
                // 清理旧的dist目录
                if Path::new(&web_dist).exists() {
                    fs::remove_dir_all(&web_dist).ok();
                }

                let extract_result = Command::new("tar")
                    .args(["-xzf", &temp_file, "-C", &web_dir])
                    .status();

                match extract_result {
                    Ok(status) if status.success() => {
                        println!(" {}", "✓".green());
                        
                        // 清理临时文件
                        fs::remove_file(&temp_file).ok();
                        
                        println!("\n  {} Web 前端下载完成！", "✓".green());
                        println!("  {} 版本: {}", "→".blue(), version);
                        println!("  {} 位置: {}", "→".blue(), web_dist);
                    }
                    _ => {
                        println!(" {}", "✗ 解压失败".red());
                        fs::remove_file(&temp_file).ok();
                    }
                }
            }
            _ => {
                println!(" {}", "✗ 下载失败".red());
                println!("  {} 请检查网络连接或使用代理", "ℹ".blue());
            }
        }
    }

    fn cmd_logs(&self, lines: u16, follow: bool) {
        let log_file = format!("{}/logs/latest.log", self.install_dir);

        if !Path::new(&log_file).exists() {
            println!("  {} 日志文件不存在", "✗".red());
            return;
        }

        if follow {
            let _ = Command::new("tail")
                .args(["-f", "-n", &lines.to_string(), &log_file])
                .status();
        } else {
            let _ = Command::new("tail")
                .args(["-n", &lines.to_string(), &log_file])
                .status();
        }
    }

    fn show_config_summary(&self, config: &EnvConfig) {
        println!("\n  {}", "配置摘要:".yellow());
        if config.vlm_api_key.is_empty() {
            println!("    {} API Key: 未配置", "✗".red());
        } else {
            let masked = format!(
                "{}****",
                &config.vlm_api_key[..8.min(config.vlm_api_key.len())]
            );
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
        Some(Commands::Config {
            key,
            model,
            endpoint,
        }) => manager.cmd_config(key, model, endpoint),
        Some(Commands::Status) => manager.cmd_status(),
        Some(Commands::GenerateConfig { output }) => manager.cmd_generate_config(output),
        Some(Commands::Start { web }) => manager.cmd_start(web),
        Some(Commands::Stop) => manager.cmd_stop(),
        Some(Commands::Restart) => manager.cmd_restart(),
        Some(Commands::Logs { lines, follow }) => manager.cmd_logs(lines, follow),
        Some(Commands::Ps) => manager.cmd_ps(),
        Some(Commands::DownloadWeb { version }) => manager.cmd_download_web(version),
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
        println!("  {} 下载 Web 前端", "9".cyan());
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
            "9" => manager.cmd_download_web(None),
            "0" | "q" | "quit" | "exit" => {
                println!("\n  再见！\n");
                break;
            }
            _ => {}
        }
    }
}
