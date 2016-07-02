extern crate time;

use std::default::Default;
use std::error::Error as StdError;
use std::fmt;
use std::result;

// DEFAULT_EPOCH is 2016-01-01T00:00:00.000
static DEFAULT_EPOCH: u64 = 1451602800;

#[derive(Debug, Clone)]
pub struct Config {
    pub node_id_bits: i8,
    pub sequence_bits: i8,
    pub node_id: u64,
    pub epoch: u64,
}

impl fmt::Display for Config {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "config-{:x}-{:x}-{:x}", self.node_id_bits, self.node_id, self.sequence_bits)
    }
}

#[derive(Debug)]
pub struct CornFlake {
    node_id: u64,
    sequence: u64,
    sequence_mask: u64,
    last_timestamp: u64,
    epoch: u64,

    node_id_left_shift: i8,
    timestamp_left_shift: i8,
}

impl fmt::Display for CornFlake {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "flake-{:x}-{:x}-{:x}", self.last_timestamp, self.node_id, self.sequence)
    }
}

#[derive(Debug)]
pub enum CornFlakeError {
    TooFewTimestampBits,
    NodeIdTooBig(u64),
    ClockRunningBackwards,
}

impl fmt::Display for CornFlakeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &CornFlakeError::TooFewTimestampBits => write!(f, "TooFewTimestampBits (less then 41bit)"),
            &CornFlakeError::NodeIdTooBig(ref id) => write!(f, "NodeIdTooBig: {}", id),
            &CornFlakeError::ClockRunningBackwards => write!(f, "Clock running backwards!!!"),
        }
    }
}

impl StdError for CornFlakeError {
    fn description(&self) -> &str {
        match self {
            &CornFlakeError::TooFewTimestampBits => "TooFewTimestampBits (less then 41bit)",
            &CornFlakeError::NodeIdTooBig(_) => "NodeIdTooBig",
            &CornFlakeError::ClockRunningBackwards => "Clock running backwards!!!",
        }
    }
}

pub type Result<T> = result::Result<T, CornFlakeError>;

impl Default for Config {
    fn default() -> Config {
        Config {
            node_id_bits: 10,
            sequence_bits: 10,
            node_id: 0,
            epoch: DEFAULT_EPOCH,
        }
    }
}

impl CornFlake {
    pub fn new(config: &Config) -> Result<CornFlake> {
        if config.node_id_bits + config.sequence_bits > 22 {
            return Err(CornFlakeError::TooFewTimestampBits);
        }
        if config.node_id > (1 << config.node_id_bits) {
            return Err(CornFlakeError::NodeIdTooBig(config.node_id));
        }

        Ok(CornFlake {
            node_id: config.node_id,
            sequence: 0,
            last_timestamp: 0,
            epoch: config.epoch,

            node_id_left_shift: config.node_id_bits,
            timestamp_left_shift: config.node_id_bits + config.sequence_bits,
            sequence_mask: !0 ^ (!0 << config.sequence_bits),
        })
    }

    #[inline(always)]
    fn epoch_timestamp(&self) -> u64 {
        let t = time::get_time();
        ((t.sec as u64 - self.epoch) * 1000) + (t.nsec / 1000000) as u64
    }

    fn til_next_ms(&self) -> u64 {
        let mut timestamp = self.epoch_timestamp();
        while timestamp <= self.last_timestamp {
            timestamp = self.epoch_timestamp();
        }
        timestamp
    }

    pub fn node_id(&self) -> u64 {
        self.node_id
    }

    pub fn next_id(&mut self) -> Result<u64> {
        let mut timestamp = self.epoch_timestamp();

        if timestamp < self.last_timestamp {
            return Err(CornFlakeError::ClockRunningBackwards);
        }

        if timestamp == self.last_timestamp {
            self.sequence = (self.sequence + 1) & self.sequence_mask;
            if self.sequence == 0 {
                timestamp = self.til_next_ms();
            }
        } else {
            self.sequence = 0;
        }

        self.last_timestamp = timestamp;

        Ok((timestamp << self.timestamp_left_shift) | (self.node_id << self.node_id_left_shift) | self.sequence)
    }
}


#[cfg(test)]
mod tests {
    use super::Config;
    use super::CornFlake;

    #[test]
    fn initialize_and_run_default() {
        let c: Config = Default::default();
        let mut f = CornFlake::new(&c).unwrap();

        for _ in 1..100 {
            println!("{} ", f.next_id().unwrap());
        }
    }
}
