use chrono::{DateTime, TimeZone, Utc};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum TimeParseError {
    #[error("invalid RFC3339 timestamp")]
    Rfc3339(#[from] chrono::ParseError),
    #[error("invalid epoch timestamp")]
    Epoch,
}

pub fn parse_rfc3339_utc(value: &str) -> Result<DateTime<Utc>, TimeParseError> {
    Ok(DateTime::parse_from_rfc3339(value)?.with_timezone(&Utc))
}

pub fn epoch_seconds_to_utc(value: i64) -> Result<DateTime<Utc>, TimeParseError> {
    Utc.timestamp_opt(value, 0)
        .single()
        .ok_or(TimeParseError::Epoch)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_rfc3339_offset_and_z() {
        let offset = parse_rfc3339_utc("2026-06-10T13:40:00.531727+00:00").unwrap();
        let zulu = parse_rfc3339_utc("2026-06-10T13:40:00.531727Z").unwrap();
        assert_eq!(offset, zulu);
    }

    #[test]
    fn converts_epoch_seconds() {
        let parsed = epoch_seconds_to_utc(1781082464).unwrap();
        assert_eq!(parsed.to_rfc3339(), "2026-06-10T09:07:44+00:00");
    }
}
