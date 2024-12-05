use serde::{Deserialize, Serialize};
use std::fs;
use std::time::Duration;
use tokio::process::Command;
use tokio::time::sleep;
use std::io::{Write, Read};
use std::os::windows::io::AsRawHandle;
use winapi::um::namedpipeapi::CreateFileA;
use winapi::um::fileapi::FILE_GENERIC_WRITE;
use winapi::um::winnt::{FILE_ATTRIBUTE_NORMAL, GENERIC_WRITE};
use winapi::um::handleapi::INVALID_HANDLE_VALUE;

#[derive(Debug, Serialize, Deserialize)]
struct VideoTask {
    delay: String,
    file_path: String,
    start_time: u64,
}

// Parse the JSON schedule
async fn parse_schedule(file_path: &str) -> anyhow::Result<Vec<VideoTask>> {
    let file_content = fs::read_to_string(file_path)?;
    let schedule: Vec<VideoTask> = serde_json::from_str(&file_content)?;
    Ok(schedule)
}

// Send a command to MPV via a named pipe
fn send_command(pipe_name: &str, command: &str) -> anyhow::Result<()> {
    unsafe {
        let pipe_handle = CreateFileA(
            pipe_name.as_ptr() as *const i8,
            GENERIC_WRITE,
            0,
            std::ptr::null_mut(),
            3, // OPEN_EXISTING
            FILE_ATTRIBUTE_NORMAL,
            std::ptr::null_mut(),
        );

        if pipe_handle == INVALID_HANDLE_VALUE {
            return Err(anyhow::anyhow!("Failed to open named pipe"));
        }

        let mut pipe = std::fs::File::from_raw_handle(pipe_handle as *mut _);
        pipe.write_all(command.as_bytes())?;
        pipe.write_all(b"\n")?;
        Ok(())
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Path to MPV executable and named pipe
    let mpv_executable = "./mpv_dir/mpv.exe";
    let ipc_pipe_name = r"\\.\pipe\mpv_ipc";

    // Start MPV with named pipe
    let mut mpv_process = Command::new(mpv_executable)
        .args(["--idle", "--input-ipc-server", ipc_pipe_name])
        .spawn()?;

    let schedule = parse_schedule("schedule.json").await?;

    for task in schedule {
        // Parse the delay
        let delay_parts: Vec<u64> = task
            .delay
            .split(':')
            .map(|part| part.parse::<u64>().unwrap_or(0))
            .collect();

        let delay_duration = Duration::from_secs(
            delay_parts[0] * 3600 + delay_parts[1] * 60 + delay_parts[2],
        );

        if delay_duration > Duration::from_secs(0) {
            sleep(delay_duration).await;
        }

        println!("Playing: {} at {} seconds", task.file_path, task.start_time);

        // Send loadfile command
        let loadfile_command = format!(
            "{{\"command\":[\"loadfile\",\"{}\"]}}",
            task.file_path.replace("\\", "/")
        );
        send_command(ipc_pipe_name, &loadfile_command)?;

        // Send seek command
        let seek_command = format!(
            "{{\"command\":[\"seek\",{},\"absolute\"]}}",
            task.start_time
        );
        send_command(ipc_pipe_name, &seek_command)?;

        // Small delay to ensure commands are processed
        sleep(Duration::from_secs(1)).await;
    }

    // Wait for MPV process to exit
    mpv_process.wait().await?;

    Ok(())
}
