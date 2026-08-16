#!/usr/bin/env bash
#
# One-time provisioning for the NIKHIL//OS backend on a free-tier EC2 box.
#
# Run as root on a fresh Ubuntu 24.04 LTS instance:
#   sudo bash scripts/provision.sh
#
# What it does:
#   1. Opens the firewall (22, 80, 443) for Caddy auto-HTTPS.
#   2. Installs Docker Engine + the compose plugin.
#   3. Creates a non-root `deploy` user with Docker access.
#   4. Adds 1 GB swap — free-tier boxes have 1 GB RAM and Postgres + the
#      release build can spike past it.
#   5. Installs a public key for SSH access (from $PUBLIC_KEY, or prompts).
#
# Idempotent: safe to re-run.

set -euo pipefail

DEPLOY_USER="${DEPLOY_USER:-deploy}"
SWAP_SIZE_MB="${SWAP_SIZE_MB:-1024}"

if [ "$(id -u)" -ne 0 ]; then
  echo "error: run as root (sudo bash scripts/provision.sh)" >&2
  exit 1
fi

log() { printf '\n\033[1;32m==> %s\033[0m\n' "$*"; }

log "1/5 — firewall (22, 80, 443)"
if command -v ufw >/dev/null 2>&1; then
  ufw allow OpenSSH
  ufw allow 80/tcp comment 'Caddy HTTP-01'
  ufw allow 443/tcp comment 'Caddy HTTPS'
  ufw --force enable
else
  echo "ufw not present — ensure your security group allows 22, 80, 443"
fi

log "2/5 — Docker Engine + compose plugin"
if ! command -v docker >/dev/null 2>&1; then
  curl -fsSL https://get.docker.com | sh
fi
systemctl enable --now docker
usermod -aG docker "$SUDO_USER" 2>/dev/null || true
docker compose version

log "3/5 — non-root deploy user"
if ! id "$DEPLOY_USER" >/dev/null 2>&1; then
  adduser --disabled-password --gecos "" "$DEPLOY_USER"
  usermod -aG docker "$DEPLOY_USER"
  mkdir -p "/home/$DEPLOY_USER/.ssh"
  chmod 700 "/home/$DEPLOY_USER/.ssh"
  chown -R "$DEPLOY_USER:$DEPLOY_USER" "/home/$DEPLOY_USER/.ssh"
fi

log "4/5 — swap ($SWAP_SIZE_MB MB)"
if [ ! -f /swapfile ]; then
  fallocate -l "${SWAP_SIZE_MB}M" /swapfile
  chmod 600 /swapfile
  mkswap /swapfile >/dev/null
  swapon /swapfile
  grep -q '^/swapfile ' /etc/fstab || echo '/swapfile none swap sw 0 0' >> /etc/fstab
  # Tune swappiness down a touch: swap is a safety net, not a hot path.
  sysctl -w vm.swappiness=10 >/dev/null
  grep -q 'vm.swappiness' /etc/sysctl.conf || echo 'vm.swappiness=10' >> /etc/sysctl.conf
  log "swap enabled: /swapfile"
else
  echo "swap already present — skipping"
fi

log "5/5 — SSH access for $DEPLOY_USER"
SSH_DIR="/home/$DEPLOY_USER/.ssh"
AUTH_KEYS="$SSH_DIR/authorized_keys"
if [ -n "${PUBLIC_KEY:-}" ]; then
  echo "$PUBLIC_KEY" >> "$AUTH_KEYS"
  chmod 600 "$AUTH_KEYS"
  chown "$DEPLOY_USER:$DEPLOY_USER" "$AUTH_KEYS"
  log "public key installed"
elif [ ! -s "$AUTH_KEYS" ]; then
  echo "No PUBLIC_KEY given and none installed. Add one manually:"
  echo "  # as $DEPLOY_USER:"
  echo "  echo '<your public key>' >> ~/.ssh/authorized_keys"
else
  echo "authorized_keys already populated — skipping"
fi

cat <<'EOF'

Done. Next steps:
  1. Add the deploy user's public key to the GitHub repo (Settings → Deploy keys,
     read access is sufficient) so the CI deploy workflow can `git fetch`.
  2. As the deploy user:
       git clone git@github.com:nikhilkumarpanigrahi/nikhil-os.git
       cd nikhil-os/backend
       cp .env.example .env      # fill every value
       docker compose up -d --build
  3. Point your domain's A record at this instance's public IP.
  4. curl https://<your-domain>/healthz   →  should return 200 {}
EOF
