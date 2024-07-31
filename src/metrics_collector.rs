use std::str::FromStr;

use crate::command_executor::CommandExecutor;

mod rate_limit;

#[derive(Debug, PartialEq)]
pub enum Metrics {
    RateLimit,
}

impl FromStr for Metrics {
    type Err = ();

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        match input {
            "rate_limit" => Ok(Metrics::RateLimit),
            _ => Err(()),
        }
    }
}

pub struct OpMetricsCollector {
    command_executor: Box<dyn CommandExecutor>,
}

impl OpMetricsCollector {
    pub fn new(command_executor: Box<dyn CommandExecutor>) -> Self {
        OpMetricsCollector { command_executor }
    }

    pub fn collect(&self, metrics: Vec<Metrics>) {
        // TODO: Collect all metrics in async manner (use Tokio)
        for metric in metrics {
            match metric {
                Metrics::RateLimit => self.collect_rate_limit(),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics_from_str() {
        assert_eq!(Metrics::from_str("rate_limit"), Ok(Metrics::RateLimit));
        assert_eq!(Metrics::from_str("unknown"), Err(()));
    }
}
