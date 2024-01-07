use core::fmt;

use chrono::{DateTime, NaiveDateTime, TimeZone, Utc};
use itertools::Itertools;

enum Precision {
    Second,
    Minute,
    Day,
}

#[repr(i64)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Interval {
    OneSecond = 1,
    OneMinute = 60,
    ThreeMinutes = 180,
    FiveMinutes = 300,
    FifteenMinutes = 900,
    ThirtyMinutes = 1800,
    OneHour = 3600,
    TwoHours = 7200,
    FourHours = 14400,
    SixHours = 21600,
    EightHours = 28800,
    TwelveHours = 43200,
    OneDay = 86400,
    ThreeDays = 259200,
    OneWeek = 604800,
}

impl Interval {
    fn render_gap(&self) -> usize {
        match self {
            Interval::OneSecond => 30,
            Interval::OneMinute => 15,
            Interval::ThreeMinutes => 20,
            Interval::FiveMinutes => 12,
            Interval::FifteenMinutes => 8,
            Interval::ThirtyMinutes => 8,
            Interval::OneHour => todo!(),
            Interval::TwoHours => todo!(),
            Interval::FourHours => todo!(),
            Interval::SixHours => todo!(),
            Interval::EightHours => todo!(),
            Interval::TwelveHours => todo!(),
            Interval::OneDay => todo!(),
            Interval::ThreeDays => todo!(),
            Interval::OneWeek => todo!(),
        }
    }

    fn render_precision(&self) -> Precision {
        match self {
            Interval::OneSecond => Precision::Second,
            Interval::OneMinute => Precision::Minute,
            Interval::ThreeMinutes => Precision::Minute,
            Interval::FiveMinutes => Precision::Minute,
            Interval::FifteenMinutes => Precision::Minute,
            Interval::ThirtyMinutes => Precision::Minute,
            Interval::OneHour => Precision::Minute,
            Interval::TwoHours => Precision::Minute,
            Interval::FourHours => Precision::Minute,
            Interval::SixHours => Precision::Minute,
            Interval::EightHours => Precision::Minute,
            Interval::TwelveHours => Precision::Minute,
            Interval::OneDay => Precision::Day,
            Interval::ThreeDays => Precision::Day,
            Interval::OneWeek => Precision::Day,
        }
    }
}

pub(crate) struct XAxis {
    width: u16,
    min: i64,
    max: i64,
    interval: Interval,
}

impl XAxis {
    pub fn new(width: u16, min: i64, max: i64, unit: Interval) -> Self {
        assert!(min <= max);
        Self {
            width,
            min,
            max,
            interval: unit,
        }
    }

    /// render priority
    ///
    /// 1. second diff      -> HH:MM:SS
    /// 2. minute/hour diff -> HH:MM
    /// 3. day/month diff   -> mm/dd
    /// 4. year diff        -> YYYY
    ///
    /// worst case: last one is "YYYY-mm-dd HH:MM:SS"(19 chars)
    pub fn render(&self) -> Vec<String> {
        let width = self.width as usize;

        let mut result = vec!["─".repeat(width), " ".repeat(width)];

        let timestamps = (self.min..=self.max)
            .step_by(self.interval as usize * 1000)
            .map(|t| {
                let naive = NaiveDateTime::from_timestamp_millis(t).unwrap();
                (t, Utc.from_utc_datetime(&naive))
            })
            .collect_vec();
        let timestamp_len = timestamps.len();

        if timestamps.len() == 1 {
            let now = Utc::now();
            let (_, last) = timestamps.last().unwrap();
            let rendered = shorted_now_string(now, *last, self.interval.render_precision());
            let written = overwrite_string(
                &mut result[1],
                (timestamp_len - 1) as isize - (rendered.len() / 2) as isize,
                rendered,
                true,
            );
            if written {
                result[0].replace_range((timestamp_len - 1) * 3..timestamp_len * 3, "┴");
            }
        } else if timestamps.len() > 1 {
            // handle last timestamp
            {
                let (_, prev) = timestamps[timestamp_len - 2];
                let (_, now) = timestamps.last().unwrap();
                let rendered = shorted_now_string(prev, *now, self.interval.render_precision());
                let written = overwrite_string(
                    &mut result[1],
                    (timestamp_len - 1) as isize - (rendered.len() / 2) as isize,
                    rendered,
                    true,
                );
                if written {
                    result[0].replace_range((timestamp_len - 1) * 3..timestamp_len * 3, "┴");
                }
            }

            let gap = self.interval.render_gap() as i64 * (self.interval as i64) * 1000;
            for (idx, tuples) in timestamps.windows(2).enumerate() {
                let (_, prev) = tuples[0];
                let (timestamp, now) = tuples[1];

                if timestamp % gap != 0 {
                    continue;
                }

                let rendered = diff_datetime_string(prev, now);
                let written = overwrite_string(
                    &mut result[1],
                    (idx + 1) as isize - (rendered.len() / 2) as isize,
                    format!(" {} ", rendered),
                    false,
                );

                if written {
                    result[0].replace_range((idx + 2) * 3..(idx + 3) * 3, "┴");
                }
            }
        }

        result
    }
}

fn shorted_now_string<Tz: TimeZone>(
    prev: DateTime<Tz>,
    now: DateTime<Tz>,
    precision: Precision,
) -> String
where
    Tz::Offset: fmt::Display,
{
    let prev_year = prev.format("%Y").to_string();
    let now_year = now.format("%Y").to_string();
    if prev_year != now_year {
        return match precision {
            Precision::Second => now.format("%Y/%m/%d %H:%M:%S"),
            Precision::Minute => now.format("%Y/%m/%d %H:%M"),
            Precision::Day => now.format("%Y/%m/%d"),
        }
        .to_string();
    }

    let prev_date = prev.format("%m/%d").to_string();
    let now_date = now.format("%m/%d").to_string();
    if prev_date != now_date {
        return match precision {
            Precision::Second => now.format("%m/%d %H:%M:%S"),
            Precision::Minute => now.format("%m/%d %H:%M"),
            Precision::Day => now.format("%m/%d"),
        }
        .to_string();
    }

    let prev_detailed_time = prev.format("%H:%M:%S").to_string();
    let now_detailed_time = now.format("%H:%M:%S").to_string();
    if prev_detailed_time != now_detailed_time {
        return match precision {
            Precision::Second => now.format("%H:%M:%S"),
            Precision::Minute => now.format("%H:%M"),
            Precision::Day => now.format("%m/%d"),
        }
        .to_string();
    }

    String::default()
}

fn diff_datetime_string<Tz: TimeZone>(prev: DateTime<Tz>, now: DateTime<Tz>) -> String
where
    Tz::Offset: fmt::Display,
{
    let prev_year = prev.format("%Y").to_string();
    let now_year = now.format("%Y").to_string();
    if prev_year != now_year {
        return now_year;
    }

    let prev_date = prev.format("%m/%d").to_string();
    let now_date = now.format("%m/%d").to_string();
    if prev_date != now_date {
        return now_date;
    }

    let prev_time = prev.format("%H:%M").to_string();
    let now_time = now.format("%H:%M").to_string();
    if prev_time != now_time {
        return now_time;
    }

    let prev_detailed_time = prev.format("%H:%M:%S").to_string();
    let now_detailed_time = now.format("%H:%M:%S").to_string();
    if prev_detailed_time != now_detailed_time {
        return now_detailed_time;
    }

    String::default()
}

fn overwrite_string(str: &mut String, idx: isize, value: String, overlap: bool) -> bool {
    if str.len() < value.len() {
        return false;
    }

    let idx = if idx < 0 {
        0
    } else if str.len() < idx as usize + value.len() {
        str.len() - value.len()
    } else {
        idx as usize
    };

    if !overlap {
        for char in str[idx..(idx + value.len())].chars() {
            if char != ' ' {
                // not allow overlap string value
                return false;
            }
        }
    }

    str.replace_range(idx..(idx + value.len()), value.as_str());
    true
}

#[cfg(test)]
mod tests {
    use crate::x_axis::{overwrite_string, Interval};

    use super::XAxis;

    #[test]
    fn test_overwrite_string() {
        let mut str = "x".repeat(10);
        overwrite_string(&mut str, 2, String::from("yy"), true);
        assert_eq!(str, String::from("xxyyxxxxxx"));

        let mut str = "x".repeat(10);

        overwrite_string(&mut str, 8, String::from("zzzzz"), true);
        assert_eq!(str, String::from("xxxxxzzzzz"));

        let mut str = "x".repeat(10);
        overwrite_string(&mut str, -2, String::from("zzzzz"), true);
        assert_eq!(str, String::from("zzzzzxxxxx"));
    }

    #[test]
    fn render() {
        let axis = XAxis::new(60, 1704006000000, 1704009600000, Interval::OneMinute);
        assert_eq!(
            axis.render(),
            vec![
                "───────────────┴──────────────┴──────────────┴─────────────┴",
                "             07:15          07:30          07:45       08:00"
            ]
        );
    }
}
