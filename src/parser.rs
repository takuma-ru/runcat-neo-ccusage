use serde_json::Value;

pub fn parse_json(
    json_str: &str,
    is_custom: bool,
    json_array_key: &str,
    date_field_query: &str,
    current_date_str: &str,
) -> Result<(f64, u64), String> {
    let parsed: Value =
        serde_json::from_str(json_str).map_err(|e| format!("Failed to parse JSON: {}", e))?;

    if is_custom {
        if let Some(totals) = parsed.get("totals") {
            let cost = totals
                .get("totalCost")
                .or_else(|| totals.get("costUSD"))
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0);
            let tokens = totals
                .get("totalTokens")
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            return Ok((cost, tokens));
        }
        return Ok((0.0, 0));
    }

    if let Some(array) = parsed.get(json_array_key).and_then(|a| a.as_array()) {
        if let Some(last_entry) = array.last() {
            let last_date = last_entry
                .get(date_field_query)
                .or_else(|| last_entry.get("period"))
                .and_then(|v| v.as_str())
                .unwrap_or("");

            if last_date == current_date_str {
                let cost = last_entry
                    .get("totalCost")
                    .or_else(|| last_entry.get("costUSD"))
                    .and_then(|v| v.as_f64())
                    .unwrap_or(0.0);
                let tokens = last_entry
                    .get("totalTokens")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0);
                return Ok((cost, tokens));
            }
        }
    }

    Ok((0.0, 0))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_custom_range() {
        let json_str = r#"{
            "totals": {
                "totalCost": 46.8987,
                "totalTokens": 217796133
            }
        }"#;
        let (cost, tokens) = parse_json(json_str, true, "", "", "").unwrap();
        assert_eq!(cost, 46.8987);
        assert_eq!(tokens, 217796133);
    }

    #[test]
    fn test_parse_custom_range_fallback() {
        let json_str = r#"{
            "totals": {
                "costUSD": 51017.97,
                "totalTokens": 65304317582
            }
        }"#;
        let (cost, tokens) = parse_json(json_str, true, "", "", "").unwrap();
        assert_eq!(cost, 51017.97);
        assert_eq!(tokens, 65304317582);
    }

    #[test]
    fn test_parse_monthly_matched() {
        let json_str = r#"{
            "monthly": [
                {
                    "month": "2026-07",
                    "totalCost": 81.36,
                    "totalTokens": 244173061
                }
            ]
        }"#;
        let (cost, tokens) = parse_json(json_str, false, "monthly", "month", "2026-07").unwrap();
        assert_eq!(cost, 81.36);
        assert_eq!(tokens, 244173061);
    }

    #[test]
    fn test_parse_monthly_unmatched() {
        let json_str = r#"{
            "monthly": [
                {
                    "month": "2026-06",
                    "totalCost": 81.36,
                    "totalTokens": 244173061
                }
            ]
        }"#;
        let (cost, tokens) = parse_json(json_str, false, "monthly", "month", "2026-07").unwrap();
        assert_eq!(cost, 0.0);
        assert_eq!(tokens, 0);
    }

    #[test]
    fn test_parse_weekly_matched() {
        let json_str = r#"{
            "weekly": [
                {
                    "week": "2026-07-19",
                    "totalCost": 1.79,
                    "totalTokens": 4411865
                }
            ]
        }"#;
        let (cost, tokens) = parse_json(json_str, false, "weekly", "week", "2026-07-19").unwrap();
        assert_eq!(cost, 1.79);
        assert_eq!(tokens, 4411865);
    }
}
