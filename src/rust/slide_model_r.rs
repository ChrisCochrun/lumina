enum PresType {
    Html,
    Pdf,
}

enum SlideType {
    Song,
    Video,
    Image,
    Presentation(PresType),
    Content,
}

// /// An Image is a tuple struct with the string being a location on disk
// struct Image(String);

struct Slide {
    id: i32, // This is the inner index, could be 3 inside a presentation or image gallery
    kind: SlideType,
}

struct SlideModel {
    index: i32,
    slides: Vec<Slide>,
}
