use chrono::{Datelike, NaiveDate, Duration};

#[derive(Debug, Clone, PartialEq)]
pub struct PeriodConfig {
    pub current_date_str: String,
    pub json_array_key: String,
    pub date_field_query: String,
    pub start_date_str: String,
    pub end_date_str: String,
    pub period_label: String,
    pub is_custom: bool,
    pub period_since_formatted: Option<String>,
    pub period_until_formatted: Option<String>,
}

fn resolve_macos_date(arg: &str) -> Option<String> {
    let mut date_args = Vec::new();
    for token in arg.split_whitespace() {
        date_args.push(token.to_string());
    }
    date_args.push("+%Y%m%d".to_string());

    let output = std::process::Command::new("date")
        .args(&date_args)
        .output()
        .ok()?;

    if output.status.success() {
        let date_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if date_str.len() == 8 && date_str.chars().all(|c| c.is_ascii_digit()) {
            return Some(date_str);
        }
    }
    None
}

pub fn calculate_period(
    period: &str,
    since: &Option<String>,
    until: &Option<String>,
    today: NaiveDate,
) -> Result<PeriodConfig, String> {
    // 1. Resolve relative macOS date arguments first
    let resolved_since = since.as_ref().and_then(|s| {
        if s.starts_with("-v") {
            resolve_macos_date(s)
        } else {
            None
        }
    });

    let resolved_until = until.as_ref().and_then(|u| {
        if u.starts_with("-v") {
            resolve_macos_date(u)
        } else {
            None
        }
    });

    let effective_since = resolved_since.as_ref().or(since.as_ref().filter(|s| !s.starts_with("-v")));
    let effective_until = resolved_until.as_ref().or(until.as_ref().filter(|u| !u.starts_with("-v")));

    let since_opt = effective_since.cloned();
    let until_opt = effective_until.cloned();

    // 2. Check if since is a dynamic billing day (e.g., "25", "25th")
    let mut is_dynamic_day = false;
    let mut billing_day = 1;
    if let Some(ref s) = since_opt {
        let cleaned_s = s.trim().to_lowercase()
            .replace("th", "")
            .replace("st", "")
            .replace("nd", "")
            .replace("rd", "");
        if let Ok(day) = cleaned_s.parse::<u32>() {
            if day >= 1 && day <= 31 {
                is_dynamic_day = true;
                billing_day = day;
            }
        }
    }

    if is_dynamic_day {
        let (start_date, end_date) = if today.day() >= billing_day {
            let start = NaiveDate::from_ymd_opt(today.year(), today.month(), billing_day).unwrap();
            let next_month = if today.month() == 12 { 1 } else { today.month() + 1 };
            let next_year = if today.month() == 12 { today.year() + 1 } else { today.year() };
            let end = if billing_day == 1 {
                let next_month_first = NaiveDate::from_ymd_opt(next_year, next_month, 1).unwrap();
                next_month_first.pred_opt().unwrap()
            } else {
                NaiveDate::from_ymd_opt(next_year, next_month, billing_day - 1).unwrap()
            };
            (start, end)
        } else {
            let prev_month = if today.month() == 1 { 12 } else { today.month() - 1 };
            let prev_year = if today.month() == 1 { today.year() - 1 } else { today.year() };
            let start = NaiveDate::from_ymd_opt(prev_year, prev_month, billing_day).unwrap();
            let end = if billing_day == 1 {
                let current_month_first = NaiveDate::from_ymd_opt(today.year(), today.month(), 1).unwrap();
                current_month_first.pred_opt().unwrap()
            } else {
                NaiveDate::from_ymd_opt(today.year(), today.month(), billing_day - 1).unwrap()
            };
            (start, end)
        };

        return Ok(PeriodConfig {
            current_date_str: "".to_string(),
            json_array_key: "".to_string(),
            date_field_query: "".to_string(),
            start_date_str: start_date.format("%Y/%m/%d").to_string(),
            end_date_str: end_date.format("%Y/%m/%d").to_string(),
            period_label: "Custom".to_string(),
            is_custom: true,
            period_since_formatted: Some(start_date.format("%Y%m%d").to_string()),
            period_until_formatted: Some(end_date.format("%Y%m%d").to_string()),
        });
    }

    if since_opt.is_some() || until_opt.is_some() {
        let since_str = since_opt.clone().unwrap_or_else(|| {
            let prev_month = today - Duration::days(30);
            prev_month.format("%Y%m%d").to_string()
        });
        let until_str = until_opt.clone().unwrap_or_else(|| {
            today.format("%Y%m%d").to_string()
        });

        let start_cleaned = since_str.replace("-", "");
        let end_cleaned = until_str.replace("-", "");

        fn format_date(cleaned: &str) -> String {
            if cleaned.len() == 8 {
                format!("{}/{}/{}", &cleaned[0..4], &cleaned[4..6], &cleaned[6..8])
            } else {
                cleaned.to_string()
            }
        }

        let start_date_str = format_date(&start_cleaned);
        let end_date_str = format_date(&end_cleaned);

        return Ok(PeriodConfig {
            current_date_str: "".to_string(),
            json_array_key: "".to_string(),
            date_field_query: "".to_string(),
            start_date_str,
            end_date_str,
            period_label: "Custom".to_string(),
            is_custom: true,
            period_since_formatted: Some(start_cleaned),
            period_until_formatted: Some(end_cleaned),
        });
    }

    let period_lower = period.to_lowercase();
    match period_lower.as_str() {
        "daily" => {
            let date_str = today.format("%Y-%m-%d").to_string();
            let display_str = today.format("%Y/%m/%d").to_string();
            Ok(PeriodConfig {
                current_date_str: date_str,
                json_array_key: "daily".to_string(),
                date_field_query: "date".to_string(),
                start_date_str: display_str.clone(),
                end_date_str: display_str,
                period_label: "Daily".to_string(),
                is_custom: false,
                period_since_formatted: None,
                period_until_formatted: None,
            })
        }
        "weekly" => {
            let days_since_sun = today.weekday().num_days_from_sunday();
            let start = today - Duration::days(days_since_sun as i64);
            let end = today + Duration::days((6 - days_since_sun) as i64);

            Ok(PeriodConfig {
                current_date_str: start.format("%Y-%m-%d").to_string(),
                json_array_key: "weekly".to_string(),
                date_field_query: "week".to_string(),
                start_date_str: start.format("%Y/%m/%d").to_string(),
                end_date_str: end.format("%Y/%m/%d").to_string(),
                period_label: "Weekly".to_string(),
                is_custom: false,
                period_since_formatted: None,
                period_until_formatted: None,
            })
        }
        _ => {
            let start = NaiveDate::from_ymd_opt(today.year(), today.month(), 1).unwrap();
            let next_month = if today.month() == 12 { 1 } else { today.month() + 1 };
            let next_year = if today.month() == 12 { today.year() + 1 } else { today.year() };
            let next_month_first = NaiveDate::from_ymd_opt(next_year, next_month, 1).unwrap();
            let end = next_month_first.pred_opt().unwrap();

            Ok(PeriodConfig {
                current_date_str: today.format("%Y-%m").to_string(),
                json_array_key: "monthly".to_string(),
                date_field_query: "month".to_string(),
                start_date_str: start.format("%Y/%m/%d").to_string(),
                end_date_str: end.format("%Y/%m/%d").to_string(),
                period_label: "Monthly".to_string(),
                is_custom: false,
                period_since_formatted: None,
                period_until_formatted: None,
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_monthly() {
        let today = NaiveDate::from_ymd_opt(2026, 7, 29).unwrap();
        let config = calculate_period("monthly", &None, &None, today).unwrap();
        assert_eq!(config.period_label, "Monthly");
        assert_eq!(config.json_array_key, "monthly");
        assert_eq!(config.date_field_query, "month");
        assert_eq!(config.current_date_str, "2026-07");
        assert_eq!(config.start_date_str, "2026/07/01");
        assert_eq!(config.end_date_str, "2026/07/31");
        assert_eq!(config.is_custom, false);
    }

    #[test]
    fn test_calculate_weekly() {
        let today = NaiveDate::from_ymd_opt(2026, 7, 29).unwrap();
        let config = calculate_period("weekly", &None, &None, today).unwrap();
        assert_eq!(config.period_label, "Weekly");
        assert_eq!(config.json_array_key, "weekly");
        assert_eq!(config.date_field_query, "week");
        assert_eq!(config.current_date_str, "2026-07-26");
        assert_eq!(config.start_date_str, "2026/07/26");
        assert_eq!(config.end_date_str, "2026/08/01");
        assert_eq!(config.is_custom, false);
    }

    #[test]
    fn test_calculate_daily() {
        let today = NaiveDate::from_ymd_opt(2026, 7, 29).unwrap();
        let config = calculate_period("daily", &None, &None, today).unwrap();
        assert_eq!(config.period_label, "Daily");
        assert_eq!(config.json_array_key, "daily");
        assert_eq!(config.date_field_query, "date");
        assert_eq!(config.current_date_str, "2026-07-29");
        assert_eq!(config.start_date_str, "2026/07/29");
        assert_eq!(config.end_date_str, "2026/07/29");
        assert_eq!(config.is_custom, false);
    }

    #[test]
    fn test_calculate_custom() {
        let today = NaiveDate::from_ymd_opt(2026, 7, 29).unwrap();
        let since = Some("20260720".to_string());
        let until = Some("20260727".to_string());
        let config = calculate_period("monthly", &since, &until, today).unwrap();
        assert_eq!(config.period_label, "Custom");
        assert_eq!(config.start_date_str, "2026/07/20");
        assert_eq!(config.end_date_str, "2026/07/27");
        assert_eq!(config.is_custom, true);
        assert_eq!(config.period_since_formatted, Some("20260720".to_string()));
        assert_eq!(config.period_until_formatted, Some("20260727".to_string()));
    }

    #[test]
    fn test_calculate_custom_dashed() {
        let today = NaiveDate::from_ymd_opt(2026, 7, 29).unwrap();
        let since = Some("2026-07-20".to_string());
        let until = Some("2026-07-27".to_string());
        let config = calculate_period("monthly", &since, &until, today).unwrap();
        assert_eq!(config.period_label, "Custom");
        assert_eq!(config.start_date_str, "2026/07/20");
        assert_eq!(config.end_date_str, "2026/07/27");
        assert_eq!(config.is_custom, true);
    }

    #[test]
    fn test_calculate_dynamic_billing_cycle_after_day() {
        let today = NaiveDate::from_ymd_opt(2026, 7, 29).unwrap(); // 29th
        let since = Some("25".to_string());
        let config = calculate_period("monthly", &since, &None, today).unwrap();
        assert_eq!(config.period_label, "Custom");
        assert_eq!(config.start_date_str, "2026/07/25");
        assert_eq!(config.end_date_str, "2026/08/24");
        assert_eq!(config.is_custom, true);
        assert_eq!(config.period_since_formatted, Some("20260725".to_string()));
        assert_eq!(config.period_until_formatted, Some("20260824".to_string()));
    }

    #[test]
    fn test_calculate_dynamic_billing_cycle_before_day() {
        let today = NaiveDate::from_ymd_opt(2026, 7, 20).unwrap(); // 20th
        let since = Some("25th".to_string());
        let config = calculate_period("monthly", &since, &None, today).unwrap();
        assert_eq!(config.period_label, "Custom");
        assert_eq!(config.start_date_str, "2026/06/25");
        assert_eq!(config.end_date_str, "2026/07/24");
        assert_eq!(config.is_custom, true);
        assert_eq!(config.period_since_formatted, Some("20260625".to_string()));
        assert_eq!(config.period_until_formatted, Some("20260724".to_string()));
    }

    #[test]
    fn test_calculate_macos_relative_date_monday() {
        let today = NaiveDate::from_ymd_opt(2026, 7, 29).unwrap(); // Wednesday
        let since = Some("-v-mon".to_string());
        let config = calculate_period("monthly", &since, &None, today).unwrap();
        assert_eq!(config.period_label, "Custom");
        assert_eq!(config.start_date_str, "2026/07/27"); // Prev Monday
        assert_eq!(config.is_custom, true);
        assert_eq!(config.period_since_formatted, Some("20260727".to_string()));
    }
}
