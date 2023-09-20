// The purpose of this file is to provide validation
// of whether or not a file exists
#[cxx_qt::bridge]
mod file_helper {
    use cxx_qt_lib::QVariantValue;
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
            let _file_string = file_string.strip_prefix("file://");
            match _file_string {
                None => {
                    let _exists =
                        Path::new(&file.to_string()).exists();
                    println!("{file} exists? {_exists}");
                    _exists
                }
                Some(file) => {
                    let _exists = Path::new(&file).exists();
                    println!("{file} exists? {_exists}");
                    _exists
                }
            }
        }
    }
}
