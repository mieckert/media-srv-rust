# Media Server in Rust

This a small media web server written in 
[Rust](https://rust-lang.org) using the [Rocket](https://rocket.rs) 
web framework.  Simply point the server to a root directory (specified
in `Rocket.toml`) and you can browse the directories and play video 
files in those directories.  It uses [video.js](https://videojs.com/) 
as Video Player.

This is private for fun project.  Do not use it in production or any
serious settings.

## Installing/Running

Edit `Rocket.toml` to set your root directory, port, and IP.  
Then simply run `cargo build` (it will install the needed JavaScript
and CSS dependencies as well) and `cargo run`.

## Limitations / TODOS

- [ ] Support for other video formats (currently only files ending 
      in '*.mp4' are shown)
- [ ] Support for other media formats (images, PDFs, etc.)      
- [ ] Sorting of files and directories (alphabetical etc.)
- [ ] Nicer layout
- [ ] Root path on Windows (how can you serve multiple drives, e.g., 
      `c:\` and `d:\`); possibly also for other systems to specify
      multiple roots
