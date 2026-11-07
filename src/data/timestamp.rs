pub fn parse_timestamp(input: &str) -> Result<i64, String> {
    let value = input.trim();
    let (date, time) = value
        .split_once(' ')
        .ok_or_else(|| format!("Invalid timestamp '{value}'; expected YYYY-MM-DD HH:MM[:SS]"))?;
    let date_parts: Vec<&str> = date.split('-').collect();
    let time_parts: Vec<&str> = time.split(':').collect();
    if date_parts.len() != 3 || !(2..=3).contains(&time_parts.len()) {
        return Err(format!(
            "Invalid timestamp '{value}'; expected YYYY-MM-DD HH:MM[:SS]"
        ));
    }

    let part = |text: &str, width: usize| -> Result<i64, String> {
        if text.len() != width || !text.bytes().all(|byte| byte.is_ascii_digit()) {
            return Err(format!("Invalid timestamp '{value}'"));
        }
        text.parse()
            .map_err(|_| format!("Invalid timestamp '{value}'"))
    };

    let year = part(date_parts[0], 4)?;
    let month = part(date_parts[1], 2)?;
    let day = part(date_parts[2], 2)?;
    let hour = part(time_parts[0], 2)?;
    let minute = part(time_parts[1], 2)?;
    let second = match time_parts.get(2) {
        Some(text) => part(text, 2)?,
        None => 0,
    };
    let month_days = match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if year % 400 == 0 || (year % 4 == 0 && year % 100 != 0) => 29,
        2 => 28,
        _ => 0,
    };
    if year == 0 || day == 0 || day > month_days || hour > 23 || minute > 59 || second > 59 {
        return Err(format!("Invalid timestamp '{value}'"));
    }

    let adjusted_year = year - i64::from(month <= 2);
    let era = adjusted_year.div_euclid(400);
    let year_of_era = adjusted_year - era * 400;
    let adjusted_month = month + if month > 2 { -3 } else { 9 };
    let day_of_year = (153 * adjusted_month + 2) / 5 + day - 1;
    let day_of_era = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;
    let days_since_epoch = era * 146097 + day_of_era - 719468;

    // Input is India exchange-local time; convert it to UTC epoch seconds without using the host timezone.
    Ok(days_since_epoch * 86400 + hour * 3600 + minute * 60 + second - 19800)
}

pub fn format_timestamp(timestamp: i64) -> String {
    let local_seconds = timestamp + 19800;
    let days = local_seconds.div_euclid(86400);
    let seconds_of_day = local_seconds.rem_euclid(86400);
    let shifted_days = days + 719468;
    let era = shifted_days.div_euclid(146097);
    let day_of_era = shifted_days - era * 146097;
    let year_of_era =
        (day_of_era - day_of_era / 1460 + day_of_era / 36524 - day_of_era / 146096) / 365;
    let mut year = year_of_era + era * 400;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let month_prime = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * month_prime + 2) / 5 + 1;
    let month = month_prime + if month_prime < 10 { 3 } else { -9 };
    year += i64::from(month <= 2);
    let hour = seconds_of_day / 3600;
    let minute = seconds_of_day % 3600 / 60;
    let second = seconds_of_day % 60;
    format!("{year:04}-{month:02}-{day:02} {hour:02}:{minute:02}:{second:02}")
}

#[cfg(test)]
mod tests {
    use super::{format_timestamp, parse_timestamp};

    #[test]
    fn parses_minute_and_second_precision_as_india_local_time() {
        let minute = parse_timestamp("2026-10-20 09:15").unwrap();
        let second = parse_timestamp("2026-10-20 09:15:30").unwrap();
        assert_eq!(second - minute, 30);
        assert_eq!(format_timestamp(minute), "2026-10-20 09:15:00");
    }

    #[test]
    fn rejects_invalid_dates_and_times_but_accepts_leap_day() {
        assert!(parse_timestamp("2024-02-29 09:15").is_ok());
        assert!(parse_timestamp("2026-02-29 09:15").is_err());
        assert!(parse_timestamp("2026-04-31 09:15").is_err());
        assert!(parse_timestamp("2026-10-20 24:00").is_err());
        assert!(parse_timestamp("2026-10-20 09:15:60").is_err());
        assert!(parse_timestamp("2026/10/20 09:15").is_err());
    }
}
