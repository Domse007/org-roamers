use std::time::{Duration, Instant};

pub struct PerfPoint {
    start: Instant,
    end: Option<Instant>,
}

impl PerfPoint {
    pub fn new() -> Self {
        Self {
            start: Instant::now(),
            end: None,
        }
    }

    fn duration(&self) -> Duration {
        self.end.unwrap_or_else(Instant::now) - self.start
    }
}

pub struct PerfCollector {
    datapoints: u128,
    avg: Duration,
    last: Duration,
}

impl PerfCollector {
    pub fn new() -> Self {
        Self {
            datapoints: 0,
            avg: Duration::from_secs(0),
            last: Duration::from_secs(0),
        }
    }

    pub fn submit(&mut self, point: PerfPoint) {
        self.submit_internal(point.duration());
    }

    fn submit_internal(&mut self, duration: Duration) {
        if self.datapoints == 0 {
            self.avg = duration;
            self.datapoints += 1;
        } else {
            let avg = (self.datapoints * self.avg.as_nanos() + duration.as_nanos())
                / (self.datapoints + 1);
            self.avg = Duration::from_nanos(avg.try_into().unwrap());
            self.last = duration;
            self.datapoints += 1;
        }
    }

    pub fn report(&self, what: &str) {
        tracing::info!(
            "{} took {}ms with an average of {}ms.",
            what,
            self.last.as_millis(),
            self.avg.as_millis()
        );
    }
}
