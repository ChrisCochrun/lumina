use cxx_qt_build::CxxQtBuilder;

fn main() {
    // CxxQtBuilder::new().file("src/rust/my_object.rs").build();
    CxxQtBuilder::new()
        .file("src/rust/service_thing.rs")
        .file("src/rust/file_helper.rs")
        .build();
}
