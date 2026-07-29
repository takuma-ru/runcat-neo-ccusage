# RunCat Neo Custom Metrics for ccusage 🐾

This is a highly flexible, robust, and generic shell script that bridges **[ccusage](https://github.com/mscouter/ccusage)** (the AI Agent CLI token/cost usage tracker) and **[RunCat Neo](https://github.com/kyome/RunCat-Neo)** (the next-generation open-source macOS status bar monitor).

It queries monthly tokens and cost usage for your various AI coding agents (Claude, Codex, Gemini, Copilot, etc.), formats them to the exact JSON schema required by RunCat Neo, and automates updates using a macOS cron job.

---

## ✨ Features

- **Multi-Agent Support:** Track global usage (`all`), or target specific agents like `claude`, `codex`, `gemini`, or `copilot`.
- **Smart Defaults:** Automatically maps each agent to its corresponding title, macOS SF Symbol, and conversion type (USD or Credits).
- **Credits Conversion:** Automatically converts cost in USD to Credits (e.g., `1 USD = 25 Credits` for Codex).
- **Calendar-Month Alignment:** Robust date validation ensures costs automatically reset to zero at the beginning of a new month, preventing outdated data from lingering before your first prompt of the month.
- **Easy Installer:** Includes full self-contained options (`--install` / `--uninstall`) to easily manage multiple independent agent tasks inside your `crontab` without manual editing.

---

## 🛠 Prerequisites

Before starting, ensure you have the following installed on your Mac:

1. **[RunCat Neo](https://github.com/kyome/RunCat-Neo)** (Make sure the **Metrics Bar** feature is enabled).
2. **ccusage**: The command-line utility to monitor your AI CLI costs (`npm install -g ccusage` or similar).
3. **jq**: High-performance JSON parser (`brew install jq`).

---

## 🚀 Getting Started

### 1. Download & Prepare the Script

Save the `runcat-neo-ccusage.sh` script to your preferred location (e.g., `~/.config/run-cat-neo/`), then make it executable:

```bash
mkdir -p ~/.config/run-cat-neo
cd ~/.config/run-cat-neo
# Save runcat-neo-ccusage.sh here
chmod +x runcat-neo-ccusage.sh
```

### 2. Run / Test the Script

You can run the script manually to generate the metrics JSON files:

```bash
# Global AI Agent usage (USD)
./runcat-neo-ccusage.sh --agent all

# Claude usage (USD, custom sparkles symbol)
./runcat-neo-ccusage.sh --agent claude --symbol "sparkles"

# Codex usage (automatically converts to credits)
./runcat-neo-ccusage.sh --agent codex
```

This will generate files like `runcat_claude_metrics.json` inside your `~/.config/run-cat-neo/` directory.

### 3. Automate with Cron (Auto-Install)

Simply pass the `--install` / `-i` flag to register the script as a background cron job running every 10 minutes. You can install multiple agents completely independently!

```bash
# Install Claude tracking
./runcat-neo-ccusage.sh --agent claude --install

# Install Codex tracking as Credits
./runcat-neo-ccusage.sh --agent codex --install
```

To remove a background tracking task, run:
```bash
./runcat-neo-ccusage.sh --agent claude --uninstall
```

---

## 🎨 Setting Up RunCat Neo

1. Right-click the running cat in your menu bar and open **Settings (Preferences)**.
2. Go to **Metrics** > **Custom Metrics**.
3. Click **Add Custom Metrics Source**.
4. Choose the JSON file created by the script (e.g., `~/.config/run-cat-neo/runcat_claude_metrics.json`).
   * *Tip:* Since `.config` is a hidden directory, press `Cmd + Shift + .` in the file picker dialog to show hidden files, or press `Cmd + Shift + G` and paste the path: `~/.config/run-cat-neo`.
5. Enable **Metrics Bar** in the Settings.
6. Click on the Metrics Bar in your macOS menu bar, and toggle your new custom source to **On**.

---

## ⚙️ Options Reference

```text
Options:
  -a, --agent <name>    Agent to track (e.g., claude, codex, gemini, copilot, or all) [default: all]
  -t, --title <title>    Custom card title displayed in RunCat Neo
  -s, --symbol <symbol>  Custom SF Symbol identifier (macOS)
  -r, --rate <rate>      Credit conversion rate (1 USD = X Credits) [default: 25]
  -c, --credits          Display metrics as credits instead of USD
  -i, --install          Install this configuration to crontab (runs every 10 minutes)
  -u, --uninstall        Remove this configuration from crontab
  -h, --help             Show this help message
```

---

## 📄 License

MIT
