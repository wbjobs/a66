use std::time::{SystemTime, UNIX_EPOCH};

pub fn now_str() -> String {
    let dur = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    let secs = dur.as_secs() as i64;
    let offset_secs = 8 * 3600;
    let utc_secs = secs + offset_secs;
    let days = utc_secs / 86400;
    let rem = utc_secs % 86400;
    let hours = rem / 3600;
    let mins = (rem % 3600) / 60;
    let secs = rem % 60;
    let (y, m, d) = days_to_ymd(days);
    format!(
        "{:04}-{:02}-{:02} {:02}:{:02}:{:02}",
        y, m, d, hours, mins, secs
    )
}

pub fn days_to_ymd(mut days: i64) -> (i32, u32, u32) {
    days += 719163;
    let mut year: i32 = (400 * days + 140201) as i32 / 146097;
    let mut day_of_year = days as i32 - (365 * year + year / 4 - year / 100 + year / 400);
    while day_of_year < 0 {
        year -= 1;
        day_of_year = days as i32 - (365 * year + year / 4 - year / 100 + year / 400);
    }
    let feb_days = if (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0) {
        29
    } else {
        28
    };
    let m_days = [31, feb_days, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
    let mut month: u32 = 1;
    for &md in &m_days {
        if day_of_year < md {
            break;
        }
        day_of_year -= md;
        month += 1;
    }
    (year, month, day_of_year as u32 + 1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_days_to_ymd() {
        let (y, m, d) = days_to_ymd(0);
        assert_eq!((y, m, d), (1970, 1, 1));

        let (y, m, d) = days_to_ymd(18442);
        assert_eq!((y, m, d), (2020, 7, 1));
    }

    #[test]
    fn test_now_str() {
        let s = now_str();
        assert!(s.len() == 19);
        assert!(s.contains("-"));
        assert!(s.contains(":"));
        assert!(s.contains(" "));
    }
}
