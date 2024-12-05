use std::fs::File;
use std::io::{BufReader, Write, BufRead};
use std::os::unix::net::UnixStream;
use chrono::{NaiveTime, Timelike};
use serde::Deserialize;
use tokio::time::{sleep, Duration};

#[derive(Deserialize)]
struct VideoTask {
    delay: String,
    file_path: String,
    start_time: String,
}

fn send_command(socket_path: &str, command: &str) -> Result<(), Box<dyn std::error::Error>> {
    if cfg!(target_os = "windows") {
        let mut stream = File::create(socket_path)?;
        stream.write_all(command.as_bytes())?;
        stream.write_all(b"\n")?;
    } else if !cfg!(target_os = "linux") {
        let mut stream = UnixStream::connect(socket_path)?;
        stream.write_all(command.as_bytes())?;
        stream.write_all(b"\n")?;
    }
    Ok(())
}

fn wait_for_event(socket_path: &str, event: &str) -> Result<(), Box<dyn std::error::Error>> {
    if cfg!(target_os = "windows") {
        let mut stream = File::open(socket_path)?;
        let reader = BufReader::new(stream);

        for line in reader.lines() {
            let line = line?;
            if line.contains(event) {
                break;
            }
        }
    } else if !cfg!(target_os = "linux") {
        let stream = UnixStream::connect(socket_path)?;
        let reader = BufReader::new(stream);

        for line in reader.lines() {
            let line = line?;
            if line.contains(event) {
                break;
            }
        }
    }
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let ipc_socket_file = File::open("socket.txt")?;
    let ipc_socket = BufReader::new(ipc_socket_file).lines().next().unwrap()?.as_str();

    let file = File::open("schedule.json")?;
    let reader = BufReader::new(file);
    let schedule: Vec<VideoTask> = serde_json::from_reader(reader)?;

    for task in &schedule {
        let task_time = NaiveTime::parse_from_str(&task.delay, "%H:%M:%S")?;
        let relative_time = task_time.second();
        if relative_time > 0 {
            sleep(Duration::from_secs(relative_time as u64)).await;
        }

        println!("Playing: {} at {} seconds", task.file_path, task.start_time);

        send_command(ipc_socket, &format!("{{\"command\": [\"loadfile\", \"{}\"]}}", task.file_path))?;
        wait_for_event(ipc_socket, "file-loaded")?;
        send_command(ipc_socket, &format!("{{\"command\": [\"seek\", \"{}\", \"absolute\"]}}", task.start_time))?;

        sleep(Duration::from_secs(1)).await;
    }

    Ok(())
}