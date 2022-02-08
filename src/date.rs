use chrono::{Date, DateTime, FixedOffset, NaiveDate, TimeZone, Utc};
use serde::{self, Deserialize, Deserializer};

// * format => 2021-01-01 00:00:00 +0100 CET
const FORMAT: &str = "%Y-%m-%d %H:%M:%S %z %Z";

pub fn deserialize<'de, D>(deserializer: D) -> Result<DateTime<FixedOffset>, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;

    DateTime::parse_from_str(&s, FORMAT).map_err(serde::de::Error::custom)
}

fn get_day_month(year: i32, month: u32) -> u32 {
    let date: NaiveDate;
    if month == 12 {
        date = NaiveDate::from_ymd(year + 1, 1, 1)
    } else {
        date = NaiveDate::from_ymd(year, month + 1, 1)
    }

    u32::try_from(
        date.signed_duration_since(NaiveDate::from_ymd(year, month, 1))
            .num_days(),
    )
    .expect("failed to convert i64 to u32")
}

pub fn get_past_date(mut year: i32, mut month: u32, mut day: u32, day_remove: u32) -> Date<Utc> {
    if u32::checked_sub(day, day_remove).is_none() || (day - day_remove) < 1 {
        if (month - 1) < 1 {
            month = 12;
            year -= 1;
        } else {
            month -= 1;
        }

        let new_day = get_day_month(year, month);
        day = match day {
            2 => new_day,
            1 => new_day - 1,
            _ => panic!("invalid day in condition..."),
        }
    } else {
        day -= day_remove;
    }

    Utc.from_utc_date(&NaiveDate::from_ymd(year, month, day))
}
