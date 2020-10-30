use std::path::Path;
use std::fs;

const VIDEO_EXTENSIONS: &'static [&'static str] = &["mp4"];

pub fn read_dir<D: AsRef<Path>>(dir: D) -> Result<(Vec<String>, Vec<String>), std::io::Error> {
    let mut subdirs: Vec<String> = vec![];
    let mut files: Vec<String> = vec![];
    
    fs::read_dir(dir)?
    .flat_map(|entry| {
        if let Err(e) = &entry {
            println!("Error reading dir entry: {:?}", e);
        }
        entry
    })
    .filter(|entry: &fs::DirEntry| {
        !entry.file_name().to_string_lossy().starts_with('.')
    })
    .for_each(|entry: fs::DirEntry| {        
        match entry.file_type() {
            Ok(file_type) if file_type.is_dir() => {
                subdirs.push(entry.file_name().to_string_lossy().into_owned())
            },
            Ok(file_type) if file_type.is_file() => {
                let file_name = entry.file_name().to_string_lossy().into_owned();
                let ext = file_name.rsplit('.').next().unwrap_or_default();
                println!("ext = {}", ext);
                if VIDEO_EXTENSIONS.contains(&ext) {
                    files.push(file_name)
                }                
            },
            Ok(_) => {
                println!("Ignoring symlink: {}", &entry.file_name().to_string_lossy());
            },
            Err(e) => {
                println!("Error getting file type for dir entry {}: {:?}", &entry.file_name().to_string_lossy(), e);
            }
        }
    });

    Ok((subdirs, files))
}
