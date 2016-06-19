#[macro_use]
extern crate log;
extern crate time;

use std::default::Default;

// EPOCH is 2016-01-01T00:00:00.000
static EPOCH: u64 = 1451602800000;

#[derive(Debug)]
pub struct CornFlake {
    node_id_bits: i8,
    sequence_bits: i8,

    node_id: u64,
    sequence: u64,

    timestamp_left_shift: i8,
    sequence_mask: u64,

    last_timestamp: u64,
}

impl Default for CornFlake {
    fn default() -> CornFlake {
        CornFlake {
            node_id_bits: 10,
            sequence_bits: 10,

            node_id: 0,
            sequence: 0,

            timestamp_left_shift: 20,
            sequence_mask: !0 ^ (!0 << 10),

            last_timestamp: 0,
        }
    }
}

impl CornFlake {
    pub fn new(node_id: u64) -> CornFlake {
        let mut flake: CornFlake = Default::default();

        assert!(node_id < (1 << flake.node_id_bits));

        flake.node_id = node_id;

        flake
    }

    fn til_next_ms(&self) -> u64 {
        let mut timestamp = time::precise_time_ns() * 1000000;
        while timestamp <= self.last_timestamp {
            timestamp = time::precise_time_ns() * 1000000;
        }
        timestamp
    }

    pub fn node_id(&self) -> u64 {
        self.node_id
    }

    pub fn next_id(&mut self) -> u64 {
        let mut timestamp = time::precise_time_ns();

        if timestamp < self.last_timestamp {
            error!("clock running backwards!!!");
            timestamp = self.til_next_ms();
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

        return ((timestamp - EPOCH) << self.timestamp_left_shift) |
            (self.node_id << self.node_id_bits) | self.sequence;
    }
}



#[cfg(test)]
mod tests {
    use super::CornFlake;

    #[test]
    fn it_works() {
        let mut f = CornFlake::new(1);

        for _ in 1..100000 {
            println!("{} ", f.next_id());
        }
    }
}
