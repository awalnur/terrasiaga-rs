# 🚀 Deployment Guide - Terra Siaga

## 📋 Overview

Panduan deployment Terra Siaga untuk berbagai environment: development, staging, dan production.

## 🐳 Docker Deployment

### Production Docker Setup

1. **Dockerfile Optimized**
```dockerfile
# Multi-stage build for smaller image
FROM rust:1.70 as builder

WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src ./src

# Build release binary
RUN cargo build --release

# Runtime image
FROM debian:bullseye-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl1.1 \
    libpq5 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY --from=builder /app/target/release/terra-siaga ./

EXPOSE 8080
CMD ["./terra-siaga"]
```

2. **Docker Compose Production**
```yaml
version: '3.8'
services:
  app:
    image: terra-siaga:latest
    ports:
      - "8080:8080"
    environment:
      - DATABASE_URL=${DATABASE_URL}
      - REDIS_URL=${REDIS_URL}
      - JWT_SECRET=${JWT_SECRET}
    depends_on:
      - postgres
      - redis
    restart: unless-stopped
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:8080/health"]
      interval: 30s
      timeout: 10s
      retries: 3

  postgres:
    image: postgres:14
    environment:
      POSTGRES_DB: terrasiaga
      POSTGRES_USER: ${DB_USER}
      POSTGRES_PASSWORD: ${DB_PASSWORD}
    volumes:
      - postgres_data:/var/lib/postgresql/data
      - ./backups:/backups
    restart: unless-stopped

  redis:
    image: redis:6-alpine
    command: redis-server --appendonly yes
    volumes:
      - redis_data:/data
    restart: unless-stopped

  nginx:
    image: nginx:alpine
    ports:
      - "80:80"
      - "443:443"
    volumes:
      - ./nginx.conf:/etc/nginx/nginx.conf
      - ./ssl:/etc/nginx/ssl
    depends_on:
      - app
    restart: unless-stopped

volumes:
  postgres_data:
  redis_data:
```

### Build & Deploy Commands

```bash
# Build production image
docker build -t terra-siaga:latest .

# Deploy with compose
docker-compose -f docker-compose.prod.yml up -d

# View logs
docker-compose logs -f app

# Update deployment
docker-compose pull
docker-compose up -d --no-deps app
```

## ☁️ Cloud Deployment

### AWS ECS Deployment

1. **Task Definition**
```json
{
  "family": "terra-siaga",
  "networkMode": "awsvpc",
  "requiresCompatibilities": ["FARGATE"],
  "cpu": "512",
  "memory": "1024",
  "executionRoleArn": "arn:aws:iam::account:role/ecsTaskExecutionRole",
  "containerDefinitions": [
    {
      "name": "terra-siaga",
      "image": "your-ecr-repo/terra-siaga:latest",
      "portMappings": [
        {
          "containerPort": 8080,
          "protocol": "tcp"
        }
      ],
      "environment": [
        {
          "name": "DATABASE_URL",
          "value": "${DATABASE_URL}"
        }
      ],
      "logConfiguration": {
        "logDriver": "awslogs",
        "options": {
          "awslogs-group": "/ecs/terra-siaga",
          "awslogs-region": "ap-southeast-1",
          "awslogs-stream-prefix": "ecs"
        }
      }
    }
  ]
}
```

2. **Service Definition**
```bash
# Create ECS service
aws ecs create-service \
  --cluster terra-siaga-cluster \
  --service-name terra-siaga-service \
  --task-definition terra-siaga:1 \
  --desired-count 2 \
  --launch-type FARGATE \
  --network-configuration "awsvpcConfiguration={subnets=[subnet-12345],securityGroups=[sg-12345],assignPublicIp=ENABLED}"
```

### DigitalOcean App Platform

```yaml
# .do/app.yaml
name: terra-siaga
services:
- name: api
  source_dir: /
  github:
    repo: your-org/terra-siaga
    branch: main
  run_command: ./target/release/terra-siaga
  environment_slug: rust
  instance_count: 2
  instance_size_slug: basic-xxs
  envs:
  - key: DATABASE_URL
    value: ${db.DATABASE_URL}
  - key: REDIS_URL
    value: ${redis.REDIS_URL}
  health_check:
    http_path: /health

databases:
- name: db
  engine: PG
  num_nodes: 1
  size: db-s-dev-database
  
- name: redis
  engine: REDIS
  num_nodes: 1
  size: db-s-dev-database
```

## 🔧 Server Deployment

### Ubuntu Server Setup

1. **Initial Server Setup**
```bash
# Update system
sudo apt update && sudo apt upgrade -y

# Install required packages
sudo apt install -y curl build-essential pkg-config libssl-dev libpq-dev

# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# Install PostgreSQL
sudo apt install postgresql postgresql-contrib
sudo systemctl start postgresql
sudo systemctl enable postgresql

# Install Redis
sudo apt install redis-server
sudo systemctl start redis
sudo systemctl enable redis

# Install Nginx
sudo apt install nginx
sudo systemctl start nginx
sudo systemctl enable nginx
```

2. **Application Setup**
```bash
# Create app user
sudo useradd -m -s /bin/bash terrasiaga
sudo usermod -aG sudo terrasiaga

# Clone and build
sudo -u terrasiaga git clone https://github.com/your-org/terra-siaga.git /home/terrasiaga/app
cd /home/terrasiaga/app
sudo -u terrasiaga cargo build --release

# Setup environment
sudo -u terrasiaga cp .env.example .env
sudo -u terrasiaga nano .env  # Configure environment

# Run migrations
sudo -u terrasiaga diesel migration run
```

3. **Systemd Service**
```ini
# /etc/systemd/system/terra-siaga.service
[Unit]
Description=Terra Siaga Emergency Response System
After=network.target postgresql.service redis.service

[Service]
Type=exec
User=terrasiaga
Group=terrasiaga
WorkingDirectory=/home/terrasiaga/app
ExecStart=/home/terrasiaga/app/target/release/terra-siaga
Restart=always
RestartSec=5
Environment=RUST_LOG=info

[Install]
WantedBy=multi-user.target
```

```bash
# Enable and start service
sudo systemctl daemon-reload
sudo systemctl enable terra-siaga
sudo systemctl start terra-siaga
sudo systemctl status terra-siaga
```

### Nginx Configuration

```nginx
# /etc/nginx/sites-available/terra-siaga
server {
    listen 80;
    server_name api.terrasiaga.id;
    
    # Redirect to HTTPS
    return 301 https://$server_name$request_uri;
}

server {
    listen 443 ssl http2;
    server_name api.terrasiaga.id;
    
    # SSL Configuration
    ssl_certificate /etc/letsencrypt/live/api.terrasiaga.id/fullchain.pem;
    ssl_certificate_key /etc/letsencrypt/live/api.terrasiaga.id/privkey.pem;
    ssl_protocols TLSv1.2 TLSv1.3;
    ssl_ciphers ECDHE-RSA-AES256-GCM-SHA512:DHE-RSA-AES256-GCM-SHA512;
    
    # Security headers
    add_header X-Frame-Options DENY;
    add_header X-Content-Type-Options nosniff;
    add_header X-XSS-Protection "1; mode=block";
    add_header Strict-Transport-Security "max-age=31536000; includeSubDomains";
    
    # Rate limiting
    limit_req_zone $binary_remote_addr zone=api:10m rate=10r/s;
    limit_req zone=api burst=20 nodelay;
    
    location / {
        proxy_pass http://127.0.0.1:8080;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        
        # Timeouts
        proxy_connect_timeout 60s;
        proxy_send_timeout 60s;
        proxy_read_timeout 60s;
    }
    
    # Health check endpoint (no rate limiting)
    location /health {
        proxy_pass http://127.0.0.1:8080/health;
        access_log off;
    }
}
```

## 🔐 SSL/TLS Setup

### Let's Encrypt with Certbot

```bash
# Install Certbot
sudo apt install certbot python3-certbot-nginx

# Obtain certificate
sudo certbot --nginx -d api.terrasiaga.id

# Auto-renewal
sudo crontab -e
# Add: 0 12 * * * /usr/bin/certbot renew --quiet
```

## 📊 Monitoring & Observability

### Prometheus & Grafana Setup

1. **Prometheus Configuration**
```yaml
# prometheus.yml
global:
  scrape_interval: 15s

scrape_configs:
  - job_name: 'terra-siaga'
    static_configs:
      - targets: ['localhost:8080']
    metrics_path: '/metrics'
    scrape_interval: 5s
```

2. **Grafana Dashboard**
```json
{
  "dashboard": {
    "title": "Terra Siaga Metrics",
    "panels": [
      {
        "title": "HTTP Request Rate",
        "type": "graph",
        "targets": [
          {
            "expr": "rate(http_requests_total[5m])"
          }
        ]
      }
    ]
  }
}
```

### Log Management

```bash
# Logrotate configuration
# /etc/logrotate.d/terra-siaga
/var/log/terra-siaga/*.log {
    daily
    missingok
    rotate 30
    compress
    delaycompress
    notifempty
    create 644 terrasiaga terrasiaga
    postrotate
        systemctl reload terra-siaga
    endscript
}
```

## 🔄 CI/CD Pipeline

### GitHub Actions

```yaml
# .github/workflows/deploy.yml
name: Deploy to Production

on:
  push:
    branches: [main]

jobs:
  deploy:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      
      - name: Setup Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
          
      - name: Run tests
        run: cargo test
        
      - name: Build release
        run: cargo build --release
        
      - name: Build Docker image
        run: |
          docker build -t terra-siaga:${{ github.sha }} .
          docker tag terra-siaga:${{ github.sha }} terra-siaga:latest
          
      - name: Deploy to server
        uses: appleboy/ssh-action@v0.1.5
        with:
          host: ${{ secrets.HOST }}
          username: ${{ secrets.USERNAME }}
          key: ${{ secrets.SSH_KEY }}
          script: |
            cd /home/terrasiaga/app
            git pull origin main
            cargo build --release
            sudo systemctl restart terra-siaga
```

## 🗄️ Database Management

### Backup Strategy

```bash
#!/bin/bash
# backup.sh
DATE=$(date +%Y%m%d_%H%M%S)
BACKUP_DIR="/backups"
DB_NAME="terrasiaga"

# Create backup
pg_dump -h localhost -U postgres $DB_NAME > $BACKUP_DIR/backup_$DATE.sql

# Compress backup
gzip $BACKUP_DIR/backup_$DATE.sql

# Remove backups older than 30 days
find $BACKUP_DIR -name "backup_*.sql.gz" -mtime +30 -delete

# Upload to S3 (optional)
aws s3 cp $BACKUP_DIR/backup_$DATE.sql.gz s3://terra-siaga-backups/
```

### Migration Deployment

```bash
#!/bin/bash
# migrate.sh
set -e

echo "Running database migrations..."
diesel migration run

echo "Verifying migration status..."
diesel migration list

echo "Migrations completed successfully"
```

## 🚨 Rollback Procedures

### Application Rollback

```bash
# Rollback to previous version
git checkout previous-stable-tag
cargo build --release
sudo systemctl restart terra-siaga

# Or using Docker
docker-compose down
docker-compose up -d --no-deps app
```

### Database Rollback

```bash
# Rollback specific migration
diesel migration revert

# Restore from backup
gunzip backup_20250723_100000.sql.gz
psql -h localhost -U postgres terrasiaga < backup_20250723_100000.sql
```

## 📋 Production Checklist

### Pre-deployment
- [ ] All tests passing
- [ ] Security audit completed
- [ ] Performance testing done
- [ ] Database migrations tested
- [ ] Environment variables configured
- [ ] SSL certificates valid
- [ ] Backup strategy in place

### Post-deployment
- [ ] Health checks passing
- [ ] Logs monitoring setup
- [ ] Metrics collection working
- [ ] API endpoints responding
- [ ] Database connections stable
- [ ] External services connected

## 🚨 Troubleshooting

### Common Issues

1. **Service won't start**
```bash
sudo journalctl -u terra-siaga -f
sudo systemctl status terra-siaga
```

2. **Database connection issues**
```bash
# Check PostgreSQL status
sudo systemctl status postgresql
pg_isready -h localhost -p 5432
```

3. **High memory usage**
```bash
# Monitor memory
htop
free -h
# Adjust service limits in systemd
```

### Emergency Procedures

```bash
# Stop service immediately
sudo systemctl stop terra-siaga

# Emergency database backup
pg_dump terrasiaga > emergency_backup.sql

# Roll back to last known good state
git checkout last-known-good-commit
cargo build --release
sudo systemctl start terra-siaga
```

---

**Always test deployments in staging environment first!** 🛡️
