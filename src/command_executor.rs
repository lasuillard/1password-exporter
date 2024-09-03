use std::process::Command;

#[cfg(test)]
use mockall::{predicate::*, *};

#[cfg_attr(test, automock)]
pub(crate) trait CommandExecutor {
    fn exec(&self, args: Vec<&'static str>) -> Result<String, std::io::Error>;
}

pub(crate) struct OpCommandExecutor {
    op_path: String,
    service_account_token: Option<String>,
}

impl OpCommandExecutor {
    pub(crate) fn new(op_path: String, service_account_token: Option<String>) -> Self {
        OpCommandExecutor {
            op_path,
            service_account_token,
        }
    }
}

impl CommandExecutor for OpCommandExecutor {
    fn exec(&self, args: Vec<&'static str>) -> Result<String, std::io::Error> {
        let mut cmd: &mut Command = &mut Command::new(self.op_path.clone());
        if let Some(sa_token) = &self.service_account_token {
            cmd = cmd.env("OP_SERVICE_ACCOUNT_TOKEN", sa_token);
        }
        let output = cmd.args(args).output()?;
        let stdout: String = output.stdout.iter().map(|&x| x as char).collect();

        Ok(stdout)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_executor() {
        // Arrange
        let executor = OpCommandExecutor::new("op".to_string(), None);
        let output = executor.exec(vec!["--version"]).unwrap();

        // Act
        let version_info = output.trim().split(".").collect::<Vec<&str>>();

        // Assert
        assert_eq!(version_info.len(), 3);
        assert!(version_info[0].parse::<i32>().is_ok());
        assert!(version_info[1].parse::<i32>().is_ok());
        assert!(version_info[2].parse::<i32>().is_ok());
    }
}
