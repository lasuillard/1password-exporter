use crate::command_executor::CommandExecutor;

mod rate_limit;

pub struct OpMetricsScraper {
    command_executor: Box<dyn CommandExecutor>,
}

impl OpMetricsScraper {
    pub fn new(command_executor: Box<dyn CommandExecutor>) -> Self {
        OpMetricsScraper { command_executor }
    }
}
