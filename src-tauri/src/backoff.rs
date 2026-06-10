use std::time::Duration;

const SEQUENCE_MINUTES: [u64; 5] = [3, 6, 12, 24, 30];

#[derive(Debug, Clone, Default)]
pub struct Backoff {
    attempts: usize,
}

impl Backoff {
    pub fn next_delay(&mut self) -> Duration {
        let index = self.attempts.min(SEQUENCE_MINUTES.len() - 1);
        self.attempts += 1;
        Duration::from_secs(SEQUENCE_MINUTES[index] * 60)
    }

    pub fn reset(&mut self) {
        self.attempts = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rate_limit_backoff_caps_at_thirty_minutes() {
        let mut backoff = Backoff::default();
        let delays: Vec<u64> = (0..7)
            .map(|_| backoff.next_delay().as_secs() / 60)
            .collect();
        assert_eq!(delays, vec![3, 6, 12, 24, 30, 30, 30]);
    }
}
