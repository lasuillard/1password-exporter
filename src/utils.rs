use std::collections::HashMap;

/// Parse given input (`Key: Value` format) into a key-value pair.
pub(crate) fn parse_kv(input: &str) -> HashMap<&str, &str> {
    let mut hmap = HashMap::new();

    for line in input.trim().lines() {
        let parts = line
            .split_once(":")
            .expect(&format!("Invalid input, missing colon: {}", line));
        let (key, value) = parts;

        hmap.insert(key, value.trim());
    }

    hmap
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_kv() {
        let input = r#"
URL:               https://my.1password.com
Integration ID:    WADYB2CBTFBIFKESZ6AV74PUGE
User Type:         SERVICE_ACCOUNT
"#;

        let hmap = parse_kv(input);

        assert_eq!(hmap.get("URL").unwrap(), &"https://my.1password.com");
        assert_eq!(
            hmap.get("Integration ID").unwrap(),
            &"WADYB2CBTFBIFKESZ6AV74PUGE"
        );
        assert_eq!(hmap.get("User Type").unwrap(), &"SERVICE_ACCOUNT");
    }
}
