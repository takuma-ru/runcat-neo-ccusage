# runcat-neo-ccusage

runcat-neo-ccusage is a high-performance, single-binary Rust utility that formats token and cost metrics from [ccusage](https://github.com/mscouter/ccusage) into the custom JSON schema required by [RunCat Neo](https://github.com/kyome/RunCat-Neo).

## Prerequisites

- macOS
- RunCat Neo (with Metrics Bar enabled)
- ccusage
- Rust toolchain / Cargo (only for building from source)

*Note: Unlike the shell script version, the compiled Rust binary has **zero dependency on jq, bc, or custom date commands**, ensuring complete execution safety under background cron environments.*

## Installation & Compilation

Clone this repository and compile the binary from source:

```bash
cargo build --release
cp target/release/runcat-neo-ccusage .
```

## Usage

Run the compiled executable manually to generate the metrics JSON file:

```bash
# Track all agents (USD)
./runcat-neo-ccusage --agent all

# Track Claude (USD)
./runcat-neo-ccusage --agent claude

# Track Codex (Credits)
./runcat-neo-ccusage --agent codex

# Check system status, active cron jobs, and generated metrics
./runcat-neo-ccusage status
```

### Automation (crontab)

You can register the binary directly to your crontab using the `--install` flag:

```bash
./runcat-neo-ccusage --agent claude --install
```

To remove the cron job:

```bash
./runcat-neo-ccusage --agent claude --uninstall
```

## RunCat Neo Configuration

1. Open RunCat Neo Settings.
2. Navigate to **Metrics** > **Custom Metrics** and click **Add Custom Metrics Source**.
3. Select the generated JSON file inside this repository directory (e.g., `~/dev/runcat-neo-ccusage/runcat_claude_metrics.json`).
4. Enable the **Metrics Bar** and toggle the custom metric source to On.

## Options

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

## License

MPL 2.0
