use std::{collections::HashMap, iter::zip};

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

pub(crate) fn parse_table(input: &str) -> Vec<HashMap<&str, &str>> {
    let mut lines = input.trim().split('\n').collect::<Vec<&str>>();

    // Pop first element as header
    let header = lines
        .remove(0)
        .split_ascii_whitespace()
        .collect::<Vec<&str>>();
    let num_fields = header.len();
    log::debug!("Parsed input headers {:?}", header);

    // Rest of the lines are data
    // ? Split by 2 or more whitespaces; hacky, may unstable
    let re = regex::Regex::new(r"\s{2,}").expect("Invalid regex");
    let mut table = Vec::with_capacity(lines.len());
    for (i, line) in lines.iter().enumerate() {
        let fields = re.splitn(line, num_fields);
        let zipped = zip(header.clone(), fields);
        let hmap = zipped.collect::<HashMap<&str, &str>>();

        log::debug!("Parsed {i}-th row: {:?}", hmap);
        table.push(hmap);
    }

    table
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

    #[test]
    fn test_parse_table() {
        let cases = [(
            r#"
TYPE       ACTION        LIMIT    USED    REMAINING    RESET
token      write         100      0       100          N/A
token      read          1000     0       1000         N/A
account    read_write    1000     4       996          1 hour from now
"#,
            vec![
                HashMap::from([
                    ("TYPE", "token"),
                    ("ACTION", "write"),
                    ("LIMIT", "100"),
                    ("USED", "0"),
                    ("REMAINING", "100"),
                    ("RESET", "N/A"),
                ]),
                HashMap::from([
                    ("TYPE", "token"),
                    ("ACTION", "read"),
                    ("LIMIT", "1000"),
                    ("USED", "0"),
                    ("REMAINING", "1000"),
                    ("RESET", "N/A"),
                ]),
                HashMap::from([
                    ("TYPE", "account"),
                    ("ACTION", "read_write"),
                    ("LIMIT", "1000"),
                    ("USED", "4"),
                    ("REMAINING", "996"),
                    ("RESET", "1 hour from now"),
                ]),
            ],
        )];
        for (input, expect) in cases {
            let table = parse_table(input);
            assert_eq!(table, expect);
        }
    }
}
