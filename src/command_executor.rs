use std::process::Command;

#[cfg(test)]
use mockall::{predicate::*, *};

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_executor() {
        // Arrange
        let executor = OpCommandExecutor {};
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
