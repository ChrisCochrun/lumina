#[cxx_qt::bridge]
mod ytdl {
    use dirs;
    use std::{fs, thread};
    use youtube_dl::YoutubeDl;

    unsafe extern "C++" {
        include!("cxx-qt-lib/qurl.h");
        type QUrl = cxx_qt_lib::QUrl;
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;
    }

    #[derive(Clone, Default)]
    #[cxx_qt::qobject]
    pub struct Ytdl {
        #[qproperty]
        title: QString,
        #[qproperty]
        thumbnail: QUrl,
        #[qproperty]
        loaded: bool,
        #[qproperty]
        loading: bool,
        #[qproperty]
        file: QUrl,
    }

    impl qobject::Ytdl {
        #[qinvokable]
        pub fn get_video(
            mut self: Pin<&mut Self>,
            url: QUrl,
        ) -> bool {
            if !url.is_valid() {
                false
            } else {
                let data_dir = dirs::data_local_dir().unwrap();
                if let Some(mut data_dir) = dirs::data_local_dir() {
                    data_dir.push("librepresenter");
                    data_dir.push("ytdl");
                    if !data_dir.exists() {
                        fs::create_dir(&data_dir)
                            .expect("Could not create ytdl dir");
                    }
                    println!("{:?}", data_dir);
                    self.as_mut().set_loading(true);
                    let thread = self.qt_thread();
                    thread::spawn(move || {
                        let url = url.to_string();
                        let output_dirs = data_dir.to_str().unwrap();
                        println!("{output_dirs}");
                        let ytdl = YoutubeDl::new(url)
                            .socket_timeout("15")
                            .output_directory(output_dirs)
                            .output_template("%(title)s.%(ext)s")
                            .download(true)
                            .run()
                            .unwrap();
                        let output =
                            ytdl.into_single_video().unwrap();
                        println!("{:?}", output.title);
                        println!("{:?}", output.thumbnail);
                        println!("{:?}", output.url);
                        let title = QString::from(&output.title);
                        let thumbnail = QUrl::from(
                            &output.thumbnail.unwrap_or_default(),
                        );
                        let mut file = String::from(output_dirs);
                        file.push_str("/");
                        file.push_str(&output.title);
                        file.push_str(".");
                        file.push_str(
                            &output.ext.unwrap_or_default(),
                        );
                        println!("{:?}", file);

                        thread.queue(move |mut qobject_ytdl| {
                            qobject_ytdl.as_mut().set_loaded(true);
                            qobject_ytdl.as_mut().set_loading(false);
                            qobject_ytdl.as_mut().set_title(title);
                            qobject_ytdl
                                .as_mut()
                                .set_thumbnail(thumbnail);
                            qobject_ytdl
                                .as_mut()
                                .set_file(QUrl::from(&file));
                        })
                    });
                    true
                } else {
                    false
                }
            }
        }
    }
}
