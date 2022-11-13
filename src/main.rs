use std::{env, fs, io, thread, time};
use std::process::{Command};
use std::io::BufRead;


fn main() {
    let args: Vec<String> = env::args().collect();
    println!("{:?}", args);

    let config_path = &args[1];

    let contents = fs::read_to_string(config_path)
        .expect("Should have been able to read the file");

    println!("{}", contents);

    let file = fs::File::open(config_path)
        .expect("Should have been able to read the file");

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

            println!("{:?}", [minute, hour, day, month, weekday, command]);

            for arg in iter {
                println!("arg: {}", arg);
            }
        }
    }

    let wait = time::Duration::from_millis(1000);

    loop {
        println!("Hello!");

        let _child = Command::new("python3.8")
            .arg("-V")
            .spawn()
            .expect("Failed to spawn child process");

        thread::sleep(wait);
    }
}
