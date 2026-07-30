# runcat-neo-ccusage

Formats token and cost metrics from [ccusage](https://github.com/mscouter/ccusage) into [RunCat Neo](https://github.com/kyome/RunCat-Neo)'s custom JSON schema.

## Prerequisites

To run this utility, you only need:
- macOS
- RunCat Neo (with Metrics Bar enabled)
- [ccusage](https://github.com/mscouter/ccusage) installed

## Installation

Download the pre-compiled universal binary (supporting both Apple Silicon and Intel Macs natively):

```bash
curl -L https://github.com/takuma-ru/runcat-neo-ccusage/releases/latest/download/runcat-neo-ccusage-mac.tar.gz | tar -xz
```

## Usage

```bash
# Generate metrics JSON
./runcat-neo-ccusage --agent claude

# Register background auto-update (runs every 10 minutes)
./runcat-neo-ccusage --agent claude --install

# Remove background auto-update
./runcat-neo-ccusage --agent claude --uninstall

# Check status of active cron jobs and generated metrics
./runcat-neo-ccusage status
```

*For all configuration options, run: `./runcat-neo-ccusage --help`*

## RunCat Neo Configuration

1. Open RunCat Neo **Settings** > **Metrics** > **Custom Metrics**.
2. Click **Add Custom Metrics Source** and select the generated JSON file (e.g., `runcat_claude_metrics.json`).
3. Enable **Metrics Bar** and toggle the new source to On.

## License

MPL 2.0
