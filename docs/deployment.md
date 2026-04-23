# 部署指南

本文档提供了 PDF Module MCP 服务器的部署指南。

## 系统要求

### 最低要求

- **操作系统**: Linux (Ubuntu 20.04+, Debian 10+, CentOS 8+)
- **CPU**: 2 核心处理器
- **内存**: 4GB RAM
- **磁盘**: 20GB 可用空间
- **网络**: 稳定的互联网连接

### 推荐配置

- **操作系统**: Linux (Ubuntu 22.04+, Debian 12+)
- **CPU**: 4 核心处理器
- **内存**: 8GB RAM
- **磁盘**: 50GB SSD
- **网络**: 1Gbps 带宽

## 部署方式

### 1. Docker 部署

#### 使用 Dockerfile

1. **构建镜像**:
```bash
docker build -t pdf-module:latest .
```

2. **运行容器**:
```bash
docker run -d \
  --name pdf-module \
  -p 8000:8000 \
  -p 8001:8001 \
  -v $(pwd)/data:/app/data \
  -v $(pwd)/logs:/app/logs/audit \
  -e STORAGE_TYPE=local \
  -e CACHE_ENABLED=true \
  pdf-module:latest
```

3. **查看日志**:
```bash
docker logs -f pdf-module
```

#### 使用 Docker Compose

1. **启动服务**:
```bash
docker-compose up -d
```

2. **查看状态**:
```bash
docker-compose ps
```

3. **停止服务**:
```bash
docker-compose down
```

4. **重启服务**:
```bash
docker-compose restart
```

### 2. Kubernetes 部署

#### 创建 Kubernetes 部署文件

```yaml
# deployment.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: pdf-module
  labels:
    app: pdf-module
spec:
  replicas: 3
  selector:
    matchLabels:
      app: pdf-module
  template:
    metadata:
      labels:
        app: pdf-module
    spec:
      containers:
      - name: pdf-rest
        image: pdf-module:latest
        ports:
        - containerPort: 8000
        env:
        - name: STORAGE_TYPE
          value: "s3"
        - name: STORAGE_S3_BUCKET
          value: "my-pdf-bucket"
        - name: CACHE_ENABLED
          value: "true"
        resources:
          requests:
            memory: "256Mi"
            cpu: "250m"
          limits:
            memory: "512Mi"
            cpu: "500m"
        livenessProbe:
          httpGet:
            path: /api/v1/x2text/health
            port: 8000
          initialDelaySeconds: 30
          periodSeconds: 10
        readinessProbe:
          httpGet:
            path: /api/v1/x2text/health
            port: 8000
          initialDelaySeconds: 5
          periodSeconds: 5
---
apiVersion: v1
kind: Service
metadata:
  name: pdf-module-service
spec:
  selector:
    app: pdf-module
  ports:
  - protocol: TCP
    port: 8000
    targetPort: 8000
  type: LoadBalancer
```

#### 部署到 Kubernetes

```bash
kubectl apply -f deployment.yaml
```

### 3. 传统部署

#### 使用 systemd 服务

1. **创建服务文件**:
```bash
sudo nano /etc/systemd/system/pdf-module.service
```

```ini
[Unit]
Description=PDF Module MCP Server
After=network.target

[Service]
Type=simple
User=pdfuser
Group=pdfuser
WorkingDirectory=/opt/pdf-module
Environment="RUST_LOG=info"
Environment="STORAGE_TYPE=local"
Environment="CACHE_ENABLED=true"
ExecStart=/usr/local/bin/pdf-rest --host 0.0.0.0 --port 8000
Restart=always
RestartSec=10

[Install]
WantedBy=multi-user.target
```

2. **启用并启动服务**:
```bash
sudo systemctl daemon-reload
sudo systemctl enable pdf-module
sudo systemctl start pdf-module
sudo systemctl status pdf-module
```

3. **查看日志**:
```bash
sudo journalctl -u pdf-module -f
```

### 4. 云平台部署

#### AWS 部署

1. **使用 ECS**:
   - 创建 ECS 集群
   - 上传 Docker 镜像到 ECR
   - 创建 ECS 任务定义
   - 配置负载均衡器

2. **使用 Lambda**:
   - 创建 Lambda 函数
   - 上传部署包
   - 配置 API Gateway
   - 设置环境变量

#### Google Cloud 部署

1. **使用 Cloud Run**:
```bash
gcloud run deploy pdf-module \
  --image gcr.io/PROJECT_ID/pdf-module \
  --platform managed \
  --region us-central1 \
  --allow-unauthenticated \
  --set-env-vars STORAGE_TYPE=local,CACHE_ENABLED=true
```

2. **使用 GKE**:
   - 创建 GKE 集群
   - 部署到 Kubernetes
   - 配置 Cloud Load Balancing

#### Azure 部署

1. **使用 Container Instances**:
```bash
az container create \
  --resource-group pdf-module-rg \
  --name pdf-module \
  --image pdf-module:latest \
  --ports 8000 \
  --environment-variables STORAGE_TYPE=local CACHE_ENABLED=true
```

2. **使用 AKS**:
   - 创建 AKS 集群
   - 部署到 Kubernetes
   - 配置 Azure Load Balancer

## 数据库配置

### 使用 PostgreSQL 存储审计日志

1. **创建数据库**:
```sql
CREATE DATABASE pdf_module_audit;
```

2. **创建表**:
```sql
CREATE TABLE audit_logs (
    id SERIAL PRIMARY KEY,
    created_at TIMESTAMP DEFAULT NOW(),
    file_name VARCHAR(255),
    file_type VARCHAR(50),
    file_size_kb FLOAT,
    adapter_used VARCHAR(50),
    status VARCHAR(50),
    processing_time_ms BIGINT,
    cache_hit BOOLEAN,
    extracted_text_length BIGINT,
    error_message TEXT
);
```

3. **配置连接**:
```bash
export DATABASE_URL=postgresql://user:password@localhost:5432/pdf_module_audit
```

### 使用 Redis 作为缓存后端

1. **安装 Redis**:
```bash
sudo apt-get install redis-server
sudo systemctl start redis-server
```

2. **配置连接**:
```bash
export REDIS_URL=redis://localhost:6379
```

## 负载均衡

### 使用 Nginx

1. **创建 Nginx 配置**:
```nginx
upstream pdf_module {
    server localhost:8000;
    server localhost:8001;
    server localhost:8002;
}

server {
    listen 80;
    server_name pdf.example.com;

    location / {
        proxy_pass http://pdf_module;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }
}
```

2. **重启 Nginx**:
```bash
sudo nginx -t
sudo systemctl reload nginx
```

### 使用 HAProxy

1. **创建 HAProxy 配置**:
```haproxy
frontend pdf_frontend
    bind *:80
    default_backend pdf_backend

backend pdf_backend
    balance roundrobin
    server pdf1 localhost:8000 check
    server pdf2 localhost:8001 check
    server pdf3 localhost:8002 check
```

2. **重启 HAProxy**:
```bash
sudo systemctl restart haproxy
```

## 监控和日志

### 使用 Prometheus 监控

1. **配置 Prometheus 抓取**:
```yaml
scrape_configs:
  - job_name: 'pdf-module'
    static_configs:
      - targets: ['localhost:8000']
    metrics_path: '/metrics'
```

2. **创建 Grafana 仪表板**:
   - 请求速率
   - 错误率
   - 响应时间
   - 缓存命中率

### 使用 ELK Stack 管理日志

1. **配置 Filebeat**:
```yaml
filebeat.inputs:
- type: log
  enabled: true
  paths:
    - /var/log/pdf-module/*.log
  fields:
    app: pdf-module
  fields_under_root: true

output.elasticsearch:
  hosts: ["localhost:9200"]
```

2. **启动 Filebeat**:
```bash
./filebeat -e -c filebeat.yml
```

## 安全配置

### SSL/TLS 配置

1. **获取 SSL 证书**:
```bash
sudo certbot certonly --standalone -d pdf.example.com
```

2. **配置 Nginx SSL**:
```nginx
server {
    listen 443 ssl http2;
    server_name pdf.example.com;

    ssl_certificate /etc/letsencrypt/live/pdf.example.com/fullchain.pem;
    ssl_certificate_key /etc/letsencrypt/live/pdf.example.com/privkey.pem;

    ssl_protocols TLSv1.2 TLSv1.3;
    ssl_ciphers HIGH:!aNULL:!MD5;

    location / {
        proxy_pass http://pdf_module;
    }
}
```

### 防火墙配置

```bash
# 开放 HTTP 和 HTTPS 端口
sudo ufw allow 80/tcp
sudo ufw allow 443/tcp

# 开放应用端口
sudo ufw allow 8000/tcp
sudo ufw allow 8001/tcp

# 启用防火墙
sudo ufw enable
```

## 备份和恢复

### 备份配置

```bash
# 备份配置文件
tar -czf pdf-module-config-backup-$(date +%Y%m%d).tar.gz \
  .env \
  docker-compose.yml \
  /etc/systemd/system/pdf-module.service

# 备份数据目录
tar -czf pdf-module-data-backup-$(date +%Y%m%d).tar.gz \
  /var/lib/pdf-module/data \
  /var/log/pdf-module
```

### 恢复配置

```bash
# 恢复配置文件
tar -xzf pdf-module-config-backup-YYYYMMDD.tar.gz

# 恢复数据目录
tar -xzf pdf-module-data-backup-YYYYMMDD.tar.gz -C /

# 重启服务
sudo systemctl restart pdf-module
```

## 性能优化

### 系统调优

1. **调整文件描述符限制**:
```bash
# 临时修改
ulimit -n 65536

# 永久修改
echo "* soft nofile 65536" >> /etc/security/limits.conf
echo "* hard nofile 65536" >> /etc/security/limits.conf
```

2. **优化内核参数**:
```bash
# 编辑 /etc/sysctl.conf
echo "net.core.somaxconn = 65535" >> /etc/sysctl.conf
echo "net.ipv4.tcp_max_syn_backlog = 8192" >> /etc/sysctl.conf
echo "net.ipv4.tcp_tw_reuse = 1" >> /etc/sysctl.conf

# 应用更改
sudo sysctl -p
```

### 应用优化

1. **调整缓存大小**:
```bash
export CACHE_MAX_SIZE_MB=500
```

2. **启用压缩**:
```bash
export ENABLE_COMPRESSION=true
```

3. **调整工作线程数**:
```bash
export WORKER_THREADS=4
```

## 故障排除

### 常见问题

1. **服务无法启动**:
   - 检查端口是否被占用: `netstat -tuln | grep 8000`
   - 检查日志文件: `journalctl -u pdf-module -n 50`
   - 验证环境变量: `env | grep PDF_MODULE`

2. **性能问题**:
   - 检查系统资源: `top`, `htop`
   - 查看缓存命中率
   - 检查数据库连接数

3. **内存泄漏**:
   - 监控内存使用: `watch -n 1 free -m`
   - 分析核心转储文件
   - 使用 Valgrind 检查内存

### 日志分析

1. **错误日志**:
```bash
grep ERROR /var/log/pdf-module/*.log
```

2. **访问日志**:
```bash
grep "POST /api/v1/x2text/extract" /var/log/pdf-module/access.log
```

3. **性能日志**:
```bash
grep "processing_time" /var/log/pdf-module/*.log
```

## 升级策略

### 滚动升级

1. **准备新版本**:
```bash
docker pull pdf-module:latest
```

2. **更新容器**:
```bash
docker-compose up -d --no-deps pdf-rest pdf-mcp
```

3. **验证升级**:
```bash
curl http://localhost:8000/api/v1/x2text/health
```

### 蓝绿部署

1. **部署新版本到新环境**
2. **测试新版本**
3. **切换流量**
4. **监控新版本**
5. **如有问题,快速回滚**

## 维护

### 定期维护任务

1. **每日**:
   - 检查服务状态
   - 查看错误日志
   - 监控资源使用

2. **每周**:
   - 清理旧日志文件
   - 检查磁盘空间
   - 审查安全日志

3. **每月**:
   - 更新依赖包
   - 审查访问日志
   - 备份配置和数据

### 紧急响应

1. **服务宕机**:
   - 检查服务状态
   - 查看错误日志
   - 重启服务
   - 通知相关人员

2. **性能下降**:
   - 监控资源使用
   - 检查缓存状态
   - 分析慢查询
   - 扩容或优化

3. **安全事件**:
   - 隔离受影响系统
   - 分析访问日志
   - 修补安全漏洞
   - 通知安全团队
