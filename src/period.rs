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
}

pub fn calculate_period(
    period: &str,
    since: &Option<String>,
    until: &Option<String>,
    today: NaiveDate,
) -> Result<PeriodConfig, String> {
    if since.is_some() || until.is_some() {
        let since_str = since.clone().unwrap_or_else(|| {
            let prev_month = today - Duration::days(30);
            prev_month.format("%Y%m%d").to_string()
        });
        let until_str = until.clone().unwrap_or_else(|| {
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
}
