# runcat-neo-ccusage

runcat-neo-ccusage is a shell script that formats monthly token and cost metrics from [ccusage](https://github.com/mscouter/ccusage) into the custom JSON schema required by [RunCat Neo](https://github.com/kyome/RunCat-Neo).

## Prerequisites

- macOS
- RunCat Neo (with Metrics Bar enabled)
- ccusage
- jq

## Installation

Save `runcat-neo-ccusage.sh` locally and make it executable:

```bash
chmod +x runcat-neo-ccusage.sh
```

## Usage

Run the script manually to generate the metrics JSON file:

```bash
# Track all agents (USD)
./runcat-neo-ccusage.sh --agent all

# Track Claude (USD)
./runcat-neo-ccusage.sh --agent claude

# Track Codex (Credits)
./runcat-neo-ccusage.sh --agent codex

# Check system status, active cron jobs, and generated metrics
./runcat-neo-ccusage.sh status
```

### Automation (crontab)

You can register the script to your crontab using the `--install` flag:

```bash
./runcat-neo-ccusage.sh --agent claude --install
```

To remove the cron job:

```bash
./runcat-neo-ccusage.sh --agent claude --uninstall
```

## RunCat Neo Configuration

1. Open RunCat Neo Settings.
2. Navigate to **Metrics** > **Custom Metrics** and click **Add Custom Metrics Source**.
3. Select the generated JSON file (e.g., `~/.config/run-cat-neo/runcat_claude_metrics.json`).
4. Enable the **Metrics Bar** and toggle the custom metric source to On.

## Options

```text
  -a, --agent <name>    Agent to track (claude, codex, gemini, copilot, or all) [default: all]
  -t, --title <title>    Custom card title in RunCat Neo
  -s, --symbol <symbol>  Custom SF Symbol identifier (macOS)
  -r, --rate <rate>      Credit conversion rate (1 USD = X Credits) [default: 25]
  -c, --credits          Display metrics as credits instead of USD
  -i, --install          Install configuration to crontab (runs every 10 minutes)
  -u, --uninstall        Remove configuration from crontab
  -h, --help             Show this help message
```

## License

MPL 2.0
