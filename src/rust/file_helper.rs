// The purpose of this file is to provide validation
// of whether or not a file exists
#[cxx_qt::bridge]
mod file_helper {
    use cxx_qt_lib::QVariantValue;
    use rfd::FileDialog;
    use std::path::Path;

    unsafe extern "C++" {
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;
        include!("cxx-qt-lib/qurl.h");
        type QUrl = cxx_qt_lib::QUrl;
        include!("cxx-qt-lib/qvariant.h");
        type QVariant = cxx_qt_lib::QVariant;
    }

    #[derive(Clone)]
    #[cxx_qt::qobject]
    pub struct FileHelper {
        #[qproperty]
        name: QString,
        #[qproperty]
        file_path: QString,
    }

    impl Default for FileHelper {
        fn default() -> Self {
            Self {
                name: QString::from(""),
                file_path: QString::from(""),
            }
        }
    }

    impl qobject::FileHelper {
        // #[qinvokable]
        // pub fn save(self: Pin<&mut Self>, file: QUrl, service_list: QVariant) -> bool {
        //     println!("{}", file);
        //     match service_list.value() {
        //         QVariantValue::<QString>(..) => println!("string"),
        //         QVariantValue::<QUrl>(..) => println!("url"),
        //         QVariantValue::<QDate>(..) => println!("date"),
        //         _ => println!("QVariant is..."),
        //     }
        //     return true;
        // }

        #[qinvokable]
        pub fn load(self: Pin<&mut Self>, file: QUrl) -> Vec<String> {
            println!("{file}");
            vec!["hi".to_string()]
        }

        #[qinvokable]
        pub fn validate(self: Pin<&mut Self>, file: QUrl) -> bool {
            let file_string = file.to_string();
            let file_string = file_string.strip_prefix("file://");
            match file_string {
                Some(file) => {
                    let exists = Path::new(&file).exists();
                    println!("{file} exists? {exists}");
                    exists
                }
                None => {
                    let exists =
                        Path::new(&file.to_string()).exists();
                    println!("{file} exists? {exists}");
                    exists
                }
            }
        }

        #[qinvokable]
        pub fn save_file(self: Pin<&mut Self>) -> QUrl {
            let file = FileDialog::new()
                .set_file_name("NVTFC.pres")
                .set_title("Save Presentation")
                .save_file();
            if let Some(file) = file {
                println!("saving-file: {:?}", file);
                let mut string =
                    String::from(file.to_str().unwrap_or(""));
                if string.is_empty() {
                    QUrl::default()
                } else {
                    string.insert_str(0, "file://");
                    QUrl::from(string.as_str())
                }
            } else {
                QUrl::default()
            }
        }

        #[qinvokable]
        pub fn load_file(self: Pin<&mut Self>) -> QUrl {
            let file = FileDialog::new()
                .set_title("Load Presentation")
                .pick_file();
            if let Some(file) = file {
                println!("loading-file: {:?}", file);
                let mut string =
                    String::from(file.to_str().unwrap_or(""));
                if string.is_empty() {
                    QUrl::default()
                } else {
                    string.insert_str(0, "file://");
                    QUrl::from(string.as_str())
                }
            } else {
                QUrl::default()
            }
        }
    }
}
