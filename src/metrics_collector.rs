use crate::command_executor::CommandExecutor;

mod rate_limit;

pub struct OpMetricsCollector {
    command_executor: Box<dyn CommandExecutor>,
}

impl OpMetricsCollector {
    pub fn new(command_executor: Box<dyn CommandExecutor>) -> Self {
        OpMetricsCollector { command_executor }
    }
}
