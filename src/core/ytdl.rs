use std::path::PathBuf;

use youtube_dl::YoutubeDl;

pub async fn download_video(
    url: impl Into<String>,
    mut output_directory: PathBuf,
) -> Result<PathBuf, youtube_dl::Error> {
    YoutubeDl::new(url)
        .output_directory(output_directory.to_string_lossy())
        .output_template("%(title).%(ext)s")
        .run_async()
        .await
        .map(|output| {
            if let Some(video) = output.into_single_video() {
                let video_path = format!(
                    "{}.{}",
                    video.title.expect("Should be a title"),
                    video.ext.expect("Should be an extension")
                );
                output_directory.push(video_path);
            };
            output_directory
        })
}
