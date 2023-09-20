// @generated automatically by Diesel CLI.

diesel::table! {
    images (id) {
        id -> Integer,
        title -> Text,
        #[sql_name = "filePath"]
        path -> Text,
    }
}

diesel::table! {
    presentations (id) {
        id -> Integer,
        title -> Text,
        #[sql_name = "filePath"]
        path -> Text,
        #[sql_name = "pageCount"]
        page_count -> Nullable<Integer>,
        html -> Bool,
    }
}

diesel::table! {
    songs (id) {
        id -> Integer,
        title -> Text,
        lyrics -> Nullable<Text>,
        author -> Nullable<Text>,
        ccli -> Nullable<Text>,
        audio -> Nullable<Text>,
        #[sql_name = "vorder"]
        verse_order -> Nullable<Text>,
        background -> Nullable<Text>,
        #[sql_name = "backgroundType"]
        background_type -> Nullable<Text>,
        #[sql_name = "horizontalTextAlignment"]
        horizontal_text_alignment -> Nullable<Text>,
        #[sql_name = "verticalTextAlignment"]
        vertical_text_alignment -> Nullable<Text>,
        font -> Nullable<Text>,
        #[sql_name = "fontSize"]
        font_size -> Nullable<Integer>,
    }
}

diesel::table! {
    videos (id) {
        id -> Integer,
        title -> Text,
        #[sql_name = "filePath"]
        path -> Text,
        #[sql_name = "startTime"]
        start_time -> Nullable<Float>,
        #[sql_name = "endTime"]
        end_time -> Nullable<Float>,
        #[sql_name = "loop"]
        looping -> Bool,
    }
}

diesel::allow_tables_to_appear_in_same_query!(
    images,
    presentations,
    songs,
    videos,
);
