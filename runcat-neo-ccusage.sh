#!/bin/bash

# --- Path & Environment Setup ---
# Dynamically add standard paths and active manager paths so cron can locate ccusage, jq, and mise
export PATH="$HOME/.local/bin:/opt/homebrew/bin:/opt/homebrew/sbin:/usr/local/bin:/usr/bin:/bin:/usr/sbin:/sbin:$PATH"
if [ -d "$HOME/.local/share/mise/shims" ]; then
  export PATH="$HOME/.local/share/mise/shims:$PATH"
fi
export LC_NUMERIC="en_US.UTF-8"

# --- Defaults ---
AGENT="all"
TITLE=""
SYMBOL=""
CREDIT_RATE=25
USE_CREDITS=""
INSTALL=false
UNINSTALL=false

# --- Help Message ---
show_help() {
  echo "RunCat Neo Custom Metrics Updater for ccusage"
  echo "============================================="
  echo "Usage: $(basename "$0") [options]"
  echo ""
  echo "Options:"
  echo "  -a, --agent <name>    Agent to track (e.g., claude, codex, gemini, copilot, or all) [default: all]"
  echo "  -t, --title <title>    Custom card title displayed in RunCat Neo"
  echo "  -s, --symbol <symbol>  Custom SF Symbol identifier (macOS)"
  echo "  -r, --rate <rate>      Credit conversion rate (1 USD = X Credits) [default: 25]"
  echo "  -c, --credits          Display metrics as credits instead of USD"
  echo "  -i, --install          Install this configuration to crontab (runs every 10 minutes)"
  echo "  -u, --uninstall        Remove this configuration from crontab"
  echo "  -h, --help             Show this help message"
  echo ""
  echo "Examples:"
  echo "  # Track global monthly usage (All Agents) in USD"
  echo "  $(basename "$0") --agent all"
  echo ""
  echo "  # Track Claude monthly usage with a custom title and symbol"
  echo "  $(basename "$0") --agent claude --title \"Claude Code\" --symbol \"sparkles\""
  echo ""
  echo "  # Track Codex monthly usage converted to credits (default behavior for codex)"
  echo "  $(basename "$0") --agent codex"
  echo ""
  echo "  # Install Claude monitoring to crontab"
  echo "  $(basename "$0") --agent claude --install"
  echo ""
  echo "  # Remove Codex monitoring from crontab"
  echo "  $(basename "$0") --agent codex --uninstall"
}

# --- Parse Arguments ---
while [[ "$#" -gt 0 ]]; do
  case "$1" in
    -a|--agent) AGENT="$2"; shift ;;
    -t|--title) TITLE="$2"; shift ;;
    -s|--symbol) SYMBOL="$2"; shift ;;
    -r|--rate) CREDIT_RATE="$2"; shift ;;
    -c|--credits) USE_CREDITS="true" ;;
    -i|--install) INSTALL=true ;;
    -u|--uninstall) UNINSTALL=true ;;
    -h|--help) show_help; exit 0 ;;
    *) echo "Error: Unknown option $1" >&2; show_help; exit 1 ;;
  esac
  shift
done

# --- Determine Smart Defaults ---
if [ -z "$TITLE" ]; then
  case "$AGENT" in
    all) TITLE="AI Usage" ;;
    claude) TITLE="Claude" ;;
    codex) TITLE="Codex" ;;
    gemini) TITLE="Gemini" ;;
    copilot) TITLE="Copilot" ;;
    *) TITLE="$(echo "${AGENT:0:1}" | tr '[:lower:]' '[:upper:]')${AGENT:1}" ;;
  esac
fi

if [ -z "$SYMBOL" ]; then
  case "$AGENT" in
    all) SYMBOL="brain.headpoint.filled" ;;
    claude) SYMBOL="asterisk.circle" ;;
    codex) SYMBOL="seal" ;;
    gemini) SYMBOL="sparkles" ;;
    copilot) SYMBOL="cat" ;;
    *) SYMBOL="chart.bar.horizontal.page.fill" ;;
  esac
fi

# Automatically enable credits conversion for Codex by default unless explicitly specified
if [ -z "$USE_CREDITS" ]; then
  if [ "$AGENT" = "codex" ]; then
    USE_CREDITS="true"
  else
    USE_CREDITS="false"
  fi
fi

# --- File System Configuration ---
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
OUTPUT_DIR="$SCRIPT_DIR"
OUTPUT_FILE="$OUTPUT_DIR/runcat_${AGENT}_metrics.json"
TEMP_FILE="${OUTPUT_FILE}.tmp"

# --- Cron Self-Management ---
SCRIPT_PATH="$SCRIPT_DIR/$(basename "${BASH_SOURCE[0]}")"

# Reconstruct installation command arguments
EXEC_CMD="$SCRIPT_PATH --agent $AGENT"
if [ "$TITLE" != "AI Usage" ] && [ "$TITLE" != "Claude" ] && [ "$TITLE" != "Codex" ] && [ "$TITLE" != "Gemini" ] && [ "$TITLE" != "Copilot" ]; then
  EXEC_CMD="$EXEC_CMD --title \"$TITLE\""
fi
if [ "$SYMBOL" != "brain.headpoint.filled" ] && [ "$SYMBOL" != "asterisk.circle" ] && [ "$SYMBOL" != "seal" ] && [ "$SYMBOL" != "sparkles" ] && [ "$SYMBOL" != "cat" ]; then
  EXEC_CMD="$EXEC_CMD --symbol \"$SYMBOL\""
fi
if [ "$USE_CREDITS" = "true" ] && [ "$AGENT" != "codex" ]; then
  EXEC_CMD="$EXEC_CMD --credits"
fi
if [ "$USE_CREDITS" = "false" ] && [ "$AGENT" = "codex" ]; then
  # If credits turned off for Codex, make sure to pass it
  EXEC_CMD="$EXEC_CMD" # Not strictly required as we can specify in help
fi
if [ "$CREDIT_RATE" != "25" ]; then
  EXEC_CMD="$EXEC_CMD --rate $CREDIT_RATE"
fi

CRON_LINE="*/10 * * * * $EXEC_CMD > /dev/null 2>&1"

uninstall_cron() {
  if crontab -l 2>/dev/null | grep -F -q "$SCRIPT_PATH --agent $AGENT"; then
    crontab -l 2>/dev/null | grep -F -v "$SCRIPT_PATH --agent $AGENT" | crontab -
    echo "Success: Uninstalled $TITLE ($AGENT) from crontab."
  else
    echo "Info: No active cron job found for $TITLE ($AGENT)."
  fi
}

install_cron() {
  uninstall_cron
  (crontab -l 2>/dev/null; echo "$CRON_LINE") | crontab -
  echo "Success: Installed $TITLE ($AGENT) to crontab."
  echo "Cron job will run every 10 minutes:"
  echo "  $CRON_LINE"
}

if [ "$UNINSTALL" = true ]; then
  uninstall_cron
  exit 0
fi

if [ "$INSTALL" = true ]; then
  install_cron
  exit 0
fi

# --- Main Metrics Fetching & Processing ---
mkdir -p "$OUTPUT_DIR"

# Validate tools are installed
if ! command -v ccusage &> /dev/null; then
  echo "Error: 'ccusage' command not found. Please install ccusage first." >&2
  exit 1
fi
if ! command -v jq &> /dev/null; then
  echo "Error: 'jq' command not found. Please install jq first." >&2
  exit 1
fi

# Fetch from ccusage
if [ "$AGENT" = "all" ]; then
  USAGE_JSON=$(ccusage monthly --json)
else
  USAGE_JSON=$(ccusage "$AGENT" monthly --json)
fi

if [ -z "$USAGE_JSON" ] || ! jq -e . >/dev/null 2>&1 <<<"$USAGE_JSON"; then
  COST_USD=0
  TOTAL_TOKENS=0
else
  CURRENT_MONTH=$(date +%Y-%m)
  LAST_MONTH=$(echo "$USAGE_JSON" | jq -r '.monthly[-1]?.month // .monthly[-1]?.period // ""' 2>/dev/null)

  # Calendar-month alignment check
  if [ "$LAST_MONTH" = "$CURRENT_MONTH" ]; then
    COST_USD=$(echo "$USAGE_JSON" | jq '.monthly[-1]?.totalCost // .monthly[-1]?.costUSD // 0' 2>/dev/null)
    TOTAL_TOKENS=$(echo "$USAGE_JSON" | jq '.monthly[-1]?.totalTokens // 0' 2>/dev/null)
  else
    COST_USD=0
    TOTAL_TOKENS=0
  fi
fi

# Validate output numeric formats
if [[ ! "$COST_USD" =~ ^[0-9]+(\.[0-9]+)?$ ]]; then
  COST_USD=0
fi
if [[ ! "$TOTAL_TOKENS" =~ ^[0-9]+$ ]]; then
  TOTAL_TOKENS=0
fi

# Format values based on preference
if [ "$USE_CREDITS" = "true" ]; then
  TOTAL_CREDITS=$(echo "$COST_USD * $CREDIT_RATE" | bc -l)
  FORMATTED_VALUE="$(printf "%.0f" "$TOTAL_CREDITS" 2>/dev/null || printf "0")"
  METRICS_BAR_VALUE="${FORMATTED_VALUE} cʀ"
  CREDIT_TITLE="Credits"
  JSON_FORMATTED_VALUE="$FORMATTED_VALUE"
else
  FORMATTED_VALUE="$(printf "%.2f" "$COST_USD" 2>/dev/null || printf "0.00")"
  METRICS_BAR_VALUE="\$${FORMATTED_VALUE}"
  CREDIT_TITLE="Cost"
  JSON_FORMATTED_VALUE="\$${FORMATTED_VALUE}"
fi

FORMATTED_TOKENS=$(printf "%d" "$TOTAL_TOKENS")
TIMESTAMP=$(date -u +%Y-%m-%dT%H:%M:%SZ)
NEXT_RESET="$(date -v+1m -v1d +%Y/%m/%d)"

# Export variables for jq
export RUNCAT_TITLE="$TITLE"
export RUNCAT_SYMBOL="$SYMBOL"
export RUNCAT_METRICS_BAR_VALUE="$METRICS_BAR_VALUE"
export RUNCAT_LAST_UPDATED="$TIMESTAMP"
export RUNCAT_CREDIT_TITLE="$CREDIT_TITLE"
export RUNCAT_CREDIT_VAL="$JSON_FORMATTED_VALUE"
export RUNCAT_TOKENS_VAL="$FORMATTED_TOKENS"
export RUNCAT_NEXT_RESET="$NEXT_RESET"

# Generate RunCat Neo Custom Metrics JSON
jq -n '
{
  "title": env.RUNCAT_TITLE,
  "symbol": env.RUNCAT_SYMBOL,
  "metricsBarValue": env.RUNCAT_METRICS_BAR_VALUE,
  "lastUpdatedDate": env.RUNCAT_LAST_UPDATED,
  "metrics": [
    {
      "title": env.RUNCAT_CREDIT_TITLE,
      "formattedValue": env.RUNCAT_CREDIT_VAL
    },
    {
      "title": "Tokens",
      "formattedValue": env.RUNCAT_TOKENS_VAL
    },
    {
      "title": "Next reset",
      "formattedValue": env.RUNCAT_NEXT_RESET
    }
  ]
}
' > "$TEMP_FILE"

# Atomically replace JSON
mv "$TEMP_FILE" "$OUTPUT_FILE"

echo "Successfully updated $TITLE monthly metrics at $OUTPUT_FILE"
