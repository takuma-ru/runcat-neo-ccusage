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
CONVERSION_RATE=""
EXPLICIT_RATE=false
UNIT=""
PERIOD="monthly"
PERIOD_SINCE=""
PERIOD_UNTIL=""
CUSTOM_RANGE=false
INSTALL=false
UNINSTALL=false
STATUS=false

# --- Status Dashboard ---
show_status() {
  echo "RunCat Neo Custom Metrics Status"
  echo "================================="
  echo ""

  # 1. Check Tools
  echo "[System Tools]"
  if command -v ccusage &>/dev/null; then
    echo "  - ccusage : Installed ($(command -v ccusage))"
  else
    echo "  - ccusage : NOT FOUND"
  fi
  if command -v jq &>/dev/null; then
    echo "  - jq      : Installed ($(command -v jq))"
  else
    echo "  - jq      : NOT FOUND"
  fi
  echo ""

  # 2. Check Crontab Setup
  echo "[Crontab Configurations]"
  CRON_JOBS=$(crontab -l 2>/dev/null | grep -F "$SCRIPT_PATH" || true)
  if [ -z "$CRON_JOBS" ]; then
    echo "  No active cron jobs found for this script."
  else
    while IFS= read -r line; do
      # Extract agent name from cron line
      local agent_name="all"
      if [[ "$line" =~ --agent\ ([a-zA-Z0-9_-]+) ]]; then
        agent_name="${BASH_REMATCH[1]}"
      fi
      echo "  - $agent_name Tracker : Enabled"
      echo "    Line: $line"
    done <<< "$CRON_JOBS"
  fi
  echo ""

  # 3. Check JSON Files
  echo "[Metrics Outputs]"
  local files=("$SCRIPT_DIR"/runcat_*_metrics.json)
  local found_files=false
  for f in "${files[@]}"; do
    if [ -f "$f" ]; then
      found_files=true
      local filename=$(basename "$f")
      echo "  - $filename:"

      # Last modified time (macOS / BSD compatible)
      local mod_time=""
      if [ "$(uname)" = "Darwin" ]; then
        mod_time=$(stat -f "%Sm" -t "%Y-%m-%d %H:%M:%S" "$f" 2>/dev/null)
      else
        mod_time=$(date -r "$f" "+%Y-%m-%d %H:%M:%S" 2>/dev/null)
      fi

      echo "    Last Updated : $mod_time"

      # Content summary
      local title=$(jq -r '.title // "N/A"' "$f" 2>/dev/null)
      local symbol=$(jq -r '.symbol // "N/A"' "$f" 2>/dev/null)
      local bar_val=$(jq -r '.metricsBarValue // "N/A"' "$f" 2>/dev/null)

      echo "    Title        : $title"
      echo "    Symbol       : $symbol"
      echo "    Bar Value    : $bar_val"

      # Print metrics list
      echo "    Details      :"
      jq -c '.metrics[]?' "$f" 2>/dev/null | while read -r metric; do
        local m_title=$(echo "$metric" | jq -r '.title // ""' 2>/dev/null)
        local m_val=$(echo "$metric" | jq -r '.formattedValue // ""' 2>/dev/null)
        echo "      * $m_title : $m_val"
      done
      echo ""
    fi
  done

  if [ "$found_files" = false ]; then
    echo "  No generated metrics JSON files found."
  fi
}

# --- Number Formatting Helper ---
format_large_number() {
  local num="$1"
  if [[ ! "$num" =~ ^[0-9]+$ ]] || [ "$num" -eq 0 ]; then
    echo "0"
    return
  fi

  if [ "$num" -ge 1000000000 ]; then
    local val=$(echo "scale=2; $num / 1000000000" | bc -l)
    printf "%.2fB" "$val"
  elif [ "$num" -ge 1000000 ]; then
    local val=$(echo "scale=2; $num / 1000000" | bc -l)
    printf "%.2fM" "$val"
  elif [ "$num" -ge 1000 ]; then
    local val=$(echo "scale=2; $num / 1000" | bc -l)
    printf "%.2fK" "$val"
  else
    echo "$num"
  fi
}

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
  echo "  -p, --period <period>  Target retrieval period (monthly, weekly, or daily) [default: monthly]"
  echo "  --period-since <date>  Custom retrieval start date (YYYYMMDD or YYYY-MM-DD)"
  echo "  --period-until <date>  Custom retrieval end date (YYYYMMDD or YYYY-MM-DD)"
  echo "  -U, --unit <unit>      Unit/currency to display (e.g., USD, JPY, credits, or custom text)"
  echo "  -r, --rate <rate>      Conversion rate from USD (default: 25 for credits, 150 for JPY, 1 for others)"
  echo "  -i, --install          Install this configuration to crontab (runs every 10 minutes)"
  echo "  -u, --uninstall        Remove this configuration from crontab"
  echo "  status, --status       Show the status of active cron jobs and metrics files"
  echo "  -h, --help             Show this help message"
  echo ""
  echo "Examples:"
  echo "  # Track global monthly usage (All Agents) in USD"
  echo "  $(basename "$0") --agent all"
  echo ""
  echo "  # Track Claude weekly usage in JPY"
  echo "  \$(basename "\$0") --agent claude --period weekly --unit JPY"
  echo ""
  echo "  # Track custom billing cycle period (e.g., from 25th to 24th)"
  echo "  \$(basename "\$0") --agent claude --period-since 20260625 --period-until 20260724"
  echo ""
  echo "  # Track Codex monthly usage converted to credits (default behavior for codex)"
  echo "  \$(basename "\$0") --agent codex"
  echo ""
  echo "  # Install Claude monitoring to crontab"
  echo "  $(basename "$0") --agent claude --install"
  echo ""
  echo "  # Check system status and generated files"
  echo "  $(basename "$0") status"
}

# --- Parse Arguments ---
while [[ "$#" -gt 0 ]]; do
  case "$1" in
    -a|--agent) AGENT="$2"; shift ;;
    -t|--title) TITLE="$2"; shift ;;
    -s|--symbol) SYMBOL="$2"; shift ;;
    -p|--period) PERIOD="$2"; shift ;;
    --period-since) PERIOD_SINCE="$2"; CUSTOM_RANGE=true; shift ;;
    --period-until) PERIOD_UNTIL="$2"; CUSTOM_RANGE=true; shift ;;
    -r|--rate) CONVERSION_RATE="$2"; EXPLICIT_RATE=true; shift ;;
    -U|--unit) UNIT="$2"; shift ;;
    -i|--install) INSTALL=true ;;
    -u|--uninstall) UNINSTALL=true ;;
    status|--status) STATUS=true ;;
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

# Determine Default Unit based on agent if not specified
if [ -z "$UNIT" ]; then
  if [ "$AGENT" = "codex" ]; then
    UNIT="credits"
  else
    UNIT="USD"
  fi
fi

UNIT_LOWER=$(echo "$UNIT" | tr '[:upper:]' '[:lower:]')
UNIT_UPPER=$(echo "$UNIT" | tr '[:lower:]' '[:upper:]')

# Determine Default Rate if not explicitly provided
if [ "$EXPLICIT_RATE" = false ]; then
  case "$UNIT_LOWER" in
    usd) CONVERSION_RATE=1 ;;
    credits) CONVERSION_RATE=25 ;;
    *)
      # Attempt to fetch real-time rate for ANY currency code dynamically from public API (with 2s timeout)
      FETCHED_RATE=$(curl -s --connect-timeout 2 "https://open.er-api.com/v6/latest/USD" | jq --arg c "$UNIT_UPPER" '.rates[$c] // empty' 2>/dev/null)
      if [[ "$FETCHED_RATE" =~ ^[0-9]+(\.[0-9]+)?$ ]]; then
        CONVERSION_RATE="$FETCHED_RATE"
      else
        # Off-line fallbacks for common currencies
        case "$UNIT_LOWER" in
          jpy) CONVERSION_RATE=150 ;;
          eur) CONVERSION_RATE=0.9 ;;
          gbp) CONVERSION_RATE=0.8 ;;
          *) CONVERSION_RATE=1 ;;
        esac
      fi
      ;;
  esac
fi

# --- Determine Period Configuration ---
PERIOD_LOWER=$(echo "$PERIOD" | tr '[:upper:]' '[:lower:]')

if [ "$CUSTOM_RANGE" = true ]; then
  # Fill in defaults if only one of them is provided
  if [ -z "$PERIOD_SINCE" ]; then
    PERIOD_SINCE=$(date -v-1m +%Y%m%d)
  fi
  if [ -z "$PERIOD_UNTIL" ]; then
    PERIOD_UNTIL=$(date +%Y%m%d)
  fi

  # Format for display (YYYY/MM/DD)
  START_DATE=$(echo "$PERIOD_SINCE" | tr -d '-' | sed 's/\(....\)\(..\)\(..\)/\1\/\2\/\3/')
  END_DATE=$(echo "$PERIOD_UNTIL" | tr -d '-' | sed 's/\(....\)\(..\)\(..\)/\1\/\2\/\3/')
  PERIOD_LABEL="Custom"
else
  case "$PERIOD_LOWER" in
    daily)
      CURRENT_DATE=$(date +%Y-%m-%d)
      JSON_ARRAY_KEY="daily"
      DATE_FIELD_QUERY=".date // .period"
      PERIOD_LABEL="Daily"
      START_DATE=$(date +%Y/%m/%d)
      END_DATE=$(date +%Y/%m/%d)
      ;;
    weekly)
      CURRENT_DATE=$(date -v-sun +%Y-%m-%d)
      JSON_ARRAY_KEY="weekly"
      DATE_FIELD_QUERY=".week // .period"
      PERIOD_LABEL="Weekly"
      START_DATE=$(date -v-sun +%Y/%m/%d)
      END_DATE=$(date -v+sat +%Y/%m/%d)
      ;;
    monthly|*)
      PERIOD="monthly"
      CURRENT_DATE=$(date +%Y-%m)
      JSON_ARRAY_KEY="monthly"
      DATE_FIELD_QUERY=".month // .period"
      PERIOD_LABEL="Monthly"
      START_DATE=$(date -v1d +%Y/%m/%d)
      END_DATE=$(date -v+1m -v1d -v-1d +%Y/%m/%d)
      ;;
  esac
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
EXEC_CMD="$EXEC_CMD --unit \"$UNIT\""
if [ "$EXPLICIT_RATE" = true ]; then
  EXEC_CMD="$EXEC_CMD --rate $CONVERSION_RATE"
fi
if [ "$CUSTOM_RANGE" = true ]; then
  EXEC_CMD="$EXEC_CMD --period-since \"$PERIOD_SINCE\" --period-until \"$PERIOD_UNTIL\""
else
  EXEC_CMD="$EXEC_CMD --period \"$PERIOD\""
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

if [ "$STATUS" = true ]; then
  show_status
  exit 0
fi

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
if [ "$CUSTOM_RANGE" = true ]; then
  if [ "$AGENT" = "all" ]; then
    USAGE_JSON=$(ccusage --since "$PERIOD_SINCE" --until "$PERIOD_UNTIL" --json)
  else
    USAGE_JSON=$(ccusage "$AGENT" --since "$PERIOD_SINCE" --until "$PERIOD_UNTIL" --json)
  fi
else
  if [ "$AGENT" = "all" ]; then
    USAGE_JSON=$(ccusage "$PERIOD" --json)
  else
    USAGE_JSON=$(ccusage "$AGENT" "$PERIOD" --json)
  fi
fi

if [ -z "$USAGE_JSON" ] || ! jq -e . >/dev/null 2>&1 <<<"$USAGE_JSON"; then
  COST_USD=0
  TOTAL_TOKENS=0
else
  if [ "$CUSTOM_RANGE" = true ]; then
    # Custom range directly utilizes the aggregated "totals" object
    COST_USD=$(echo "$USAGE_JSON" | jq ".totals?.totalCost // .totals?.costUSD // 0" 2>/dev/null)
    TOTAL_TOKENS=$(echo "$USAGE_JSON" | jq ".totals?.totalTokens // 0" 2>/dev/null)
  else
    LAST_DATE=$(echo "$USAGE_JSON" | jq -r ".${JSON_ARRAY_KEY}[-1]? | ${DATE_FIELD_QUERY} // \"\"" 2>/dev/null)

    # Calendar/Period alignment check
    if [ "$LAST_DATE" = "$CURRENT_DATE" ]; then
      COST_USD=$(echo "$USAGE_JSON" | jq ".${JSON_ARRAY_KEY}[-1]?.totalCost // .${JSON_ARRAY_KEY}[-1]?.costUSD // 0" 2>/dev/null)
      TOTAL_TOKENS=$(echo "$USAGE_JSON" | jq ".${JSON_ARRAY_KEY}[-1]?.totalTokens // 0" 2>/dev/null)
    else
      COST_USD=0
      TOTAL_TOKENS=0
    fi
  fi
fi

# Validate output numeric formats
if [[ ! "$COST_USD" =~ ^[0-9]+(\.[0-9]+)?$ ]]; then
  COST_USD=0
fi
if [[ ! "$TOTAL_TOKENS" =~ ^[0-9]+$ ]]; then
  TOTAL_TOKENS=0
fi

# Determine currency prefix symbol and suffix label based on unit
PREFIX_SYMBOL=""
SUFFIX_LABEL=""
CREDIT_TITLE="Cost"

case "$UNIT_LOWER" in
  usd|cad|aud|sgd|nzd|hkd|mxn|cop)
    PREFIX_SYMBOL="$"
    ;;
  jpy|cny)
    PREFIX_SYMBOL="¥"
    ;;
  eur)
    PREFIX_SYMBOL="€"
    ;;
  gbp)
    PREFIX_SYMBOL="£"
    ;;
  credits)
    SUFFIX_LABEL=" cʀ"
    ;;
  *)
    SUFFIX_LABEL=" ${UNIT}"
    ;;
esac

# Calculate converted value
CALCULATED_VALUE=$(echo "$COST_USD * $CONVERSION_RATE" | bc -l)

# Format decimal places: standard currencies except JPY/CNY/credits get 2 decimals
case "$UNIT_LOWER" in
  jpy|cny|credits)
    FORMATTED_VALUE=$(printf "%.0f" "$CALCULATED_VALUE" 2>/dev/null || printf "0")
    ;;
  *)
    FORMATTED_VALUE=$(printf "%.2f" "$CALCULATED_VALUE" 2>/dev/null || printf "0.00")
    ;;
esac

METRICS_BAR_VALUE="${PREFIX_SYMBOL}${FORMATTED_VALUE}${SUFFIX_LABEL}"
JSON_FORMATTED_VALUE="${PREFIX_SYMBOL}${FORMATTED_VALUE}${SUFFIX_LABEL}"

FORMATTED_TOKENS=$(format_large_number "$TOTAL_TOKENS")
TIMESTAMP=$(date -u +%Y-%m-%dT%H:%M:%SZ)

# Export variables for jq
export RUNCAT_TITLE="$TITLE"
export RUNCAT_SYMBOL="$SYMBOL"
export RUNCAT_METRICS_BAR_VALUE="$METRICS_BAR_VALUE"
export RUNCAT_LAST_UPDATED="$TIMESTAMP"
export RUNCAT_CREDIT_TITLE="$CREDIT_TITLE"
export RUNCAT_CREDIT_VAL="$JSON_FORMATTED_VALUE"
export RUNCAT_TOKENS_VAL="$FORMATTED_TOKENS"
export RUNCAT_PERIOD_TITLE="Period"
export RUNCAT_PERIOD_VAL="${START_DATE} - ${END_DATE}"

# Generate RunCat Neo Custom Metrics JSON
jq -n '
{
  "title": env.RUNCAT_TITLE,
  "symbol": env.RUNCAT_SYMBOL,
  "metricsBarValue": env.RUNCAT_METRICS_BAR_VALUE,
  "lastUpdatedDate": env.RUNCAT_LAST_UPDATED,
  "metrics": [
    {
      "title": "Tokens",
      "formattedValue": env.RUNCAT_TOKENS_VAL
    },
    {
      "title": env.RUNCAT_CREDIT_TITLE,
      "formattedValue": env.RUNCAT_CREDIT_VAL
    },
    {
      "title": env.RUNCAT_PERIOD_TITLE,
      "formattedValue": env.RUNCAT_PERIOD_VAL
    }
  ]
}
' > "$TEMP_FILE"

# Atomically replace JSON
mv "$TEMP_FILE" "$OUTPUT_FILE"

echo "Successfully updated $TITLE $PERIOD metrics at $OUTPUT_FILE"
