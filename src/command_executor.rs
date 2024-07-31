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
