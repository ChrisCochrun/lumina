#[cxx_qt::bridge]
mod settings {
    unsafe extern "C++" {
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;
        include!("cxx-qt-lib/qurl.h");
        type QUrl = cxx_qt_lib::QUrl;
    }

    unsafe extern "RustQt" {
        #[qobject]
        #[qml_element]
        #[qproperty(QString, screen)]
        #[qproperty(QString, sound_effect)]
        #[qproperty(QUrl, last_save_file)]
        #[qproperty(QUrl, loaded_file)]
        #[qproperty(bool, debug)]
        type Settings = super::SettingsRust;

        #[qinvokable]
        fn setup(self: Pin<&mut Settings>);

        #[qinvokable]
        fn set_save_file(self: Pin<&mut Settings>, file: QUrl);
    }
}

use configparser::ini::Ini;
use cxx_qt::CxxQtType;
use cxx_qt_lib::{QString, QUrl};
use dirs;
use std::{path::PathBuf, pin::Pin};

// In order for settings to save to the ini file,
// I'll need to create my own setting functions I think.
#[derive(Clone)]
pub struct SettingsRust {
    config: Ini,

    screen: QString,
    sound_effect: QString,
    last_save_file: QUrl,
    loaded_file: QUrl,
    debug: bool,
}

impl Default for SettingsRust {
    fn default() -> Self {
        Self {
            config: Ini::new(),
            screen: QString::from(""),
            sound_effect: QString::from(""),
            last_save_file: QUrl::from(""),
            loaded_file: QUrl::from(""),
            debug: false,
        }
    }
}

impl settings::Settings {
    pub fn setup(mut self: Pin<&mut Self>) {
        let home = dirs::config_dir();
        println!("{:?}", home);
        if let Some(mut conf) = home {
            conf.push("lumina");
            conf.push("lumina.conf");
            match self.as_mut().rust_mut().config.load(conf) {
                Ok(map) => {
                    // println!("{:?}", self.rust().config);
                    let sf = self
                        .as_ref()
                        .config
                        .get("General", "lastSaveFile");
                    println!("{:?}", sf);
                    if let Some(s) = sf {
                        self.as_mut()
                            .set_last_save_file(QUrl::from(&s));
                        self.as_mut().set_loaded_file(QUrl::from(&s));
                        println!("{s}");
                    } else {
                        println!("error loading last save file");
                    }
                }
                Err(e) => {
                    println!("settings_load_error: {:?}", e)
                }
            }
        } else {
            println!("Couldn't find home directory");
        }
    }

    pub fn set_save_file(mut self: Pin<&mut Self>, file: QUrl) {
        println!("{file}");
        match self.as_mut().rust_mut().config.set(
            "General",
            "lastSaveFile",
            Some(file.to_string()),
        ) {
            Some(s) => {
                println!(
                    "set-save-file: {:?}",
                    self.as_mut()
                        .config
                        .get("General", "lastSaveFile")
                );
                if let Err(e) = self.as_mut().write() {
                    println!("error: {:?}", e)
                }
                self.set_last_save_file(file);
            }
            _ => println!("error-setting-save-file"),
        }
    }

    pub fn write(mut self: Pin<&mut Self>) -> std::io::Result<&str> {
        let mut file = dirs::config_dir().unwrap();
        file.push("lumina");
        file.push("lumina.conf");
        match self.as_mut().config.write(file) {
            Ok(_s) => Ok("Saved File"),
            Err(e) => {
                println!("error: {:?}", e);
                Err(e)
            }
        }
    }
}
