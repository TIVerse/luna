# LUNA Deployment Guide

## Production Deployment

### Prerequisites

**System Requirements:**
- Linux (Ubuntu 20.04+, Fedora 35+, Arch)
- 4GB+ RAM (8GB recommended)
- 2GB+ disk space for models
- Audio input device (microphone)
- Audio output device (speakers/headphones)

**Dependencies:**
```bash
# Ubuntu/Debian
sudo apt-get update
sudo apt-get install -y \
    build-essential \
    pkg-config \
    libasound2-dev \
    libssl-dev \
    libsqlite3-dev \
    portaudio19-dev

# Fedora
sudo dnf install -y \
    gcc \
    pkg-config \
    alsa-lib-devel \
    openssl-devel \
    sqlite-devel \
    portaudio-devel

# Arch Linux
sudo pacman -S \
    base-devel \
    pkg-config \
    alsa-lib \
    openssl \
    sqlite \
    portaudio
```

### Installation

**1. Clone and Build:**
```bash
git clone https://github.com/TIVerse/luna.git
cd luna

# Build release version
cargo build --release

# Install to system
sudo cp target/release/luna /usr/local/bin/
```

**2. Download Models:**
```bash
# Create models directory
mkdir -p models

# Download Whisper model (choose one)
# Tiny (39MB) - fastest, less accurate
wget https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-tiny.en.bin -O models/whisper-tiny.bin

# Base (74MB) - balanced (recommended)
wget https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-base.en.bin -O models/whisper-base.bin

# Small (244MB) - more accurate
wget https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-small.en.bin -O models/whisper-small.bin
```

**3. Configure:**
```bash
# Copy example config
cp config/default.toml ~/.config/luna/config.toml

# Edit configuration
nano ~/.config/luna/config.toml
```

**Minimal Configuration:**
```toml
[system]
data_dir = "/home/yourusername/.local/share/luna"
log_level = "info"

[audio]
wake_words = ["luna", "hey luna"]
sample_rate = 16000

[brain]
whisper_model_path = "/home/yourusername/luna/models/whisper-base.bin"
confidence_threshold = 0.7
```

### Running

**Start LUNA:**
```bash
# Foreground (with logs)
luna

# Background (daemon mode)
luna &

# With custom config
luna --config /path/to/config.toml

# With debug logging
RUST_LOG=debug luna
```

**Test Installation:**
```bash
# Check version
luna --version

# Doctor command (checks system)
luna doctor

# Test audio
luna test-audio
```

## Systemd Service

**Create service file:** `/etc/systemd/system/luna.service`
```ini
[Unit]
Description=LUNA Voice Assistant
After=network.target sound.target

[Service]
Type=simple
User=youruser
Group=yourgroup
Environment="RUST_LOG=info"
ExecStart=/usr/local/bin/luna --config /home/youruser/.config/luna/config.toml
Restart=on-failure
RestartSec=5s

# Security hardening
NoNewPrivileges=true
PrivateTmp=true
ProtectSystem=strict
ProtectHome=read-only
ReadWritePaths=/home/youruser/.local/share/luna /home/youruser/.config/luna

[Install]
WantedBy=multi-user.target
```

**Enable and start:**
```bash
sudo systemctl daemon-reload
sudo systemctl enable luna
sudo systemctl start luna
sudo systemctl status luna

# View logs
journalctl -u luna -f
```

## Docker Deployment

**Dockerfile:**
```dockerfile
FROM rust:1.75 as builder

WORKDIR /app
COPY . .

# Build release
RUN cargo build --release

# Runtime image
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    libasound2 \
    libssl3 \
    libsqlite3-0 \
    portaudio19 \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/luna /usr/local/bin/
COPY --from=builder /app/config /etc/luna/config

# Create directories
RUN mkdir -p /data /models

# Audio device access required
VOLUME ["/data", "/models"]

EXPOSE 9090

CMD ["luna", "--config", "/etc/luna/config/default.toml"]
```

**docker-compose.yml:**
```yaml
version: '3.8'

services:
  luna:
    build: .
    container_name: luna-assistant
    devices:
      - /dev/snd:/dev/snd  # Audio devices
    volumes:
      - ./data:/data
      - ./models:/models
      - ./config:/etc/luna/config
    environment:
      - RUST_LOG=info
    restart: unless-stopped
    ports:
      - "9090:9090"  # Prometheus metrics
```

**Run with Docker:**
```bash
# Build image
docker-compose build

# Start service
docker-compose up -d

# View logs
docker-compose logs -f

# Stop service
docker-compose down
```

## Monitoring

### Prometheus Integration

**prometheus.yml:**
```yaml
global:
  scrape_interval: 15s

scrape_configs:
  - job_name: 'luna'
    static_configs:
      - targets: ['localhost:9090']
```

**Key Metrics:**
- `luna_commands_total` - Total commands processed
- `luna_command_duration_seconds` - Command processing latency
- `luna_errors_total` - Error count
- `luna_cache_hits_total` - Cache hit rate
- `luna_audio_buffer_size` - Audio buffer usage

### Grafana Dashboard

**Import dashboard:** `docs/grafana-dashboard.json`

**Key Panels:**
- Command throughput (commands/minute)
- Average response time
- Success rate
- Cache hit rate
- Memory usage
- Audio buffer health

### Health Checks

**HTTP endpoint:**
```bash
curl http://localhost:9090/health
```

**Response:**
```json
{
  "status": "healthy",
  "components": {
    "audio": "ok",
    "brain": "ok",
    "executor": "ok",
    "tts": "ok"
  },
  "uptime_seconds": 3600,
  "version": "1.0.0"
}
```

## Security

### Production Hardening

**1. File Permissions:**
```bash
chmod 600 ~/.config/luna/config.toml
chmod 700 ~/.local/share/luna
```

**2. Firewall (if exposing metrics):**
```bash
# Allow only local Prometheus
sudo ufw allow from 127.0.0.1 to any port 9090
```

**3. User Isolation:**
```bash
# Create dedicated user
sudo useradd -r -s /bin/false luna

# Run as dedicated user
sudo -u luna luna
```

**4. AppArmor Profile:**
```bash
# Create profile at /etc/apparmor.d/usr.local.bin.luna
/usr/local/bin/luna {
  #include <abstractions/base>
  #include <abstractions/audio>
  
  /usr/local/bin/luna mr,
  /home/*/.config/luna/** r,
  /home/*/.local/share/luna/** rw,
  /tmp/** rw,
  
  # Deny network except metrics
  deny network inet,
  network inet stream addr=127.0.0.1 port=9090,
}

# Load profile
sudo apparmor_parser -r /etc/apparmor.d/usr.local.bin.luna
```

## Troubleshooting

### Audio Issues

**No microphone detected:**
```bash
# List audio devices
luna doctor

# Check ALSA
arecord -l

# Test microphone
arecord -d 5 test.wav && aplay test.wav
```

**Fix:** Update `config.toml`:
```toml
[audio]
input_device = "hw:0,0"  # Use specific device
```

### Model Loading Fails

**Error:** "Whisper model not found"

**Fix:**
```bash
# Verify model path
ls -lh models/whisper-base.bin

# Update config
[brain]
whisper_model_path = "/absolute/path/to/models/whisper-base.bin"
```

### High CPU Usage

**Cause:** Continuous audio processing

**Fix:**
- Enable VAD (voice activity detection)
- Increase silence threshold
- Use smaller Whisper model

```toml
[audio]
silence_threshold = 0.03  # Higher = less sensitive
vad_enabled = true
```

### Memory Leaks

**Monitor memory:**
```bash
# Watch memory usage
watch -n 1 'ps aux | grep luna'

# Valgrind check
valgrind --leak-check=full luna
```

**Fix:**
- Reduce cache size
- Lower conversation memory capacity
- Check for circular references

## Performance Tuning

### CPU Optimization

**Set CPU governor:**
```bash
# Performance mode
sudo cpupower frequency-set -g performance
```

**Processor affinity:**
```bash
# Pin to specific cores
taskset -c 0-3 luna
```

### Memory Optimization

**Configure:**
```toml
[brain]
cache_size = 500  # Reduce if low memory

[system]
max_conversation_memory = 50
```

**Pre-allocate:**
```bash
# Increase available memory
sudo sysctl -w vm.swappiness=10
```

### Disk I/O

**Use SSD for database:**
```toml
[system]
data_dir = "/path/to/ssd/luna"
```

**Enable WAL mode:** (automatic)

## Backup & Recovery

### Backup Configuration

**What to backup:**
- Configuration: `~/.config/luna/`
- Database: `~/.local/share/luna/luna.db`
- Logs: `~/.local/share/luna/logs/`

**Backup script:**
```bash
#!/bin/bash
BACKUP_DIR="/backup/luna-$(date +%Y%m%d)"
mkdir -p "$BACKUP_DIR"

cp -r ~/.config/luna "$BACKUP_DIR/config"
cp ~/.local/share/luna/luna.db "$BACKUP_DIR/"
cp -r ~/.local/share/luna/logs "$BACKUP_DIR/logs"

echo "Backup complete: $BACKUP_DIR"
```

### Recovery

**Restore from backup:**
```bash
# Stop LUNA
sudo systemctl stop luna

# Restore
cp -r /backup/luna-20240101/config/* ~/.config/luna/
cp /backup/luna-20240101/luna.db ~/.local/share/luna/

# Restart
sudo systemctl start luna
```

## Scaling

### Multi-User Setup

**Each user gets their own instance:**
```bash
# User A
luna --config /home/userA/.config/luna/config.toml

# User B  
luna --config /home/userB/.config/luna/config.toml
```

### Remote Access

**SSH forwarding:**
```bash
ssh -L 9090:localhost:9090 user@luna-host

# Access metrics locally
curl http://localhost:9090/metrics
```

## Updates

### Update LUNA

**From source:**
```bash
cd luna
git pull origin main
cargo build --release
sudo systemctl restart luna
```

**Migration:**
```bash
# Run migrations (if any)
luna migrate

# Verify
luna doctor
```

## Support

### Logs

**View logs:**
```bash
# Systemd
journalctl -u luna -n 100

# File-based
tail -f ~/.local/share/luna/logs/luna.log
```

**Enable debug:**
```bash
RUST_LOG=luna=debug luna
```

### Diagnostics

**Run diagnostics:**
```bash
luna doctor --full > diagnostics.txt
```

**Include in bug reports:**
- LUNA version
- OS version
- Hardware specs
- Configuration (sanitized)
- Logs (last 100 lines)
- Diagnostics output
