pub fn format_large_number(num: u64) -> String {
    if num == 0 {
        return "0".to_string();
    }

    if num >= 1_000_000_000 {
        let val = num as f64 / 1_000_000_000.0;
        let val_truncated = (val * 100.0).floor() / 100.0;
        format!("{:.2}B", val_truncated)
    } else if num >= 1_000_000 {
        let val = num as f64 / 1_000_000.0;
        let val_truncated = (val * 100.0).floor() / 100.0;
        format!("{:.2}M", val_truncated)
    } else if num >= 1_000 {
        let val = num as f64 / 1_000.0;
        let val_truncated = (val * 100.0).floor() / 100.0;
        format!("{:.2}K", val_truncated)
    } else {
        format!("{}", num)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_under_thousand() {
        assert_eq!(format_large_number(0), "0");
        assert_eq!(format_large_number(42), "42");
        assert_eq!(format_large_number(999), "999");
    }

    #[test]
    fn test_format_thousand() {
        assert_eq!(format_large_number(1000), "1.00K");
        assert_eq!(format_large_number(11640), "11.64K");
        assert_eq!(format_large_number(999990), "999.99K");
    }

    #[test]
    fn test_format_million() {
        assert_eq!(format_large_number(1000000), "1.00M");
        assert_eq!(format_large_number(18083547), "18.08M");
        assert_eq!(format_large_number(262256608), "262.25M");
    }

    #[test]
    fn test_format_billion() {
        assert_eq!(format_large_number(1000000000), "1.00B");
        assert_eq!(format_large_number(6295962523), "6.29B");
    }
}
