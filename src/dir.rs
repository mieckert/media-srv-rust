use crate::to_url::ToUrl;
use std::fs;
use std::path::Path;
use serde::Serialize;

const VIDEO_EXTENSIONS: &'static [&'static str] = &["mp3","mp4", "webm", "mpg", "mpeg", "mov", "3gp", "mkv", "m4v", "flv", "avi"];
const AUDIO_EXTENSIONS: &'static [&'static str] = &[];
const IMAGE_EXTENSIONS: &'static [&'static str] = &["jpg", "jpeg", "png", "gif", "bmp"];
const PDF_EXTENSIONS: &'static [&'static str] = &["pdf"];

pub const DIRECTORY_ICON: &'static str = "folder";
const VIDEO_ICON: &'static str = "file-earmark-play";
const AUDIO_ICON: &'static str = "file-earmark-music";
const IMAGE_ICON: &'static str = "file-earmark-image";
const PDF_ICON: &'static str = "file-earmark-text";
const OTHER_ICON: &'static str = "file-earmark";

const DIRECTORY_PREFIX: &'static str = "dir";
const WATCH_PREFIX: &'static str = "watch";
const LISTEN_PREFIX: &'static str = "listen";
const VIEW_PREFIX: &'static str = "view";
const READ_PREFIX: &'static str = "read";
const FILES_PREFIX: &'static str = "files";

#[derive(Serialize)]

pub enum FileType {
    Directory,
    Video,
    Audio,
    Image,
    Pdf,
    Other,
}

#[derive(Serialize)]
pub struct Entry {
    pub name: String,
    pub link_url: String,
    pub download_url: String,
    pub filetype: FileType,
    pub icon: &'static str,
}

pub fn read_dir<D: AsRef<Path>, B: AsRef<Path>>(dir: D, base_path: B) -> Result<Vec<Entry>, std::io::Error> {
    let mut entries: Vec<Entry> = vec![];
    fs::read_dir(dir)?
        .flat_map(|entry| {
            if let Err(e) = &entry {
                println!("Error reading dir entry: {:?}", e);
            }
            entry
        })
        .filter(|entry: &fs::DirEntry| !entry.file_name().to_string_lossy().starts_with('.'))
        .for_each(|entry: fs::DirEntry| match entry.file_type() {
            Ok(file_type) if file_type.is_dir() => {
                let name = entry.file_name().to_string_lossy().into_owned();

                entries.push(Entry {
                    name: name.clone(),
                    link_url: format!(
                        "/{}/{}/{}",
                        DIRECTORY_PREFIX,
                        base_path.to_url(),
                        name.to_url()
                    ),
                    download_url: format!(
                        "/{}/{}/{}",
                        DIRECTORY_PREFIX,
                        base_path.to_url(),
                        name.to_url()
                    ),
                    filetype: FileType::Directory,
                    icon: DIRECTORY_ICON,
                })
            }
            Ok(file_type) if file_type.is_file() => {
                let name = entry.file_name().to_string_lossy().into_owned();
                let ext = name
                    .rsplit('.')
                    .next()
                    .unwrap_or_default()
                    .to_ascii_lowercase();

                let (filetype, icon, link_prefix, download_prefix) =
                    if VIDEO_EXTENSIONS.contains(&ext.as_ref()) {
                        (FileType::Video, VIDEO_ICON, WATCH_PREFIX, FILES_PREFIX)
                    } else if AUDIO_EXTENSIONS.contains(&ext.as_ref()) {
                        (FileType::Audio, AUDIO_ICON, LISTEN_PREFIX, FILES_PREFIX)
                    } else if IMAGE_EXTENSIONS.contains(&ext.as_ref()) {
                        (FileType::Image, IMAGE_ICON, VIEW_PREFIX, FILES_PREFIX)
                    } else if PDF_EXTENSIONS.contains(&ext.as_ref()) {
                        (FileType::Pdf, PDF_ICON, READ_PREFIX, FILES_PREFIX)
                    } else {
                        (FileType::Other, OTHER_ICON, FILES_PREFIX, FILES_PREFIX)
                    };

                entries.push(Entry {
                    name: name.clone(),
                    link_url: format!(
                        "/{}/{}/{}",
                        link_prefix,
                        base_path.to_url(),
                        name.to_url()
                    ),
                    download_url: format!(
                        "/{}/{}/{}",
                        download_prefix,
                        base_path.to_url(),
                        name.to_url()
                    ),
                    filetype,
                    icon,
                })
            }
            Ok(_) => {
                println!("Ignoring symlink: {}", &entry.file_name().to_string_lossy());
            }
            Err(e) => {
                println!(
                    "Error getting file type for dir entry {}: {:?}",
                    &entry.file_name().to_string_lossy(),
                    e
                );
            }
        });

    Ok(entries)
}
