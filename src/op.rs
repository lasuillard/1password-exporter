use std::process::Command;

#[derive(Debug)]
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

pub fn read_ratelimit() -> Vec<RateLimit> {
    let output = Command::new("op")
        .args(["service-account", "ratelimit"])
        .output()
        .unwrap();

    // Parse the output
    let stdout: String = output.stdout.iter().map(|&x| x as char).collect();
    let lines = stdout.trim().split('\n').collect::<Vec<&str>>();

    let mut result = Vec::new();

    // Iterate skipping the header
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

#[cfg(test)]
mod tests {}
