use chrono::{DateTime, Datelike, Local, NaiveDate, Timelike, Weekday};
use regex::Regex;
use std::io::BufRead;
use std::process::Command;
use std::str::SplitWhitespace;
use std::{env, fs, io, thread, time};

mod v2 {
    use super::*;

    #[derive(Debug)]
    struct Expression {
        minutes: Field,
        hours: Field,
        days: Field,
        months: Field,
        days_of_week: Field,
    }

    impl Expression {
        fn parse_field(field: &str) -> UnfixedField {
            let re_all = Regex::new(r"^\*(?:/(\d+))?$").unwrap();
            let re_range = Regex::new(r"^([^-]+)-([^/]+)(?:/(\d+))?$").unwrap();
            let re_single = Regex::new(r"^([^/]+)(?:/(\d+))?$").unwrap();

            let mut patterns = Vec::new();

            for pattern_str in field.split(',') {
                let pattern: UnfixedTimePattern;

                if let Some(caps) = re_all.captures(pattern_str) {
                    let step: Option<u8>;

                    if let Some(c) = caps.get(1) {
                        step = Some(c.as_str().parse::<u8>().unwrap());
                    } else {
                        step = None;
                    }

                    pattern = UnfixedTimePattern::All(step);
                } else if let Some(caps) = re_range.captures(pattern_str) {
                    let a = caps.get(1).unwrap().as_str();
                    let b = caps.get(2).unwrap().as_str();
                    let step: Option<u8>;

                    if let Some(c) = caps.get(3) {
                        step = Some(c.as_str().parse::<u8>().unwrap());
                    } else {
                        step = None;
                    }

                    pattern = UnfixedTimePattern::Range(a, b, step);
                } else if let Some(caps) = re_single.captures(pattern_str) {
                    let a = caps.get(1).unwrap().as_str();
                    let step: Option<u8>;

                    if let Some(c) = caps.get(2) {
                        step = Some(c.as_str().parse::<u8>().unwrap());
                    } else {
                        step = None;
                    }

                    pattern = UnfixedTimePattern::Single(a, step);
                } else {
                    panic!("invalid field: {}", field);
                }

                patterns.push(pattern);
            }

            UnfixedField { patterns }
        }

        fn str2weeknum(s: &str) -> u8 {
            if s.chars().all(|x| x.is_numeric()) {
                let n = s.parse::<u8>().unwrap();
                if n == 0 {
                    7
                } else {
                    n
                }
            } else {
                s.parse::<Weekday>()
                    .unwrap()
                    .number_from_monday()
                    .try_into()
                    .unwrap()
            }
        }

        fn fix_days_of_week(field: UnfixedField) -> Field {
            let mut patterns = Vec::new();

            for unfixed in field.patterns.iter() {
                let fixed: TimePattern;

                match unfixed {
                    UnfixedTimePattern::All(step) => {
                        fixed = TimePattern::All(*step);
                    }

                    UnfixedTimePattern::Range(start, end, step) => {
                        let mut start2 = Self::str2weeknum(start);
                        let mut end2 = Self::str2weeknum(end);

                        if end2 < start2 {
                            let tmp = start2;
                            start2 = end2;
                            end2 = tmp;
                        }

                        fixed = TimePattern::Range(start2, end2, *step);
                    }

                    UnfixedTimePattern::Single(value, step) => {
                        fixed = TimePattern::Single(Self::str2weeknum(value), *step);
                    }
                }

                patterns.push(fixed);
            }

            Field { patterns }
        }

        fn fix(field: UnfixedField) -> Field {
            let mut patterns = Vec::new();

            for unfixed in field.patterns.iter() {
                let fixed: TimePattern;

                match unfixed {
                    UnfixedTimePattern::All(step) => {
                        fixed = TimePattern::All(*step);
                    }

                    UnfixedTimePattern::Range(start, end, step) => {
                        fixed = TimePattern::Range(
                            start.parse::<u8>().unwrap(),
                            end.parse::<u8>().unwrap(),
                            *step,
                        );
                    }

                    UnfixedTimePattern::Single(value, step) => {
                        fixed = TimePattern::Single(value.parse::<u8>().unwrap(), *step);
                    }
                }

                patterns.push(fixed);
            }

            Field { patterns }
        }

        fn parse(text: &str) -> (Self, SplitWhitespace) {
            let mut iter = text.split_whitespace();
            let minutes = Self::fix(Self::parse_field(iter.next().unwrap()));
            let hours = Self::fix(Self::parse_field(iter.next().unwrap()));
            let days = Self::fix(Self::parse_field(iter.next().unwrap()));
            let months = Self::fix(Self::parse_field(iter.next().unwrap()));

            let unfixed_days_of_week = Self::parse_field(iter.next().unwrap());
            let days_of_week = Self::fix_days_of_week(unfixed_days_of_week);

            (
                Self {
                    minutes,
                    hours,
                    days,
                    months,
                    days_of_week,
                },
                iter,
            )
        }

        fn test(&self, dt: DateTime<Local>) -> bool {
            self.minutes
                .patterns
                .iter()
                .all(|x| x.test(dt.minute().try_into().unwrap()))
                && self
                    .hours
                    .patterns
                    .iter()
                    .all(|x| x.test(dt.hour().try_into().unwrap()))
                && self
                    .days
                    .patterns
                    .iter()
                    .all(|x| x.test(dt.day().try_into().unwrap()))
                && self
                    .months
                    .patterns
                    .iter()
                    .all(|x| x.test(dt.month().try_into().unwrap()))
                && self
                    .days_of_week
                    .patterns
                    .iter()
                    .all(|x| x.test(dt.weekday().number_from_monday().try_into().unwrap()))
        }
    }

    #[derive(Debug)]
    struct UnfixedField<'a> {
        patterns: Vec<UnfixedTimePattern<'a>>
    }

    #[derive(Debug, Eq, PartialEq)]
    enum UnfixedTimePattern<'a> {
        All(Option<u8>),
        Range(&'a str, &'a str, Option<u8>),
        Single(&'a str, Option<u8>)
    }

    #[derive(Debug)]
    struct Field {
        patterns: Vec<TimePattern>
    }

    #[derive(Debug, Eq, PartialEq)]
    enum TimePattern {
        All(Option<u8>),
        Range(u8, u8, Option<u8>),
        Single(u8, Option<u8>)
    }

    impl TimePattern {
        fn test(&self, n: u8) -> bool {
            match self {
                TimePattern::All(step) => {
                    let i = 0;
                    i % 2 == 0
                }
                TimePattern::Range(start, end, step) => *start <= n && n <= *end,
                TimePattern::Single(time, step) => *time == n,
            }
        }
    }

    #[test]
    fn test() {
        assert!(TimePattern::All(None) == TimePattern::All(None));

        // 2022-11-27 (Sun) 19:01:00.000
        let dt = NaiveDate::from_ymd_opt(2022, 11, 27)
            .unwrap()
            .and_hms_milli_opt(19, 1, 00, 00)
            .unwrap()
            .and_local_timezone(Local)
            .unwrap();

        println!("#### DateTime: {:?}", dt);

        let mut expression = "1 19 27 11 7";
        let (mut expr, _) = Expression::parse(expression);

        println!("#### Expression: {:?} to {:?}", expression, expr);

        assert!(expr.test(dt));

        expression = "1 19 27 11 0"; // days_of_week 0 == 7
        (expr, _) = Expression::parse(expression);

        println!("#### Expression: {:?} to {:?}", expression, expr);

        assert!(expr.test(dt));

        expression = "0 19 27 11 7";
        (expr, _) = Expression::parse(expression);

        println!("#### Expression: {:?} to {:?}", expression, expr);

        assert!(!expr.test(dt));

        panic!("##### its dummy error"); // for debug
    }
}

#[derive(Debug)]
struct Expression {
    minutes: Vec<TimeFieldValue>,
    hours: Vec<TimeFieldValue>,
    days: Vec<TimeFieldValue>,
    months: Vec<TimeFieldValue>,
    days_of_week: Vec<TimeFieldValue>,
}

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

fn parse_expression(expression: &str) -> (Expression, SplitWhitespace) {
    let mut iter = expression.split_whitespace();
    let minutes = parse_field_expression(iter.next().unwrap());
    let hours = parse_field_expression(iter.next().unwrap());
    let days = parse_field_expression(iter.next().unwrap());
    let months = parse_field_expression(iter.next().unwrap());
    let days_of_week = parse_field_expression(iter.next().unwrap());
    (
        Expression {
            minutes,
            hours,
            days,
            months,
            days_of_week,
        },
        iter,
    )
}

fn parse_field_expression(field_expression: &str) -> Vec<TimeFieldValue> {
    let mut r = Vec::new();

    for value_str in field_expression.split(',') {
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

        r.push(value);
    }

    r
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

        let expr = parse_expression("1-10/2 *  * * *").0;
        println!("#### Parsed: {:?}", expr);

        // // ================================================================
        // // 実験中
        // // ================================================================

        // let now = Local::now();

        // println!("#### Now: {:?}", now);

        // println!(
        //     "{} ({}) {} {} {} {} {}",
        //     now.minute(), expr.minutes.test(now.minute()),
        //     now.hour(),
        //     now.day(),
        //     now.month(),
        //     now.weekday().number_from_monday(),
        //     now.weekday().to_string().parse::<Weekday>().unwrap()
        // );

        // // ================================================================
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
            if let Ok(expression) = line {
                println!("Readed line from config: {:?}", expression);

                let (expr, mut iter) = parse_expression(&expression);

                println!("Time fields parsed to: {:?}", expr);

                let command = iter.next().unwrap();

                let mut cmd = Command::new(command);

                for arg in iter {
                    cmd.arg(arg);
                }

                println!("Execute command: {:?}", cmd);

                // cmd.spawn().expect("Failed to spawn child process");
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
