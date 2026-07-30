#[derive(Debug, Clone, PartialEq)]
pub struct CurrencyConfig {
    pub prefix_symbol: String,
    pub suffix_label: String,
    pub metrics_bar_value: String,
    pub json_formatted_value: String,
    pub credit_title: String,
}

pub fn get_prefix_and_suffix(unit: &str) -> (String, String) {
    let unit_lower = unit.to_lowercase();
    let mut prefix = "".to_string();
    let mut suffix = "".to_string();

    match unit_lower.as_str() {
        "usd" | "cad" | "aud" | "sgd" | "nzd" | "hkd" | "mxn" | "cop" => {
            prefix = "$".to_string();
        }
        "jpy" | "cny" => {
            prefix = "¥".to_string();
        }
        "eur" => {
            prefix = "€".to_string();
        }
        "gbp" => {
            prefix = "£".to_string();
        }
        "credits" => {
            suffix = " cʀ".to_string();
        }
        _ => {
            suffix = format!(" {}", unit);
        }
    }

    (prefix, suffix)
}

pub fn format_currency(cost_usd: f64, rate: f64, unit: &str) -> CurrencyConfig {
    let (prefix_symbol, suffix_label) = get_prefix_and_suffix(unit);
    let calculated_value = cost_usd * rate;

    let unit_lower = unit.to_lowercase();
    let formatted_value = match unit_lower.as_str() {
        "jpy" | "cny" | "credits" => {
            format!("{:.0}", calculated_value.round())
        }
        _ => {
            format!("{:.2}", calculated_value)
        }
    };

    let metrics_bar_value = format!("{}{}{}", prefix_symbol, formatted_value, suffix_label);
    let json_formatted_value = metrics_bar_value.clone();

    CurrencyConfig {
        prefix_symbol,
        suffix_label,
        metrics_bar_value,
        json_formatted_value,
        credit_title: "Cost".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_usd_formatting() {
        let config = format_currency(95.16, 1.0, "USD");
        assert_eq!(config.prefix_symbol, "$");
        assert_eq!(config.suffix_label, "");
        assert_eq!(config.metrics_bar_value, "$95.16");
        assert_eq!(config.json_formatted_value, "$95.16");
        assert_eq!(config.credit_title, "Cost");
    }

    #[test]
    fn test_jpy_formatting() {
        let config = format_currency(95.16, 150.0, "JPY");
        assert_eq!(config.prefix_symbol, "¥");
        assert_eq!(config.suffix_label, "");
        assert_eq!(config.metrics_bar_value, "¥14274");
        assert_eq!(config.json_formatted_value, "¥14274");
        assert_eq!(config.credit_title, "Cost");
    }

    #[test]
    fn test_credits_formatting() {
        let config = format_currency(95.16, 25.0, "credits");
        assert_eq!(config.prefix_symbol, "");
        assert_eq!(config.suffix_label, " cʀ");
        assert_eq!(config.metrics_bar_value, "2379 cʀ");
        assert_eq!(config.json_formatted_value, "2379 cʀ");
        assert_eq!(config.credit_title, "Cost");
    }

    #[test]
    fn test_eur_formatting() {
        let config = format_currency(95.16, 0.91, "EUR");
        assert_eq!(config.prefix_symbol, "€");
        assert_eq!(config.suffix_label, "");
        assert_eq!(config.metrics_bar_value, "€86.60");
        assert_eq!(config.json_formatted_value, "€86.60");
    }

    #[test]
    fn test_gbp_formatting() {
        let config = format_currency(95.16, 0.78, "GBP");
        assert_eq!(config.prefix_symbol, "£");
        assert_eq!(config.suffix_label, "");
        assert_eq!(config.metrics_bar_value, "£74.22");
        assert_eq!(config.json_formatted_value, "£74.22");
    }

    #[test]
    fn test_custom_formatting() {
        let config = format_currency(95.16, 1.0, "pts");
        assert_eq!(config.prefix_symbol, "");
        assert_eq!(config.suffix_label, " pts");
        assert_eq!(config.metrics_bar_value, "95.16 pts");
        assert_eq!(config.json_formatted_value, "95.16 pts");
    }
}
