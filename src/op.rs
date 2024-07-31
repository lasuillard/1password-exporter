use std::process::Command;

#[cfg(test)]
use mockall::{predicate::*, *};

#[derive(Debug, PartialEq)]
/// 1Password API rate limit data.
///
/// Retrieved from CLI `op service-account ratelimit`.
pub struct RateLimit {
    pub type_: String,
    pub action: String,
    pub limit: i32,
    pub used: i32,
    pub remaining: i32,
    #[allow(dead_code)]
    pub reset: String,
}

#[cfg_attr(test, automock)]
pub trait CommandExecutor {
    fn exec(&self, args: Vec<&'static str>) -> Result<String, std::io::Error>;
}

pub struct OpCommandExecutor {}

impl CommandExecutor for OpCommandExecutor {
    fn exec(&self, args: Vec<&'static str>) -> Result<String, std::io::Error> {
        let output = Command::new("op").args(args).output()?;
        let stdout: String = output.stdout.iter().map(|&x| x as char).collect();

        Ok(stdout)
    }
}

pub struct OpMetricsReader {
    command_executor: Box<dyn CommandExecutor>,
}

impl OpMetricsReader {
    pub fn new(command_executor: Box<dyn CommandExecutor>) -> Self {
        OpMetricsReader { command_executor }
    }

    pub fn read_ratelimit(&self) -> Vec<RateLimit> {
        let output = self
            .command_executor
            .exec(vec!["service-account", "ratelimit"])
            .unwrap();
        let lines = output.trim().split('\n').collect::<Vec<&str>>();

        // Iterate skipping the header
        let mut result = Vec::new();
        for line in lines.iter().skip(1) {
            let fields = line.split_ascii_whitespace().collect::<Vec<&str>>();
            let rate_limit = RateLimit {
                type_: fields[0].to_string(),
                action: fields[1].to_string(),
                limit: fields[2].parse().unwrap(),
                used: fields[3].parse().unwrap(),
                remaining: fields[4].parse().unwrap(),
                reset: fields[5..].join(" ").to_string(),
            };
            result.push(rate_limit);
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mytest() {
        let mut command_executor = MockCommandExecutor::new();
        command_executor.expect_exec().returning(|_| {
            Ok(r#"
TYPE       ACTION        LIMIT    USED    REMAINING    RESET
token      write         100      0       100          N/A
token      read          1000     0       1000         N/A
account    read_write    1000     4       996          1 hour from now
"#
            .to_string())
        });
        let metrics_reader = OpMetricsReader::new(Box::new(command_executor));

        let ratelimit = metrics_reader.read_ratelimit();

        assert_eq!(ratelimit.len(), 3);
        assert_eq!(
            ratelimit[0],
            RateLimit {
                type_: "token".to_string(),
                action: "write".to_string(),
                limit: 100,
                used: 0,
                remaining: 100,
                reset: "N/A".to_string(),
            }
        );
        assert_eq!(
            ratelimit[1],
            RateLimit {
                type_: "token".to_string(),
                action: "read".to_string(),
                limit: 1000,
                used: 0,
                remaining: 1000,
                reset: "N/A".to_string(),
            }
        );
        assert_eq!(
            ratelimit[2],
            RateLimit {
                type_: "account".to_string(),
                action: "read_write".to_string(),
                limit: 1000,
                used: 4,
                remaining: 996,
                reset: "1 hour from now".to_string(),
            }
        );
    }
}
