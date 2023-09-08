#[cxx_qt::bridge]
mod settings {

    use configparser::ini::Ini;
    use dirs;

    unsafe extern "C++" {
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;
        include!("cxx-qt-lib/qurl.h");
        type QUrl = cxx_qt_lib::QUrl;
    }

    #[derive(Clone)]
    #[cxx_qt::qobject]
    pub struct Settings {
        #[qproperty]
        screen: QString,
        #[qproperty]
        sound_effect: QString,
        #[qproperty]
        last_save_file: QUrl,
    }

    impl Default for Settings {
        fn default() -> Self {
            Self {
                screen: QString::from(""),
                sound_effect: QString::from(""),
                last_save_file: QUrl::from(""),
            }
        }
    }

    impl qobject::Settings {
        #[qinvokable]
        pub fn print_sound(self: Pin<&mut Self>) {
            let mut config = Ini::new();
            let _map = config.load("~/.config/lumina/lumina.conf");

            println!("{}", self.sound_effect());
        }

        #[qinvokable]
        pub fn setup(self: Pin<&mut Self>) {
            let mut config = Ini::new();
            let home = dirs::config_dir();
            println!("{:?}", home);
            if let Some(mut conf) = home {
                conf.push("lumina");
                conf.push("lumina.conf");
                let _map = config.load(conf);

                println!("{:?}", config);
                println!("{:?}", _map);
                let sf = config.get("General", "lastSaveFile");
                println!("{:?}", sf);
                if let Some(s) = sf {
                    self.set_last_save_file(QUrl::from(&s));
                    println!("{s}");
                } else {
                    println!("error loading last save file");
                }
            } else {
                println!("Couldn't find home directory");
            }
        }
    }
}
