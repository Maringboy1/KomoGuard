#!/bin/bash
# KomoGuard - One-command setup script
# Installs: KomoGuard security monitor + optional terminal enhancements

set -e

RED='\033[1;31m'
GREEN='\033[1;32m'
YELLOW='\033[1;33m'
BLUE='\033[1;34m'
NC='\033[0m'

KOMOGUARD_DIR="$HOME/KomoGuard"

echo ""
echo "  ${BLUE}╔═══════════════════════════════════════════╗${NC}"
echo "  ${BLUE}║     KomoGuard Setup Wizard v1.0          ║${NC}"
echo "  ${BLUE}╚═══════════════════════════════════════════╝${NC}"
echo ""

# Step 1: Build binary
echo "  ${YELLOW}[1/5]${NC} Building KomoGuard binary..."
cd "$KOMOGUARD_DIR"

if ! command -v cargo &>/dev/null; then
    echo "  ${RED}[-] Rust not found. Install it first:${NC}"
    echo "      curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
    exit 1
fi

cargo build --release 2>&1 | sed 's/^/      /'
BINARY="$KOMOGUARD_DIR/target/release/komoguard"

if [ ! -f "$BINARY" ]; then
    echo "  ${RED}[-] Build failed${NC}"
    exit 1
fi
echo "  ${GREEN}[✓]${NC} Built: $BINARY ($(du -h "$BINARY" | cut -f1))"

# Step 2: Install binary to PATH
echo ""
echo "  ${YELLOW}[2/5]${NC} Installing to ~/.cargo/bin/..."
mkdir -p "$HOME/.cargo/bin"
cp "$BINARY" "$HOME/.cargo/bin/komoguard"
chmod +x "$HOME/.cargo/bin/komoguard"
echo "  ${GREEN}[✓]${NC} Installed to ~/.cargo/bin/komoguard"

# Add to PATH if not already
if [[ ":$PATH:" != *":$HOME/.cargo/bin:"* ]]; then
    if [ -f "$HOME/.zshrc" ]; then
        echo 'export PATH="$HOME/.cargo/bin:$PATH"' >> "$HOME/.zshrc"
    fi
    if [ -f "$HOME/.bashrc" ]; then
        echo 'export PATH="$HOME/.cargo/bin:$PATH"' >> "$HOME/.bashrc"
    fi
    export PATH="$HOME/.cargo/bin:$PATH"
    echo "  ${GREEN}[✓]${NC} ~/.cargo/bin added to PATH"
fi

# Step 3: Create config
echo ""
echo "  ${YELLOW}[3/5]${NC} Setting up configuration..."
mkdir -p "$HOME/.config/komoguard"
if [ ! -f "$HOME/.config/komoguard/config.json" ]; then
    cp "$KOMOGUARD_DIR/config.json" "$HOME/.config/komoguard/config.json"
    echo "  ${GREEN}[✓]${NC} Config created at ~/.config/komoguard/config.json"
else
    echo "  ${GREEN}[✓]${NC} Config already exists"
fi

# Step 4: Terminal enhancements (optional)
echo ""
echo "  ${YELLOW}[4/5]${NC} Terminal enhancements (optional)..."
echo ""
echo "  KomoGuard works with your current terminal."
echo "  For a faster, more advanced terminal, the setup can install:"
echo "    ${BLUE}1.${NC} Zsh plugins (autosuggestions + syntax highlighting)"
echo "    ${BLUE}2.${NC} Starship prompt (fast Rust-based prompt)"
echo "    ${BLUE}3.${NC} fzf (fuzzy file/command search)"
echo ""
read -p "  Install terminal enhancements? [y/N]: " ENHANCE

if [[ "$ENHANCE" =~ ^[Yy]$ ]]; then
    echo ""

    # Install znap (Zsh plugin manager)
    if [ ! -d "$HOME/.znap" ]; then
        echo "  Installing znap (Zsh plugin manager)..."
        git clone --depth 1 https://github.com/marlonrichert/zsh-snap.git "$HOME/.znap" 2>/dev/null || true
        echo "  ${GREEN}[✓]${NC} znap installed"
    fi

    # Install zsh-autosuggestions
    if [ ! -d "$HOME/.znap/plugins/zsh-autosuggestions" ]; then
        echo "  Installing zsh-autosuggestions..."
        git clone --depth 1 https://github.com/zsh-users/zsh-autosuggestions \
            "$HOME/.znap/plugins/zsh-autosuggestions" 2>/dev/null || true
        echo "  ${GREEN}[✓]${NC} zsh-autosuggestions installed"
    fi

    # Install zsh-syntax-highlighting
    if [ ! -d "$HOME/.znap/plugins/zsh-syntax-highlighting" ]; then
        echo "  Installing zsh-syntax-highlighting..."
        git clone --depth 1 https://github.com/zsh-users/zsh-syntax-highlighting \
            "$HOME/.znap/plugins/zsh-syntax-highlighting" 2>/dev/null || true
        echo "  ${GREEN}[✓]${NC} zsh-syntax-highlighting installed"
    fi

    # Install Starship
    if ! command -v starship &>/dev/null; then
        echo "  Installing Starship prompt..."
        curl -sS https://starship.rs/install.sh | sh -s -- -y 2>/dev/null || true
        echo "  ${GREEN}[✓]${NC} Starship installed"
    fi

    # Install fzf
    if ! command -v fzf &>/dev/null && [ ! -f "$HOME/.fzf/bin/fzf" ]; then
        echo "  Installing fzf..."
        git clone --depth 1 https://github.com/junegunn/fzf.git "$HOME/.fzf" 2>/dev/null || true
        "$HOME/.fzf/install" --all --no-bash --no-fish 2>/dev/null || true
        echo "  ${GREEN}[✓]${NC} fzf installed"
    fi

    # Configure .zshrc with enhancements and KomoGuard
    ZSHRC="$HOME/.zshrc"

    # Backup existing .zshrc
    if [ -f "$ZSHRC" ] && [ ! -f "${ZSHRC}.bak" ]; then
        cp "$ZSHRC" "${ZSHRC}.bak"
        echo "  ${GREEN}[✓]${NC} Backed up .zshrc to .zshrc.bak"
    fi

    # Write new .zshrc
    cat > "$ZSHRC" << 'ZSHRCEOF'
# Zsh configuration - KomoGuard enhanced

# Znap plugin manager
if [ -f "$HOME/.znap/znap.zsh" ]; then
    source "$HOME/.znap/znap.zsh"

    # Plugins (lazy-loaded)
    znap source zsh-users/zsh-autosuggestions
    znap source zsh-users/zsh-syntax-highlighting

    # Completion
    autoload -Uz compinit && compinit -C
    znap fzf

    # History substring search
    if [ -f "$HOME/.znap/plugins/zsh-history-substring-search" ]; then
        znap source zsh-users/zsh-history-substring-search
    fi
fi

# Options
setopt AUTO_CD
setopt EXTENDED_HISTORY
setopt HIST_IGNORE_DUPS
setopt HIST_IGNORE_SPACE
setopt SHARE_HISTORY
HISTSIZE=10000
SAVEHIST=10000

# FZF
[ -f "$HOME/.fzf.zsh" ] && source "$HOME/.fzf.zsh"

# Starship prompt
if command -v starship &>/dev/null; then
    eval "$(starship init zsh)"
else
    # Fallback prompt
    PROMPT='%F{green}%n@%m%f:%F{blue}%~%f$ '
fi

# Aliases
alias ls='ls --color=auto'
alias ll='ls -la'
alias la='ls -A'
alias update='sudo apt update && sudo apt upgrade'

# KomoGuard - auto-activation
if [ -f "$HOME/KomoGuard/activate.sh" ]; then
    source "$HOME/KomoGuard/activate.sh"
fi
ZSHRCEOF

    echo "  ${GREEN}[✓]${NC} .zshrc configured with enhancements and KomoGuard"

    # Starship config - minimal with KomoGuard indicator
    mkdir -p "$HOME/.config"
    cat > "$HOME/.config/starship.toml" << 'STAREOF'
format = """
[□](#9A348E)\
$username\
$hostname\
$directory\
$git_branch\
$git_status\
$python\
$node\
$cmd_duration\
$line_break\
$character"""

[character]
success_symbol = '[➜](green)'
error_symbol = '[➜](red) '

[username]
show_always = true
format = '[$user]($style)@'
style_user = 'dimmed green'

[hostname]
ssh_only = false
format = '[$hostname]($style) '
style = 'dimmed cyan'

[directory]
style = 'blue'
truncation_length = 3

[git_branch]
format = '[$branch]($style) '
style = 'purple'

[git_status]
format = '[$all_status$ahead_behind]($style) '

[cmd_duration]
format = '[$duration]($style) '
style = 'yellow'

[python]
format = '[$version]($style) '

[node]
format = '[$version]($style) '
STAREOF
    echo "  ${GREEN}[✓]${NC} Starship prompt configured"

else
    # Just add KomoGuard activation to .zshrc
    ZSHRC="$HOME/.zshrc"
    if ! grep -q "KomoGuard" "$ZSHRC" 2>/dev/null; then
        echo "" >> "$ZSHRC"
        echo "# KomoGuard - System Security Monitor" >> "$ZSHRC"
        echo "if [ -f \"\$HOME/KomoGuard/activate.sh\" ]; then" >> "$ZSHRC"
        echo "    source \"\$HOME/KomoGuard/activate.sh\"" >> "$ZSHRC"
        echo "fi" >> "$ZSHRC"
        echo "  ${GREEN}[✓]${NC} KomoGuard activation added to .zshrc"
    else
        echo "  ${GREEN}[✓]${NC} KomoGuard already in .zshrc"
    fi
fi

# Step 5: Start KomoGuard
echo ""
echo "  ${YELLOW}[5/5]${NC} Starting KomoGuard..."
komoguard start 2>/dev/null || "$HOME/.cargo/bin/komoguard" start 2>/dev/null || true
echo "  ${GREEN}[✓]${NC} KomoGuard started"

# Summary
echo ""
echo "  ${GREEN}╔═══════════════════════════════════════════╗${NC}"
echo "  ${GREEN}║         KomoGuard Setup Complete!         ║${NC}"
echo "  ${GREEN}╚═══════════════════════════════════════════╝${NC}"
echo ""
echo "  Commands:"
echo "    komoguard scan     - Run a one-time security scan"
echo "    komoguard start    - Start daemon (background monitoring)"
echo "    komoguard stop     - Stop daemon"
echo "    komoguard alerts   - View security alerts"
echo "    komoguard status   - Show daemon status and info"
echo "    komoguard export   - Export alerts to JSON file"
echo "    komoguard clear    - Clear alert log"
echo ""
echo "  ${YELLOW}Restart your terminal or run: source ~/.zshrc${NC}"
echo ""
