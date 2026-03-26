use crate::dice::DieType;

/// A single recorded roll
#[derive(Debug, Clone, Copy)]
pub struct RollRecord {
    pub die: DieType,
    pub result: u16,
}

/// Circular buffer of roll history
pub struct RollHistory {
    records: [Option<RollRecord>; Self::MAX_RECORDS],
    head: usize,
    count: usize,
}

impl RollHistory {
    const MAX_RECORDS: usize = 20;

    pub fn new() -> Self {
        Self {
            records: [None; Self::MAX_RECORDS],
            head: 0,
            count: 0,
        }
    }

    /// Record a new roll
    pub fn push(&mut self, die: DieType, result: u16) {
        self.records[self.head] = Some(RollRecord { die, result });
        self.head = (self.head + 1) % Self::MAX_RECORDS;
        if self.count < Self::MAX_RECORDS {
            self.count += 1;
        }
    }

    /// Get recent rolls (newest first), up to `limit`
    pub fn recent(&self, limit: usize) -> impl Iterator<Item = &RollRecord> {
        let limit = limit.min(self.count);
        let mut indices = [0usize; 20];
        for (i, idx) in indices.iter_mut().enumerate().take(limit) {
            *idx = if self.head >= i + 1 {
                self.head - i - 1
            } else {
                Self::MAX_RECORDS - (i + 1 - self.head)
            };
        }
        indices[..limit]
            .iter()
            .filter_map(move |&i| self.records[i].as_ref())
            .collect::<heapless::Vec<&RollRecord, 20>>()
            .into_iter()
    }

    /// Total number of recorded rolls
    pub fn count(&self) -> usize {
        self.count
    }

    /// Stats for a specific die type
    pub fn stats_for(&self, die: DieType) -> Option<(u16, u16, u32, usize)> {
        let mut min = u16::MAX;
        let mut max = 0u16;
        let mut sum = 0u32;
        let mut count = 0usize;

        for record in self.records.iter().flatten() {
            if record.die == die {
                min = min.min(record.result);
                max = max.max(record.result);
                sum += record.result as u32;
                count += 1;
            }
        }

        if count > 0 {
            Some((min, max, sum / count as u32, count))
        } else {
            None
        }
    }
}

impl Default for RollHistory {
    fn default() -> Self {
        Self::new()
    }
}
