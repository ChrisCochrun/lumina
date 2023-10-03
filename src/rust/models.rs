use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;
use diesel_migrations::{
    embed_migrations, EmbeddedMigrations, MigrationHarness,
};

pub const MIGRATIONS: EmbeddedMigrations =
    embed_migrations!("src/rust/migrations");

pub fn run_migrations(conn: &mut SqliteConnection) -> bool {
    conn.run_pending_migrations(MIGRATIONS).unwrap();
    true
}

#[derive(Queryable)]
pub struct Image {
    pub id: i32,
    pub title: String,
    pub path: String,
}

#[derive(Queryable)]
pub struct Video {
    pub id: i32,
    pub title: String,
    pub path: String,
    pub start_time: Option<f32>,
    pub end_time: Option<f32>,
    pub looping: bool,
}

#[derive(Queryable)]
pub struct Presentation {
    pub id: i32,
    pub title: String,
    pub path: String,
    pub page_count: Option<i32>,
    pub html: bool,
}

// #[derive(Queryable)]
// pub struct Song {
//     pub id: i32,
//     pub title: String,
//     pub lyrics: Option<String>,
//     pub author: Option<String>,
//     pub ccli: Option<String>,
//     pub audio: Option<String>,
//     pub verse_order: Option<String>,
//     pub background: Option<String>,
//     pub background_type: Option<String>,
//     pub horizontal_text_alignment: Option<String>,
//     pub vertical_text_alignment: Option<String>,
//     pub font: Option<String>,
//     pub font_size: Option<i32>,
// }
