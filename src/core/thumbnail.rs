use dirs;
use std::error::Error;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::str;
use tracing::debug;

pub fn bg_from_video(
    video: &Path,
    screenshot: &Path,
) -> Result<(), Box<dyn Error>> {
    if !screenshot.exists() {
        let output_duration = Command::new("ffprobe")
            .args(["-i", &video.to_string_lossy()])
            .output()
            .expect("failed to execute ffprobe");
        io::stderr().write_all(&output_duration.stderr).unwrap();
        let mut at_second = 5;
        let mut log = str::from_utf8(&output_duration.stderr)
            .expect("Using non UTF-8 characters")
            .to_string();
        debug!(log);
        if let Some(duration_index) = log.find("Duration") {
            let mut duration = log.split_off(duration_index + 10);
            duration.truncate(11);
            // debug!("rust-duration-is: {duration}");
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
            debug!(hours, minutes, seconds, at_second);
        }
        let _output = Command::new("ffmpeg")
            .args([
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
        debug!("Screenshot already exists");
    }
    Ok(())
}

pub fn bg_path_from_video(video: &Path) -> PathBuf {
    let video = PathBuf::from(video);
    debug!(?video);
    let mut data_dir = dirs::data_local_dir().unwrap();
    data_dir.push("lumina");
    data_dir.push("thumbnails");
    if !data_dir.exists() {
        fs::create_dir(&data_dir)
            .expect("Could not create thumbnails dir");
    }
    let mut screenshot = data_dir.clone();
    screenshot.push(video.file_name().unwrap());
    screenshot.set_extension("png");
    screenshot
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_bg_video_creation() {
        let video = Path::new("/home/chris/vids/moms-funeral.mp4");
        let screenshot = bg_path_from_video(video);
        let screenshot_string =
            screenshot.to_str().expect("Should be thing");
        assert_eq!(screenshot_string, "/home/chris/.local/share/lumina/thumbnails/moms-funeral.png");

        // let runtime = tokio::runtime::Runtime::new().unwrap();
        let result = bg_from_video(video, &screenshot);
        // let result = runtime.block_on(future);
        match result {
            Ok(_o) => assert!(screenshot.exists()),
            Err(e) => debug_assert!(
                false,
                "There was an error in the runtime future. {:?}",
                e
            ),
        }
    }

    #[test]
    fn test_bg_not_same() {
        let video = Path::new(
            "/home/chris/vids/All WebDev Sucks and you know it.webm",
        );
        let screenshot = bg_path_from_video(video);
        let screenshot_string =
            screenshot.to_str().expect("Should be thing");
        assert_ne!(screenshot_string, "/home/chris/.local/share/lumina/thumbnails/All WebDev Sucks and you know it.webm");
    }
}
