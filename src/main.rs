use std::io::BufRead;
use std::process::Command;
use std::{env, fs, io, thread, time};

fn main() {
    let args: Vec<String> = env::args().collect();
    println!("Command-line arguments: {:?}", args);

    let config_path = &args[1];

    loop {
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
