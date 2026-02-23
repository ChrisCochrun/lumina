use crate::core::{
    kinds::ServiceItemKind, service_items::ServiceItem,
    slide::Background,
};
use cosmic::widget::image::Handle;
use miette::{IntoDiagnostic, Result, miette};
use std::{
    fs::{self, File},
    io::Write,
    iter,
    path::{Path, PathBuf},
};
use tar::{Archive, Builder};
use tracing::{debug, error};
use zstd::{Decoder, Encoder};

#[allow(clippy::too_many_lines)]
pub fn save(
    list: Vec<ServiceItem>,
    path: impl AsRef<Path>,
    overwrite: bool,
) -> Result<()> {
    let path = path.as_ref();
    if overwrite && path.exists() {
        fs::remove_file(path).into_diagnostic()?;
    }
    let save_file = File::create(path).into_diagnostic()?;
    let ron_pretty = ron::ser::PrettyConfig::default();
    let ron = ron::ser::to_string_pretty(&list, ron_pretty)
        .into_diagnostic()?;

    let encoder = Encoder::new(save_file, 3)
        .expect("file encoder shouldn't fail")
        .auto_finish();
    let mut tar = Builder::new(encoder);
    let mut temp_dir = dirs::data_dir().expect(
        "there should be a data directory, ~/.local/share/ for linux, but couldn't find it",
    );
    temp_dir.push("lumina");
    let mut s: String =
        iter::repeat_with(fastrand::alphanumeric).take(5).collect();
    s.insert_str(0, "temp_");
    temp_dir.push(s);
    fs::create_dir_all(&temp_dir).into_diagnostic()?;
    let service_file = temp_dir.join("serviceitems.ron");
    debug!(?service_file);
    fs::File::create(&service_file).into_diagnostic()?;
    match fs::File::options()
        .read(true)
        .write(true)
        .open(service_file)
    {
        Ok(mut f) => {
            match f.write(ron.as_bytes()) {
                Ok(size) => {
                    debug!(size);
                }
                Err(e) => {
                    error!(?e);
                    return Err(miette!("PROBS: {e}"));
                }
            }
            match tar.append_file("serviceitems.ron", &mut f) {
                Ok(()) => {
                    debug!(
                        "should have added serviceitems.ron to the file"
                    );
                }
                Err(e) => {
                    error!(?e);
                    return Err(miette!("PROBS: {e}"));
                }
            }
        }
        Err(e) => {
            error!("There were problems making a file i guess: {e}");
            return Err(miette!("There was a problem: {e}"));
        }
    }

    let mut append_file = |path: PathBuf| -> Result<()> {
        let file_name = path.file_name().unwrap_or_default();
        let mut file = fs::File::open(&path).into_diagnostic()?;
        tar.append_file(file_name, &mut file).into_diagnostic()?;
        Ok(())
    };

    for item in list {
        let background;
        let audio: Option<PathBuf>;
        match &item.kind {
            ServiceItemKind::Song(song) => {
                background = song.background.clone();
                audio = song.audio.clone();
            }
            ServiceItemKind::Image(image) => {
                background = Some(
                    Background::try_from(image.path.clone())
                        .into_diagnostic()?,
                );
                audio = None;
            }
            ServiceItemKind::Video(video) => {
                background = Some(
                    Background::try_from(video.path.clone())
                        .into_diagnostic()?,
                );
                audio = None;
            }
            ServiceItemKind::Presentation(presentation) => {
                background = Some(
                    Background::try_from(presentation.path.clone())
                        .into_diagnostic()?,
                );
                audio = None;
            }
            ServiceItemKind::Content(_slide) => {
                todo!()
            }
        }
        if let Some(path) = audio
            && path.exists()
        {
            debug!(?path);
            append_file(path)?;
        }
        if let Some(background) = background
            && let path = background.path
            && path.exists()
        {
            debug!(?path);
            append_file(path)?;
        }
        for slide in item.slides {
            if let Some(svg) = slide.text_svg
                && let Some(path) = svg.path
            {
                append_file(path)?;
            }
        }
    }

    match tar.finish() {
        Ok(()) => (),
        Err(e) => {
            error!(?e);
            return Err(miette!("tar error: {e}"));
        }
    }
    fs::remove_dir_all(temp_dir).into_diagnostic()
}

#[allow(clippy::too_many_lines)]
pub fn load(path: impl AsRef<Path>) -> Result<Vec<ServiceItem>> {
    let decoder =
        Decoder::new(fs::File::open(&path).into_diagnostic()?)
            .into_diagnostic()?;
    let mut tar = Archive::new(decoder);

    let mut cache_dir =
        dirs::cache_dir().expect("Should be a cache dir");
    cache_dir.push("lumina");
    cache_dir.push("cached_save_files");

    let save_name_ext = path
        .as_ref()
        .extension()
        .expect("Should have extension")
        .to_str()
        .expect("Should be fine");
    let save_name_string = path
        .as_ref()
        .file_name()
        .expect("Should be a name")
        .to_os_string()
        .into_string()
        .expect("Should be fine");
    let save_name = save_name_string
        .trim_end_matches(&format!(".{save_name_ext}"));
    cache_dir.push(save_name);

    if let Err(e) = fs::remove_dir_all(&cache_dir) {
        debug!("There is no dir here: {e}");
    }
    fs::create_dir_all(&cache_dir).into_diagnostic()?;

    for entry in tar.entries().into_diagnostic()? {
        let mut entry = entry.into_diagnostic()?;
        entry.unpack_in(&cache_dir).into_diagnostic()?;
    }

    let mut dir = fs::read_dir(&cache_dir).into_diagnostic()?;
    let ron_file = dir
        .find_map(|file| {
            if file.as_ref().ok()?.path().extension()?.to_str()?
                == "ron"
            {
                Some(file.ok()?.path())
            } else {
                None
            }
        })
        .expect("Should have a ron file");

    let ron_string =
        fs::read_to_string(ron_file).into_diagnostic()?;

    let mut items =
        ron::de::from_str::<Vec<ServiceItem>>(&ron_string)
            .into_diagnostic()?;

    for item in &mut items {
        let dir = fs::read_dir(&cache_dir).into_diagnostic()?;
        for file in dir {
            for slide in &mut item.slides {
                if let Ok(file) = file.as_ref() {
                    let file_name = file.file_name();
                    let audio_path =
                        slide.audio().clone().unwrap_or_default();
                    let text_path = slide
                        .text_svg
                        .as_ref()
                        .and_then(|svg| svg.path.clone());
                    if Some(file_name.as_os_str())
                        == slide.background.path.file_name()
                    {
                        slide.background.path = file.path();
                    } else if Some(file_name.as_os_str())
                        == audio_path.file_name()
                    {
                        let new_slide = slide
                            .clone()
                            .set_audio(Some(file.path()));
                        *slide = new_slide;
                    } else if Some(file_name.as_os_str())
                        == text_path
                            .clone()
                            .unwrap_or_default()
                            .file_name()
                        && let Some(svg) = slide.text_svg.as_mut()
                    {
                        svg.path = Some(file.path());
                        svg.handle =
                            Some(Handle::from_path(file.path()));
                    }
                }
            }

            match &mut item.kind {
                ServiceItemKind::Song(song) => {
                    if let Ok(file) = file.as_ref() {
                        let file_name = file.file_name();
                        let audio_path =
                            song.audio.clone().unwrap_or_default();
                        if Some(file_name.as_os_str())
                            == song
                                .background
                                .clone()
                                .unwrap_or_default()
                                .path
                                .file_name()
                        {
                            let background = song.background.clone();
                            song.background =
                                background.map(|mut background| {
                                    background.path = file.path();
                                    background
                                });
                        } else if Some(file_name.as_os_str())
                            == audio_path.file_name()
                        {
                            song.audio = Some(file.path());
                        }
                    }
                }
                ServiceItemKind::Video(video) => {
                    if let Ok(file) = file.as_ref() {
                        let file_name = file.file_name();
                        if Some(file_name.as_os_str())
                            == video.path.file_name()
                        {
                            video.path = file.path();
                        }
                    }
                }
                ServiceItemKind::Image(image) => {
                    if let Ok(file) = file.as_ref() {
                        let file_name = file.file_name();
                        if Some(file_name.as_os_str())
                            == image.path.file_name()
                        {
                            image.path = file.path();
                        }
                    }
                }
                ServiceItemKind::Presentation(presentation) => {
                    if let Ok(file) = file.as_ref() {
                        let file_name = file.file_name();
                        if Some(file_name.as_os_str())
                            == presentation.path.file_name()
                        {
                            presentation.path = file.path();
                        }
                    }
                }
                ServiceItemKind::Content(_slide) => todo!(),
            }
        }
    }
    Ok(items)
}

#[cfg(test)]
mod test {
    use rayon::iter::{IntoParallelIterator, ParallelIterator};
    use resvg::usvg::fontdb;

    use super::*;
    use crate::{
        core::{
            service_items::ServiceTrait,
            slide::{Slide, TextAlignment},
            songs::{Song, VerseName},
        },
        ui::text_svg::text_svg_generator,
    };
    use std::{collections::HashMap, path::PathBuf, sync::Arc};

    fn test_song() -> Song {
        let lyrics = "Some({Verse(number:4):\"Our Savior displayed\\nOn a criminal\\'s cross\\n\\nDarkness rejoiced as though\\nHeaven had lost\\n\\nBut then Jesus arose\\nWith our freedom in hand\\n\\nThat\\'s when death was arrested\\nAnd my life began\\n\\nThat\\'s when death was arrested\\nAnd my life began\",Intro(number:1):\"Death Was Arrested\\nNorth Point Worship\",Verse(number:3):\"Released from my chains,\\nI\\'m a prisoner no more\\n\\nMy shame was a ransom\\nHe faithfully bore\\n\\nHe cancelled my debt and\\nHe called me His friend\\n\\nWhen death was arrested\\nAnd my life began\",Bridge(number:1):\"Oh, we\\'re free, free,\\nForever we\\'re free\\n\\nCome join the song\\nOf all the redeemed\\n\\nYes, we\\'re free, free,\\nForever amen\\n\\nWhen death was arrested\\nAnd my life began\\n\\nOh, we\\'re free, free,\\nForever we\\'re free\\n\\nCome join the song\\nOf all the redeemed\\n\\nYes, we\\'re free, free,\\nForever amen\\n\\nWhen death was arrested\\nAnd my life began\",Other(number:99):\"When death was arrested\\nAnd my life began\\n\\nThat\\'s when death was arrested\\nAnd my life began\",Verse(number:2):\"Ash was redeemed\\nOnly beauty remains\\n\\nMy orphan heart\\nWas given a name\\n\\nMy mourning grew quiet,\\nMy feet rose to dance\\n\\nWhen death was arrested\\nAnd my life began\",Verse(number:1):\"Alone in my sorrow\\nAnd dead in my sin\\n\\nLost without hope\\nWith no place to begin\\n\\nYour love made a way\\nTo let mercy come in\\n\\nWhen death was arrested\\nAnd my life began\",Chorus(number:1):\"Oh, Your grace so free,\\nWashes over me\\n\\nYou have made me new,\\nNow life begins with You\\n\\nIt\\'s Your endless love,\\nPouring down on us\\n\\nYou have made us new,\\nNow life begins with You\"})".to_string();
        let verse_map: Option<HashMap<VerseName, String>> =
            ron::from_str(&lyrics).unwrap();
        Song {
            id: 7,
            title: "Death Was Arrested".to_string(),
            lyrics: Some(lyrics),
            author: Some(
                "North Point Worship".to_string(),
            ),
            ccli: None,
            audio: Some("/home/chris/music/North Point InsideOut/Nothing Ordinary, Pt. 1 (Live)/05 Death Was Arrested (feat. Seth Condrey).mp3".into()),
            verse_order: Some(vec!["Some([Chorus(number:1),Intro(number:1),Other(number:99),Bridge(number:1),Verse(number:4),Verse(number:2),Verse(number:3),Verse(number:1)])".to_string()]),
            background: Some(Background::try_from("/home/chris/nc/tfc/openlp/Flood/motions/Ocean_Floor_HD.mp4").unwrap()),
            text_alignment: Some(TextAlignment::MiddleCenter),
            font: None,
            font_size: Some(120),
            font_style: None,
            font_weight: None,
            text_color: None,
            stroke_size: None,
            verses: Some(vec![VerseName::Chorus { number: 1 }, VerseName::Intro { number: 1 }, VerseName::Other { number: 99 }, VerseName::Bridge { number: 1 }, VerseName::Verse { number: 4 }, VerseName::Verse { number: 2 }, VerseName::Verse { number: 3 }, VerseName::Verse { number: 1 }
            ]),
            verse_map,
            ..Default::default()
        }
    }

    fn get_items() -> Vec<ServiceItem> {
        let song = test_song();
        let mut fontdb = fontdb::Database::new();
        fontdb.load_system_fonts();
        let fontdb = Arc::new(fontdb);
        let slides = song
            .to_slides()
            .unwrap()
            .into_par_iter()
            .map(|slide| {
                text_svg_generator(
                    slide.clone(),
                    &Arc::clone(&fontdb),
                )
                .map_or_else(
                    |e| {
                        assert!(false, "Couldn't create svg: {e}");
                        slide
                    },
                    |slide| slide,
                )
            })
            .collect::<Vec<Slide>>();
        let items = vec![
            ServiceItem {
                database_id: 7,
                kind: ServiceItemKind::Song(song.clone()),
                id: 0,
                title: "Death was Arrested".into(),
                slides: slides.clone(),
            },
            ServiceItem {
                database_id: 7,
                kind: ServiceItemKind::Song(song),
                id: 1,
                title: "Death was Arrested".into(),
                slides: slides,
            },
        ];
        items
    }

    #[test]
    fn test_load() -> Result<(), String> {
        test_save();
        let path = PathBuf::from("./test.pres");
        let result = load(&path);
        match result {
            Ok(items) => {
                assert!(items.len() > 0);
                // assert_eq!(items, get_items());
                let cache_dir = cache_dir();
                assert!(fs::read_dir(&cache_dir).is_ok());
                assert!(
                    find_paths(&items),
                    "Some paths must not have the cache_dir in it's path"
                );
                find_svgs(&items)?;
                Ok(())
            }
            Err(e) => Err(e.to_string()),
        }
    }

    fn find_svgs(items: &Vec<ServiceItem>) -> Result<(), String> {
        let cache_dir = cache_dir();
        items.iter().try_for_each(|item| {
            if let ServiceItemKind::Song(..) = item.kind {
                item.slides.iter().try_for_each(|slide| {
                    slide.text_svg.as_ref().map_or(Err(String::from("There is no TextSvg for this song")), |text_svg| {

                        if text_svg.handle.is_none() {
                            return Err(String::from("There is no handle in this song's TextSvg"));
                        };

                        text_svg.path.as_ref().map_or(Err(String::from("There is no path in this song's TextSvg")), |path| {
                            if path.exists() {
                                let mut path = path.clone();
                                if path.metadata().unwrap().len() < 20000 {
                                    return Err(String::from("SVG text is too small, maybe the svg didn't generate properly"))
                                }
                                if path.pop() && path == cache_dir {
                                    Ok(())
                                } else {
                                    Err(String::from("The path of the TextSvg isn't in the load directory"))
                                }
                            } else {
                                Err(String::from("The path in this TextSvg doesn't exist"))
                            }
                        })
                    })
                })
            } else {
                Ok(())
            }
        })
    }

    // checks to make sure all paths in slides and items point to cache_dir
    fn find_paths(items: &Vec<ServiceItem>) -> bool {
        let cache_dir = cache_dir();
        items.iter().all(|item| {
            match &item.kind {
                ServiceItemKind::Song(song) => {
                    if let Some(bg) = &song.background {
                        if !bg.path.starts_with(&cache_dir) {
                            return false;
                        }
                    }
                    if let Some(audio) = &song.audio {
                        if !audio.starts_with(&cache_dir) {
                            return false;
                        }
                    }
                }
                ServiceItemKind::Video(video) => {
                    if !video.path.starts_with(&cache_dir) {
                        return false;
                    }
                }
                ServiceItemKind::Image(image) => {
                    if !image.path.starts_with(&cache_dir) {
                        return false;
                    }
                }
                ServiceItemKind::Presentation(presentation) => {
                    if !presentation.path.starts_with(&cache_dir) {
                        return false;
                    }
                }
                ServiceItemKind::Content(_slide) => todo!(),
            }
            for slide in &item.slides {
                if !slide.background().path.starts_with(&cache_dir) {
                    return false;
                }
                if !slide.audio().map_or(true, |audio| {
                    audio.starts_with(&cache_dir)
                }) {
                    return false;
                }
            }
            true
        })
    }

    fn cache_dir() -> PathBuf {
        let mut cache_dir = dirs::cache_dir().unwrap();
        cache_dir.push("lumina");
        cache_dir.push("cached_save_files");
        cache_dir.push("test");
        cache_dir
    }

    #[test]
    fn test_save() {
        let path = PathBuf::from("./test.pres");
        let list = get_items();
        match save(list, &path, true) {
            Ok(_) => {
                assert!(path.is_file());
                let Ok(file) = fs::File::open(path) else {
                    return assert!(false, "couldn't open file");
                };
                let Ok(size) = file.metadata().map(|data| data.len())
                else {
                    return assert!(
                        false,
                        "couldn't get file metadata"
                    );
                };
                assert!(size > 0);
            }
            Err(e) => assert!(false, "{e}"),
        }
    }
}
