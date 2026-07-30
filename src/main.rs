pub mod config;
pub mod currency;
pub mod formatting;
pub mod parser;
pub mod period;

use chrono::Local;
use std::io::Write;
use std::process::exit;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let config = match config::parse_args(args) {
        Ok(cfg) => cfg,
        Err(e) => {
            eprintln!("Error parsing arguments: {}", e);
            exit(1);
        }
    };

    // 1. Get Executable Path and Centralized Output Directory
    let exe_path = match std::env::current_exe() {
        Ok(path) => path,
        Err(e) => {
            eprintln!("Error getting current executable path: {}", e);
            exit(1);
        }
    };
    let script_path = exe_path.to_string_lossy().to_string();

    let home_dir = std::env::var("HOME").unwrap_or_else(|_| "/Users/kanekotakuma".to_string());
    let output_dir = std::path::PathBuf::from(home_dir)
        .join(".config")
        .join("rn-ccusage");

    // Ensure output directory exists
    if let Err(e) = std::fs::create_dir_all(&output_dir) {
        eprintln!(
            "Error creating output directory '{}': {}",
            output_dir.display(),
            e
        );
        exit(1);
    }

    // 2. Action Routing
    if config.status {
        show_status(&script_path, &output_dir);
        exit(0);
    }

    if config.uninstall {
        uninstall_cron(&script_path, &config.agent);
        exit(0);
    }

    if config.install {
        install_cron(
            &script_path,
            &config,
            &config.period_since,
            &config.period_until,
        );
        exit(0);
    }

    // 3. Regular Execution - Metrics Processing
    let today = Local::now().date_naive();
    let period_config = match period::calculate_period(
        &config.period,
        &config.period_since,
        &config.period_until,
        today,
    ) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Error calculating period: {}", e);
            exit(1);
        }
    };

    // Determine exchange rate (fetch dynamically if JPY/currency without explicit rate)
    let mut rate = config.conversion_rate.unwrap_or(1.0);
    if config.conversion_rate.is_none() || !config.install {
        // If not explicitly overridden via CLI
        let unit_lower = config.unit.to_lowercase();
        if unit_lower != "usd" && unit_lower != "credits" {
            // Attempt to fetch dynamically
            if let Some(fetched_rate) = fetch_live_rate(&config.unit) {
                rate = fetched_rate;
            } else {
                // Keep default fallback
                rate = match unit_lower.as_str() {
                    "jpy" => 150.0,
                    "eur" => 0.9,
                    "gbp" => 0.8,
                    _ => 1.0,
                };
            }
        }
    }

    // Run ccusage
    let since = period_config
        .period_since_formatted
        .clone()
        .unwrap_or_default();
    let until = period_config
        .period_until_formatted
        .clone()
        .unwrap_or_default();

    let ccusage_args = if period_config.is_custom {
        if config.agent == "all" {
            vec!["--since", &since, "--until", &until, "--json"]
        } else {
            vec![
                &config.agent,
                "--since",
                &since,
                "--until",
                &until,
                "--json",
            ]
        }
    } else {
        if config.agent == "all" {
            vec![&config.period, "--json"]
        } else {
            vec![&config.agent, &config.period, "--json"]
        }
    };

    let ccusage_output = match execute_ccusage(&ccusage_args) {
        Ok(out) => out,
        Err(e) => {
            eprintln!("Error executing ccusage: {}", e);
            exit(1);
        }
    };

    // Parse JSON
    let (cost_usd, tokens) = match parser::parse_json(
        &ccusage_output,
        period_config.is_custom,
        &period_config.json_array_key,
        &period_config.date_field_query,
        &period_config.current_date_str,
    ) {
        Ok(val) => val,
        Err(e) => {
            eprintln!("Error parsing ccusage output: {}", e);
            exit(1);
        }
    };

    // Format final currency outputs
    let currency_config = currency::format_currency(cost_usd, rate, &config.unit);

    // Build Output JSON
    let output_json = serde_json::json!({
        "title": config.title,
        "symbol": config.symbol,
        "metricsBarValue": currency_config.metrics_bar_value,
        "lastUpdatedDate": chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
        "metrics": [
            {
                "title": "Tokens",
                "formattedValue": formatting::format_large_number(tokens)
            },
            {
                "title": currency_config.credit_title,
                "formattedValue": currency_config.json_formatted_value
            },
            {
                "title": "Period",
                "formattedValue": format!("{} - {}", period_config.start_date_str, period_config.end_date_str)
            }
        ]
    });

    // Write to runcat_<agent>_metrics.json
    let out_file_path = output_dir.join(format!("runcat_{}_metrics.json", config.agent));
    let temp_file_path = output_dir.join(format!("runcat_{}_metrics.json.tmp", config.agent));

    let json_str = match serde_json::to_string_pretty(&output_json) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Error generating output JSON: {}", e);
            exit(1);
        }
    };

    if let Err(e) = std::fs::write(&temp_file_path, json_str) {
        eprintln!("Failed to write temporary JSON: {}", e);
        exit(1);
    }

    if let Err(e) = std::fs::rename(&temp_file_path, &out_file_path) {
        eprintln!("Failed to atomically replace JSON file: {}", e);
        exit(1);
    }

    println!(
        "Successfully updated {} {} metrics at {}",
        config.title,
        config.period,
        out_file_path.display()
    );
}

fn fetch_live_rate(unit: &str) -> Option<f64> {
    let unit_upper = unit.to_uppercase();
    let output = std::process::Command::new("curl")
        .args([
            "-s",
            "--connect-timeout",
            "2",
            "https://open.er-api.com/v6/latest/USD",
        ])
        .output()
        .ok()?;

    if output.status.success() {
        let json_str = String::from_utf8_lossy(&output.stdout);
        let parsed: serde_json::Value = serde_json::from_str(&json_str).ok()?;
        let rates = parsed.get("rates")?;
        let rate = rates.get(&unit_upper)?.as_f64()?;
        return Some(rate);
    }
    None
}

fn execute_ccusage(args: &[&str]) -> Result<String, String> {
    let output = std::process::Command::new("ccusage")
        .args(args)
        .output()
        .map_err(|e| format!("Failed to execute 'ccusage' command: {}", e))?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).to_string())
    }
}

fn show_status(_script_path: &str, exe_dir: &std::path::Path) {
    println!("RunCat Neo Custom Metrics Status");
    println!("=================================");
    println!();

    println!("[System Tools]");
    let ccusage_ok = std::process::Command::new("which")
        .arg("ccusage")
        .output()
        .is_ok();
    let jq_ok = std::process::Command::new("which")
        .arg("jq")
        .output()
        .is_ok();
    println!(
        "  - ccusage : {}",
        if ccusage_ok { "Installed" } else { "NOT FOUND" }
    );
    println!(
        "  - jq      : {}",
        if jq_ok { "Installed" } else { "NOT FOUND" }
    );
    println!();

    println!("[Crontab Configurations]");
    let cron_out = std::process::Command::new("crontab").arg("-l").output();
    let mut found_jobs = false;
    if let Ok(out) = cron_out {
        if out.status.success() {
            let cron_str = String::from_utf8_lossy(&out.stdout);
            for line in cron_str.lines() {
                if line.contains("rn-ccusage") {
                    found_jobs = true;
                    let mut agent = "all";
                    if line.contains("--agent claude") {
                        agent = "claude";
                    } else if line.contains("--agent codex") {
                        agent = "codex";
                    } else if line.contains("--agent gemini") {
                        agent = "gemini";
                    }
                    println!("  - {} Tracker : Enabled", agent);
                    println!("    Line: {}", line);
                }
            }
        }
    }
    if !found_jobs {
        println!("  No active cron jobs found for this script.");
    }
    println!();

    println!("[Metrics Outputs]");
    let mut found_files = false;
    if let Ok(entries) = std::fs::read_dir(exe_dir) {
        for entry in entries.filter_map(|e| e.ok()) {
            let path = entry.path();
            if let Some(filename) = path.file_name().and_then(|f| f.to_str()) {
                if filename.starts_with("runcat_") && filename.ends_with("_metrics.json") {
                    found_files = true;
                    println!("  - {}:", filename);

                    if let Ok(metadata) = std::fs::metadata(&path) {
                        if let Ok(modified) = metadata.modified() {
                            let dt: chrono::DateTime<chrono::Local> = modified.into();
                            println!("    Last Updated : {}", dt.format("%Y-%m-%d %H:%M:%S"));
                        }
                    }

                    if let Ok(content) = std::fs::read_to_string(&path) {
                        if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&content) {
                            println!(
                                "    Title        : {}",
                                parsed
                                    .get("title")
                                    .and_then(|v| v.as_str())
                                    .unwrap_or("N/A")
                            );
                            println!(
                                "    Symbol       : {}",
                                parsed
                                    .get("symbol")
                                    .and_then(|v| v.as_str())
                                    .unwrap_or("N/A")
                            );
                            println!(
                                "    Bar Value    : {}",
                                parsed
                                    .get("metricsBarValue")
                                    .and_then(|v| v.as_str())
                                    .unwrap_or("N/A")
                            );
                            println!("    Details      :");
                            if let Some(metrics) = parsed.get("metrics").and_then(|m| m.as_array())
                            {
                                for m in metrics {
                                    let t = m.get("title").and_then(|v| v.as_str()).unwrap_or("");
                                    let val = m
                                        .get("formattedValue")
                                        .and_then(|v| v.as_str())
                                        .unwrap_or("");
                                    println!("      * {} : {}", t, val);
                                }
                            }
                        }
                    }
                    println!();
                }
            }
        }
    }
    if !found_files {
        println!("  No generated metrics JSON files found.");
    }
}

fn uninstall_cron(_script_path: &str, agent: &str) {
    let cron_out = std::process::Command::new("crontab").arg("-l").output();
    let mut cron_lines = Vec::new();
    let filter_str = format!("--agent {}", agent);

    if let Ok(out) = cron_out {
        if out.status.success() {
            let cron_str = String::from_utf8_lossy(&out.stdout);
            for line in cron_str.lines() {
                if line.contains("rn-ccusage") && line.contains(&filter_str) {
                    // Skip this line to uninstall it
                    continue;
                }
                if !line.trim().is_empty() {
                    cron_lines.push(line.to_string());
                }
            }
        }
    }

    let mut child = std::process::Command::new("crontab")
        .arg("-")
        .stdin(std::process::Stdio::piped())
        .spawn()
        .expect("Failed to spawn crontab");

    {
        let mut stdin = child.stdin.take().expect("Failed to open stdin");
        for line in &cron_lines {
            writeln!(stdin, "{}", line).unwrap();
        }
    }

    child.wait().unwrap();
    println!("Success: Uninstalled cron job for agent '{}'.", agent);
}

fn install_cron(
    script_path: &str,
    config: &config::Config,
    period_since: &Option<String>,
    period_until: &Option<String>,
) {
    uninstall_cron(script_path, &config.agent);

    let mut exec_cmd = format!("{} --agent {}", script_path, config.agent);

    let default_title = match config.agent.as_str() {
        "all" => "AI Usage",
        "claude" => "Claude",
        "codex" => "Codex",
        "gemini" => "Gemini",
        "copilot" => "Copilot",
        _ => "",
    };
    if !default_title.is_empty() && config.title != default_title {
        exec_cmd.push_str(&format!(" --title \"{}\"", config.title));
    }

    let default_symbol = match config.agent.as_str() {
        "all" => "brain.headpoint.filled",
        "claude" => "asterisk.circle",
        "codex" => "seal",
        "gemini" => "sparkles",
        "copilot" => "cat",
        _ => "chart.bar.horizontal.page.fill",
    };
    if config.symbol != default_symbol {
        exec_cmd.push_str(&format!(" --symbol \"{}\"", config.symbol));
    }

    exec_cmd.push_str(&format!(" --unit \"{}\"", config.unit));

    if let Some(rate) = config.conversion_rate {
        exec_cmd.push_str(&format!(" --rate {}", rate));
    }

    if let Some(since) = period_since {
        exec_cmd.push_str(&format!(" --period-since {}", since));
    }
    if let Some(until) = period_until {
        exec_cmd.push_str(&format!(" --period-until {}", until));
    }
    if period_since.is_none() && period_until.is_none() {
        exec_cmd.push_str(&format!(" --period \"{}\"", config.period));
    }

    let cron_line = format!("*/10 * * * * {} > /dev/null 2>&1", exec_cmd);

    let cron_out = std::process::Command::new("crontab").arg("-l").output();
    let mut cron_lines = Vec::new();
    if let Ok(out) = cron_out {
        if out.status.success() {
            let cron_str = String::from_utf8_lossy(&out.stdout);
            for line in cron_str.lines() {
                if !line.trim().is_empty() {
                    cron_lines.push(line.to_string());
                }
            }
        }
    }

    cron_lines.push(cron_line.clone());

    let mut child = std::process::Command::new("crontab")
        .arg("-")
        .stdin(std::process::Stdio::piped())
        .spawn()
        .expect("Failed to spawn crontab");

    {
        let mut stdin = child.stdin.take().expect("Failed to open stdin");
        for line in &cron_lines {
            writeln!(stdin, "{}", line).unwrap();
        }
    }

    child.wait().unwrap();
    println!("Success: Installed task to crontab:");
    println!("  {}", cron_line);
}
