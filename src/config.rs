#[derive(Debug, Clone, PartialEq)]
pub struct Config {
    pub agent: String,
    pub title: String,
    pub symbol: String,
    pub period: String,
    pub period_since: Option<String>,
    pub period_until: Option<String>,
    pub conversion_rate: Option<f64>,
    pub unit: String,
    pub install: bool,
    pub uninstall: bool,
    pub status: bool,
}

pub fn parse_args(args: Vec<String>) -> Result<Config, String> {
    let mut agent = "all".to_string();
    let mut title = "".to_string();
    let mut symbol = "".to_string();
    let mut period = "monthly".to_string();
    let mut period_since: Option<String> = None;
    let mut period_until: Option<String> = None;
    let mut conversion_rate: Option<f64> = None;
    let mut unit = "".to_string();
    let mut install = false;
    let mut uninstall = false;
    let mut status = false;

    let mut i = 1;
    while i < args.len() {
        let arg = &args[i];
        match arg.as_str() {
            "-a" | "--agent" => {
                if i + 1 < args.len() {
                    agent = args[i + 1].clone();
                    i += 2;
                } else {
                    return Err("Missing value for --agent".to_string());
                }
            }
            "-t" | "--title" => {
                if i + 1 < args.len() {
                    title = args[i + 1].clone();
                    i += 2;
                } else {
                    return Err("Missing value for --title".to_string());
                }
            }
            "-s" | "--symbol" => {
                if i + 1 < args.len() {
                    symbol = args[i + 1].clone();
                    i += 2;
                } else {
                    return Err("Missing value for --symbol".to_string());
                }
            }
            "-p" | "--period" => {
                if i + 1 < args.len() {
                    period = args[i + 1].clone();
                    i += 2;
                } else {
                    return Err("Missing value for --period".to_string());
                }
            }
            "--period-since" => {
                if i + 1 < args.len() {
                    period_since = Some(args[i + 1].clone());
                    i += 2;
                } else {
                    return Err("Missing value for --period-since".to_string());
                }
            }
            "--period-until" => {
                if i + 1 < args.len() {
                    period_until = Some(args[i + 1].clone());
                    i += 2;
                } else {
                    return Err("Missing value for --period-until".to_string());
                }
            }
            "-r" | "--rate" => {
                if i + 1 < args.len() {
                    let rate_val: f64 = args[i + 1].parse()
                        .map_err(|_| "Invalid number for --rate".to_string())?;
                    conversion_rate = Some(rate_val);
                    i += 2;
                } else {
                    return Err("Missing value for --rate".to_string());
                }
            }
            "-U" | "--unit" => {
                if i + 1 < args.len() {
                    unit = args[i + 1].clone();
                    i += 2;
                } else {
                    return Err("Missing value for --unit".to_string());
                }
            }
            "-i" | "--install" => {
                install = true;
                i += 1;
            }
            "-u" | "--uninstall" => {
                uninstall = true;
                i += 1;
            }
            "status" | "--status" => {
                status = true;
                i += 1;
            }
            _ => {
                return Err(format!("Unknown option: {}", arg));
            }
        }
    }

    if title.is_empty() {
        title = match agent.as_str() {
            "all" => "AI Usage".to_string(),
            "claude" => "Claude".to_string(),
            "codex" => "Codex".to_string(),
            "gemini" => "Gemini".to_string(),
            "copilot" => "Copilot".to_string(),
            _ => {
                let mut chars = agent.chars();
                match chars.next() {
                    None => "".to_string(),
                    Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                }
            }
        };
    }

    if symbol.is_empty() {
        symbol = match agent.as_str() {
            "all" => "brain.headpoint.filled".to_string(),
            "claude" => "asterisk.circle".to_string(),
            "codex" => "seal".to_string(),
            "gemini" => "sparkles".to_string(),
            "copilot" => "cat".to_string(),
            _ => "chart.bar.horizontal.page.fill".to_string(),
        };
    }

    if unit.is_empty() {
        unit = if agent == "codex" {
            "credits".to_string()
        } else {
            "USD".to_string()
        };
    }

    if conversion_rate.is_none() {
        let unit_lower = unit.to_lowercase();
        conversion_rate = Some(match unit_lower.as_str() {
            "usd" => 1.0,
            "jpy" => 150.0,
            "credits" => 25.0,
            _ => 1.0,
        });
    }

    Ok(Config {
        agent,
        title,
        symbol,
        period,
        period_since,
        period_until,
        conversion_rate,
        unit,
        install,
        uninstall,
        status,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_defaults() {
        let args = vec!["binary_path".to_string()];
        let config = parse_args(args).unwrap();
        assert_eq!(config.agent, "all");
        assert_eq!(config.title, "AI Usage");
        assert_eq!(config.symbol, "brain.headpoint.filled");
        assert_eq!(config.period, "monthly");
        assert_eq!(config.unit, "USD");
        assert_eq!(config.conversion_rate, Some(1.0));
        assert_eq!(config.install, false);
        assert_eq!(config.uninstall, false);
        assert_eq!(config.status, false);
    }

    #[test]
    fn test_parse_agent_codex() {
        let args = vec![
            "binary_path".to_string(),
            "--agent".to_string(),
            "codex".to_string(),
        ];
        let config = parse_args(args).unwrap();
        assert_eq!(config.agent, "codex");
        assert_eq!(config.title, "Codex");
        assert_eq!(config.symbol, "seal");
        assert_eq!(config.unit, "credits");
        assert_eq!(config.conversion_rate, Some(25.0));
    }

    #[test]
    fn test_parse_custom_overrides() {
        let args = vec![
            "binary_path".to_string(),
            "--agent".to_string(),
            "claude".to_string(),
            "--title".to_string(),
            "My Claude".to_string(),
            "--symbol".to_string(),
            "sparkles".to_string(),
            "--period".to_string(),
            "weekly".to_string(),
            "--unit".to_string(),
            "JPY".to_string(),
            "--rate".to_string(),
            "155.0".to_string(),
        ];
        let config = parse_args(args).unwrap();
        assert_eq!(config.agent, "claude");
        assert_eq!(config.title, "My Claude");
        assert_eq!(config.symbol, "sparkles");
        assert_eq!(config.period, "weekly");
        assert_eq!(config.unit, "JPY");
        assert_eq!(config.conversion_rate, Some(155.0));
    }

    #[test]
    fn test_parse_custom_dates() {
        let args = vec![
            "binary_path".to_string(),
            "--period-since".to_string(),
            "20260720".to_string(),
            "--period-until".to_string(),
            "20260727".to_string(),
        ];
        let config = parse_args(args).unwrap();
        assert_eq!(config.period_since, Some("20260720".to_string()));
        assert_eq!(config.period_until, Some("20260727".to_string()));
    }

    #[test]
    fn test_parse_status() {
        let args = vec!["binary_path".to_string(), "status".to_string()];
        let config = parse_args(args).unwrap();
        assert_eq!(config.status, true);

        let args_long = vec!["binary_path".to_string(), "--status".to_string()];
        let config_long = parse_args(args_long).unwrap();
        assert_eq!(config_long.status, true);
    }

    #[test]
    fn test_parse_install_uninstall() {
        let args_inst = vec!["binary_path".to_string(), "--install".to_string()];
        let config_inst = parse_args(args_inst).unwrap();
        assert_eq!(config_inst.install, true);

        let args_uninst = vec!["binary_path".to_string(), "-u".to_string()];
        let config_uninst = parse_args(args_uninst).unwrap();
        assert_eq!(config_uninst.uninstall, true);
    }
}
