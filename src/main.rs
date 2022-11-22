use regex::Regex;
use std::io::BufRead;
use std::process::Command;
use std::{env, fs, io, thread, time};

#[derive(Debug, Eq, PartialEq)]
enum TimeFieldValue {
    All(Option<u8>),
    Range(u8, u8, Option<u8>),
    Single(u8, Option<u8>),
}

impl TimeFieldValue {
    fn test(&self, n: u8) -> bool {
        match self {
            TimeFieldValue::All(step) => {
                let i = 0;
                i % 2 == 0
            }
            TimeFieldValue::Range(start, end, step) => *start <= n && n <= *end,
            TimeFieldValue::Single(value, step) => *value == n,
        }
    }
}

#[derive(Debug)]
struct Expression {
    minutes: Vec<TimeFieldValue>
}

fn parse_field_expression(field_expression: &str) -> Vec<TimeFieldValue> {
    let mut r = Vec::new();

    for s in field_expression.split(',') {
        let value_str = s.trim();

        println!("{:?}", value_str);

        let value: TimeFieldValue;

        let re_all = Regex::new(r"^\*(?:/(\d+))?$").unwrap();
        let re_range = Regex::new(r"^(\d+)-(\d+)(?:/(\d+))?$").unwrap();
        let re_single = Regex::new(r"^(\d+)(?:/(\d+))?$").unwrap();

        if let Some(caps) = re_all.captures(value_str) {
            let step: Option<u8>;

            if let Some(c) = caps.get(1) {
                step = Some(c.as_str().parse::<u8>().unwrap());
            } else {
                step = None;
            }

            value = TimeFieldValue::All(step);
        } else if let Some(caps) = re_range.captures(value_str) {
            let a = caps.get(1).unwrap().as_str().parse::<u8>().unwrap();
            let b = caps.get(2).unwrap().as_str().parse::<u8>().unwrap();
            let step: Option<u8>;

            if let Some(c) = caps.get(3) {
                step = Some(c.as_str().parse::<u8>().unwrap());
            } else {
                step = None;
            }

            value = TimeFieldValue::Range(a, b, step);
        } else if let Some(caps) = re_single.captures(value_str) {
            let a = caps.get(1).unwrap().as_str().parse::<u8>().unwrap();
            let step: Option<u8>;

            if let Some(c) = caps.get(2) {
                step = Some(c.as_str().parse::<u8>().unwrap());
            } else {
                step = None;
            }

            value = TimeFieldValue::Single(a, step);
        } else {
            panic!("invalid field expression: {}", field_expression);
        }

        println!("{:?}", value);

        r.push(value);
    }

    r
}

fn parse_expression(expression: &str) -> Expression {
    println!("#### parse_expression({:?})", expression);

    let mut iter = expression.split_whitespace();

    let minutes_str = iter.next().unwrap();
    let minutes = parse_field_expression(minutes_str);

    Expression { minutes }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_expression() {
        assert!(TimeFieldValue::Single(0, None) == TimeFieldValue::Single(0, None));
        assert!(TimeFieldValue::Single(0, None) != TimeFieldValue::Single(1, None));

        assert!(TimeFieldValue::All(None).test(1));
        assert!(TimeFieldValue::Single(1, None).test(1));

        let expr = parse_expression("1-10/2 * * * *");
        println!("{:?}", expr);

        // panic!("##### its dummy error");  // for debug
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    println!("Command-line arguments: {:?}", args);

    let config_path = &args[1];

    loop {
        let now = time::SystemTime::now();
        let m = get_minute(now);
        println!("Now: {:?} (minute: {})", now, m);

        let file = fs::File::open(config_path).expect("Should have been able to read the file");

        // Consumes the iterator, returns an (Optional) String
        for line in io::BufReader::new(file).lines() {
            if let Ok(statement) = line {
                let mut iter = statement.split_whitespace();
                let minute = iter.next().unwrap();
                let hour = iter.next().unwrap();
                let day = iter.next().unwrap();
                let month = iter.next().unwrap();
                let weekday = iter.next().unwrap();
                let command = iter.next().unwrap();

                println!(
                    "Readed line from config: {:?}",
                    [minute, hour, day, month, weekday, command]
                );

                println!(
                    "{} == {} == {}",
                    minute,
                    m,
                    minute.parse::<u64>().unwrap() == m
                );

                let mut cmd = Command::new(command);

                for arg in iter {
                    cmd.arg(arg);
                }

                println!("Execute command: {:?}", cmd);

                cmd.spawn().expect("Failed to spawn child process");
            }
        }

        let now = time::SystemTime::now();
        let next_minute = now + time::Duration::from_secs(60);
        let next_minute_epoch = next_minute.duration_since(time::UNIX_EPOCH).unwrap();
        let fraction =
            time::Duration::from_nanos((next_minute_epoch.as_nanos() % (60 * 1000000000)) as u64);
        let wait_until = next_minute - fraction;
        let wait_time = wait_until.duration_since(now).unwrap();

        thread::sleep(wait_time);
    }
}

fn get_minute(t: time::SystemTime) -> u64 {
    t.duration_since(time::UNIX_EPOCH).unwrap().as_secs() % 3600 / 60
}
