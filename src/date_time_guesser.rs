use chrono::{DateTime, Duration, NaiveTime, Utc};
use ogn_parser::Timestamp;

pub trait DateTimeGuesser {
    fn guess_date_time(&self, dt: &DateTime<Utc>) -> Option<DateTime<Utc>>;
}

impl DateTimeGuesser for Timestamp {
    fn guess_date_time(&self, dt: &DateTime<Utc>) -> Option<DateTime<Utc>> {
        match self {
            Timestamp::HHMMSS(hh, mm, ss) => {
                let test_time = NaiveTime::from_hms_opt(*hh as u32, *mm as u32, *ss as u32)?;
                let reference_time = dt.naive_utc().time();
                let delta = reference_time - test_time;
                let offset = match delta.num_seconds() {
                    -89000..=-82800 => Some(Duration::days(-1)),
                    -3600..=3600 => Some(Duration::days(0)),
                    82800..=89000 => Some(Duration::days(1)),
                    _ => None,
                }?;
                let new_naive_dt = (dt.naive_utc().date() + offset).and_time(test_time);
                Some(DateTime::<Utc>::from_naive_utc_and_offset(
                    new_naive_dt,
                    Utc,
                ))
            }
            //Timestamp::DDHHMM(_, _, _) => None,
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::DateTimeGuesser;
    use chrono::{DateTime, Utc};
    use ogn_parser::Timestamp;

    #[test]
    fn small_differences() {
        let reference_timestamp = DateTime::parse_from_rfc3339("2023-02-18T20:22:00+00:00")
            .unwrap()
            .with_timezone(&Utc);
        let ts = Timestamp::HHMMSS(20, 21, 30);
        assert_eq!(
            ts.guess_date_time(&reference_timestamp).unwrap(),
            DateTime::parse_from_rfc3339("2023-02-18T20:21:30+00:00").unwrap()
        );

        let ts2 = Timestamp::HHMMSS(20, 22, 30);
        assert_eq!(
            ts2.guess_date_time(&reference_timestamp).unwrap(),
            DateTime::parse_from_rfc3339("2023-02-18T20:22:30+00:00").unwrap()
        );
    }

    #[test]
    fn day_change() {
        let reference_timestamp = DateTime::parse_from_rfc3339("2023-02-18T23:50:00+00:00")
            .unwrap()
            .with_timezone(&Utc);
        let ts = Timestamp::HHMMSS(0, 10, 30);
        assert_eq!(
            ts.guess_date_time(&reference_timestamp).unwrap(),
            DateTime::parse_from_rfc3339("2023-02-19T00:10:30+00:00").unwrap()
        );

        let reference_timestamp = DateTime::parse_from_rfc3339("2023-02-19T00:05:00+00:00")
            .unwrap()
            .with_timezone(&Utc);
        let ts2 = Timestamp::HHMMSS(23, 45, 30);
        assert_eq!(
            ts2.guess_date_time(&reference_timestamp).unwrap(),
            DateTime::parse_from_rfc3339("2023-02-18T23:45:30+00:00").unwrap()
        );
    }
}
