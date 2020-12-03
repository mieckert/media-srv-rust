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

Edit `Rocket.toml` to set your port, and IP.  Create a `mount.toml`
maps mount points (i.e., prefixes of the URLs exposed) to 
local directories.  Example `mount.toml`:

```
"u" = "/Users/auser"
"desk/top" = "/Users/auser/Desktop/Desktop"
```

Then simply run `cargo build` (it will install the needed JavaScript
and CSS dependencies as well) and `cargo run`.

## Limitations / TODOS

- [ ] Support for other video formats (currently only files ending 
      in '*.mp4' are shown)
- [ ] Support for other media formats (images, PDFs, etc.)      
- [ ] Sorting of files and directories (alphabetical etc.)
- [ ] Nicer layout
- [X] Root path on Windows (how can you serve multiple drives, e.g.,  `c:\` and `d:\`); possibly also for other systems to specify multiple roots
- [ ] Refactor code for serving byte ranges (Ranged_File) and possibly extract it into its own crate
- [ ] Add possibility to go up a directory ("..") except in the root directory
