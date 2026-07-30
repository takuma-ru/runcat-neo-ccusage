# runcat-neo-ccusage

Formats token and cost metrics from [ccusage](https://github.com/ccusage/ccusage) into [RunCat Neo](https://github.com/runcat-dev/RunCatNeo)'s custom JSON schema.

## Features

- **Zero Runtime Dependencies:** The compiled single-binary has zero dependency on `jq`, `bc`, or custom shell date utilities.
- **Multi-Currency & Conversion:** Supports `USD`, `JPY` (with live dynamic exchange rate fetching and offline fallback), `EUR`, `GBP`, `credits` (default for Codex), or any custom unit.
- **Flexible Ranges:** Track monthly, weekly, daily, or custom billing cycles with `--period-since` and `--period-until`.
- **Diagnostic Dashboard:** Run `./runcat-neo-ccusage status` to view active cron jobs, tool status, and generated JSON values.

## Prerequisites

To run this utility, you only need:
- macOS
- RunCat Neo (with Metrics Bar enabled)
- [ccusage](https://github.com/ccusage/ccusage) installed

## Installation

Download the pre-compiled universal binary (supporting both Apple Silicon M1/M2/M3 and Intel Macs natively):

```bash
curl -L https://github.com/takuma-ru/runcat-neo-ccusage/releases/latest/download/runcat-neo-ccusage-mac.tar.gz | tar -xz
```

## Usage & Examples

### Manual Execution
Generate metrics JSON manually for your agents:

```bash
# Track global monthly usage (All Agents) in USD
./runcat-neo-ccusage --agent all

# Track Claude weekly usage in JPY
./runcat-neo-ccusage --agent claude --period weekly --unit JPY

# Track Codex monthly usage converted to credits (default for Codex)
./runcat-neo-ccusage --agent codex

# Track a custom billing cycle period (e.g., from 25th of last month to 24th of this month)
./runcat-neo-ccusage --agent claude --period-since 20260625 --period-until 20260724
```

### Automation & Diagnostics
Install, uninstall, or view system status:

```bash
# Register background auto-update (runs every 10 minutes)
./runcat-neo-ccusage --agent claude --period weekly --unit JPY --install

# Remove background auto-update for an agent
./runcat-neo-ccusage --agent claude --uninstall

# Check status of active cron jobs and generated metrics
./runcat-neo-ccusage status
```

## Options Reference

```text
  -a, --agent <name>     Agent to track (claude, codex, gemini, copilot, or all) [default: all]
  -t, --title <title>     Custom card title in RunCat Neo
  -s, --symbol <symbol>   Custom SF Symbol identifier (macOS)
  -p, --period <period>   Target retrieval period (monthly, weekly, or daily) [default: monthly]
  --period-since <date>   Custom retrieval start date (YYYYMMDD or YYYY-MM-DD)
  --period-until <date>   Custom retrieval end date (YYYYMMDD or YYYY-MM-DD)
  -U, --unit <unit>       Unit/currency to display (USD, JPY, credits, or custom text)
  -r, --rate <rate>       Conversion rate from USD (default: 25 for credits, 150 for JPY, 1 for others)
  -i, --install           Install configuration to crontab (runs every 10 minutes)
  -u, --uninstall         Remove configuration from crontab
  -h, --help              Show this help message
```

## RunCat Neo Configuration

1. Open RunCat Neo **Settings** > **Metrics** > **Custom Metrics**.
2. Click **Add Custom Metrics Source** and select the generated JSON file inside this repository (e.g., `runcat_claude_metrics.json`).
3. Enable **Metrics Bar** and toggle the new source to On.

## License

MPL 2.0
