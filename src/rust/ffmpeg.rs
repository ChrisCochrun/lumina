use dirs;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::str;

pub fn bg_from_video(video: &Path) -> PathBuf {
    let video = PathBuf::from(video);
    println!("{:?}", video);
    println!("{:?}", video.file_name());
    let mut data_dir = dirs::data_local_dir().unwrap();
    data_dir.push("librepresenter");
    data_dir.push("thumbnails");
    if !data_dir.exists() {
        fs::create_dir(&data_dir)
            .expect("Could not create thumbnails dir");
    }
    let mut screenshot = data_dir.clone();
    screenshot.push(video.file_name().unwrap());
    screenshot.set_extension("png");
    if !screenshot.exists() {
        let output_duration = Command::new("ffprobe")
            .args(&["-i", &video.to_string_lossy()])
            .output()
            .expect("failed to execute ffprobe");
        io::stderr().write_all(&output_duration.stderr).unwrap();
        let mut at_second = 2;
        let mut log = str::from_utf8(&output_duration.stderr)
            .expect("Using non UTF-8 characters")
            .to_string();
        println!("{log}");
        if let Some(duration_index) = log.find("Duration") {
            let mut duration = log.split_off(duration_index + 10);
            duration.truncate(11);
            println!("rust-duration-is: {duration}");
            let mut hours = String::from("");
            let mut minutes = String::from("");
            let mut seconds = String::from("");
            for (i, c) in duration.chars().enumerate() {
                if i <= 1 {
                    hours.push(c);
                } else if i > 2 && i <= 4 {
                    minutes.push(c);
                } else if i > 5 && i <= 7 {
                    seconds.push(c);
                }
            }
            let hours: i32 = hours.parse().unwrap_or_default();
            let mut minutes: i32 =
                minutes.parse().unwrap_or_default();
            let mut seconds: i32 =
                seconds.parse().unwrap_or_default();
            minutes += hours * 60;
            seconds += minutes * 60;
            at_second = seconds / 5;
            println!(
                "hours: {}, minutes: {}, seconds: {}, at_second: {}",
                hours, minutes, seconds, at_second
            );
        }
        let _output = Command::new("ffmpeg")
            .args(&[
                "-i",
                &video.to_string_lossy(),
                "-ss",
                &at_second.to_string(),
                "-vframes",
                "1",
                "-y",
                &screenshot.to_string_lossy(),
            ])
            .output()
            .expect("failed to execute ffmpeg");
        // io::stdout().write_all(&output.stdout).unwrap();
        // io::stderr().write_all(&output.stderr).unwrap();
    } else {
        println!("Screenshot already exists");
    }
    screenshot
}
