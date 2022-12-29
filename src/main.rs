use chrono::{DateTime, Datelike, Duration, Local, SubsecRound, Timelike, Weekday};
use regex::Regex;
use std::io::BufRead;
use std::process::Command;
use std::str::SplitWhitespace;
use std::{env, fs, io, thread};

mod v2 {
    use super::*;

    #[derive(Debug)]
    pub struct Expression {
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
                let fixed = match unfixed {
                    UnfixedTimePattern::All(step) => TimePattern::All(*step),

                    UnfixedTimePattern::Range(start, end, step) => {
                        let mut start2 = Self::str2weeknum(start);
                        let mut end2 = Self::str2weeknum(end);

                        if end2 < start2 {
                            std::mem::swap(&mut start2, &mut end2);
                        }

                        TimePattern::Range(start2, end2, *step)
                    }

                    UnfixedTimePattern::Single(value, step) => {
                        TimePattern::Single(Self::str2weeknum(value), *step)
                    }
                };

                patterns.push(fixed);
            }

            Field { patterns }
        }

        fn fix(field: UnfixedField) -> Field {
            let mut patterns = Vec::new();

            for unfixed in field.patterns.iter() {
                let fixed = match unfixed {
                    UnfixedTimePattern::All(step) => TimePattern::All(*step),

                    UnfixedTimePattern::Range(start, end, step) => TimePattern::Range(
                        start.parse::<u8>().unwrap(),
                        end.parse::<u8>().unwrap(),
                        *step,
                    ),

                    UnfixedTimePattern::Single(value, step) => {
                        TimePattern::Single(value.parse::<u8>().unwrap(), *step)
                    }
                };

                patterns.push(fixed);
            }

            Field { patterns }
        }

        pub fn parse(text: &str) -> (Self, SplitWhitespace) {
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

        pub fn test(&self, dt: DateTime<Local>) -> bool {
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
        patterns: Vec<UnfixedTimePattern<'a>>,
    }

    #[derive(Debug, Eq, PartialEq)]
    enum UnfixedTimePattern<'a> {
        All(Option<u8>),
        Range(&'a str, &'a str, Option<u8>),
        Single(&'a str, Option<u8>),
    }

    #[derive(Debug)]
    struct Field {
        patterns: Vec<TimePattern>,
    }

    #[derive(Debug, Eq, PartialEq)]
    enum TimePattern {
        All(Option<u8>),
        Range(u8, u8, Option<u8>),
        Single(u8, Option<u8>),
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
        let dt = chrono::NaiveDate::from_ymd_opt(2022, 11, 27)
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

fn main() {
    let args: Vec<String> = env::args().collect();
    println!("Command-line arguments: {:?}", args);

    let config_path = &args[1];

    loop {
        let now = Local::now();
        println!("#### Now: {:?}", now);

        let file = fs::File::open(config_path).expect("Should have been able to read the file");

        // Consumes the iterator, returns an (Optional) String
        for line in io::BufReader::new(file).lines().flatten() {
            println!("Readed line from config: {:?}", line);

            let (expression, mut iter) = v2::Expression::parse(&line);

            println!("Time fields parsed to: {:?}", expression);

            println!("## test: {}", expression.test(now));

            if !expression.test(now) {
                continue;
            }

            let command = iter.next().unwrap();

            let mut cmd = Command::new(command);

            for arg in iter {
                cmd.arg(arg);
            }

            println!("Execute command: {:?}", cmd);

            let output = cmd.output().expect("Failed to spawn child process");

            println!("status: {}", output.status.code().unwrap());
            println!("stdout: {}", String::from_utf8_lossy(&output.stdout).trim());
            println!("stderr: {}", String::from_utf8_lossy(&output.stderr).trim());
        }

        let now = Local::now();
        let next_minute = (now + Duration::minutes(1))
            .trunc_subsecs(0)
            .with_second(0)
            .unwrap();
        let until_next_minute = (next_minute - now).to_std().unwrap();

        println!("Wait {:?}", until_next_minute);

        thread::sleep(until_next_minute);
    }
}
