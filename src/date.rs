use chrono::{Date, DateTime, Datelike, FixedOffset, NaiveDate, NaiveDateTime, TimeZone, Utc};
use serde::{self, Deserialize, Deserializer};
use std::{num::TryFromIntError, process::exit};
use log::error;

pub fn deserialize<'de, D>(deserializer: D) -> Result<DateTime<FixedOffset>, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;

    // * format => 2021-01-01 00:00:00 +0100 CET
    DateTime::parse_from_str(&s, "%Y-%m-%d %H:%M:%S %z %Z").map_err(serde::de::Error::custom)
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
    .unwrap_or_else(|_: TryFromIntError| {
        error!("failed to convert i64 to u32");
        exit(1);
    })
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
            _ => {
                error!("invalid day in condition...");
                exit(1)
            }
        }
    } else {
        day -= day_remove;
    }

    Utc.from_utc_date(&NaiveDate::from_ymd(year, month, day))
}

pub fn parse_date(date: Option<String>) -> (DateTime<Utc>, Option<DateTime<Utc>>) {
    let from: DateTime<Utc>;
    let mut to: Option<DateTime<Utc>> = None;

    if let Some(d) = date {
        if d.contains("|") {
            let dates: Vec<&str> = d.split("|").collect();
            let base_from = if dates[0].contains("T") {
                dates[0].to_string()
            } else {
                format!("{}T00:00:00", dates[0])
            };
            let base_to = if dates[1].contains("T") {
                dates[1].to_string()
            } else {
                format!("{}T00:00:00", dates[1])
            };

            let base_from = NaiveDateTime::parse_from_str(&base_from, "%FT%T").unwrap_or_else(|e| {
                error!(
                    "failed to parse from date. Verify its format (example: YYYY-MM-DD or YYYY-MM-DDTHH:MM:SS): {}",
                    e
                );
                exit(1);
            });
            let base_to = NaiveDateTime::parse_from_str(&base_to, "%FT%T").unwrap_or_else(|e| {
                error!(
                    "failed to parse to date. Verify its format (example: YYYY-MM-DD or YYYY-MM-DDTHH:MM:SS): {}",
                    e
                );
                exit(1);
            });
            from = DateTime::<Utc>::from_utc(base_from, Utc);
            to = Some(DateTime::<Utc>::from_utc(base_to, Utc));
        } else {
            let formatted_date = if d.contains("T") { d } else { d + "T00:00:00" };

            let date = NaiveDateTime::parse_from_str(&formatted_date, "%FT%T")
                .unwrap_or_else(|e| {
                    error!(
                    "failed to parse date. Verify its format (example: YYYY-MM-DD or YYYY-MM-DDTHH:MM:SS): {}",
                    e
                );
                    exit(1);
                });

            from = DateTime::<Utc>::from_utc(date, Utc);
        }
    } else {
        let now = Utc::now();

        from = get_past_date(now.year(), now.month(), now.day(), 2).and_hms(1, 0, 0);
    };

    return (from, to);
}
