use std::fs::File;
use std::io::{BufReader, Write, BufRead};
use chrono::{NaiveTime, Timelike};
use serde::Deserialize;
use tokio::time::{sleep, Duration};
#[cfg(target_os = "linux")]
use std::os::unix::net::UnixStream;
#[cfg(target_os = "windows")]
use tokio::net::windows::named_pipe::{NamedPipeClient};
use tokio::io::{AsyncWriteExt, AsyncBufReadExt};

#[derive(Deserialize)]
struct VideoTask {
    delay: String,
    file_path: String,
    start_time: String,
}

async fn send_command(socket_path: &str, command: &str) -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(target_os = "linux")]
    {
        let mut stream = UnixStream::connect(socket_path)?;
        stream.write_all(command.as_bytes())?;
        stream.write_all(b"\n")?;
    }

    #[cfg(target_os = "windows")]
    {
        // Use NamedPipeClient::connect to connect to an existing named pipe
        let mut pipe = NamedPipeClient::connect(socket_path).await?;  // Connect to the named pipe
        pipe.write_all(command.as_bytes()).await?;
        pipe.write_all(b"\n").await?;
    }

    Ok(())
}

async fn wait_for_event(socket_path: &str, event: &str) -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(target_os = "linux")]
    {
        let stream = UnixStream::connect(socket_path)?;
        let reader = BufReader::new(stream);
        for line in reader.lines() {
            let line = line?;
            if line.contains(event) {
                break;
            }
        }
    }

    #[cfg(target_os = "windows")]
    {
        // Connect to the named pipe
        let mut pipe = NamedPipeClient::from_raw_handle(socket_path).await?;  // Connect to the named pipe
        let reader = BufReader::new(pipe);
        let mut lines = reader.lines();
        while let Some(line) = lines.next_line().await? {
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
    let binding = BufReader::new(ipc_socket_file).lines().next().unwrap()?;
    let ipc_socket = binding.as_str();

    let file = File::open("schedule.json")?;
    let reader = BufReader::new(file);
    let schedule: Vec<VideoTask> = serde_json::from_reader(reader)?;

    for task in &schedule {
        let task_time = NaiveTime::parse_from_str(&task.delay, "%H:%M:%S")?;
        let total_delay_seconds = task_time.hour() * 3600 + task_time.minute() * 60 + task_time.second();
        if total_delay_seconds > 0 {
            sleep(Duration::from_secs(total_delay_seconds as u64)).await;
        }

        println!("Playing: {} at {} seconds", task.file_path, task.start_time);

        send_command(ipc_socket, &format!("{{\"command\": [\"loadfile\", \"{}\"]}}", task.file_path)).await?;
        wait_for_event(ipc_socket, "file-loaded").await?;
        send_command(ipc_socket, &format!("{{\"command\": [\"seek\", \"{}\", \"absolute\"]}}", task.start_time)).await?;

        sleep(Duration::from_secs(1)).await;
    }

    Ok(())
}
