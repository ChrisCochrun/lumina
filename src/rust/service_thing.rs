#[cxx_qt::bridge]
mod service_thing {
    unsafe extern "C++" {
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;
        include!("cxx-qt-lib/qvariant.h");
        type QVariant = cxx_qt_lib::QVariant;
    }

    unsafe extern "RustQt" {
        #[qobject]
        #[qml_element]
        #[qproperty(QString, name)]
        #[qproperty(QString, kind)]
        #[qproperty(QString, background)]
        #[qproperty(QString, background_type)]
        #[qproperty(QString, text)]
        #[qproperty(QString, audio)]
        #[qproperty(QString, font)]
        #[qproperty(QString, font_size)]
        #[qproperty(bool, active)]
        #[qproperty(bool, selected)]
        type ServiceThing = super::ServiceThingRust;

        #[qinvokable]
        fn activate(self: Pin<&mut ServiceThing>);

        #[qinvokable]
        fn check_active(self: Pin<&mut ServiceThing>);
    }
}

use cxx_qt_lib::QString;
use std::pin::Pin;

#[derive(Clone)]
pub struct ServiceThingRust {
    name: QString,
    kind: QString,
    background: QString,
    background_type: QString,
    text: QString,
    audio: QString,
    font: QString,
    font_size: QString,
    active: bool,
    selected: bool,
}

impl Default for ServiceThingRust {
    fn default() -> Self {
        Self {
            name: QString::from(""),
            kind: QString::from(""),
            background: QString::from(""),
            background_type: QString::from(""),
            text: QString::from(""),
            audio: QString::from(""),
            font: QString::from(""),
            font_size: QString::from(""),
            active: false,
            selected: false,
        }
    }
}

impl service_thing::ServiceThing {
    pub fn activate(self: Pin<&mut Self>) {
        println!("{}", self.active());
        let active: bool = *self.active();
        self.set_active(!active);
        println!("{}", !active);
    }

    pub fn check_active(self: Pin<&mut Self>) {
        println!("Are we active?: {}", self.active());
    }
}
