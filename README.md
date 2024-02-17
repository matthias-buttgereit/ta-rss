# Ta-RSS

## Installation
First make sure to have Rust installed: https://www.rust-lang.org/tools/install

Then just clone this repository, change into the directory ...
```bash
git clone git@github.com:matthias-buttgereit/ta-rss.git
cd ta-rss
```

... then compile the app in release mode (optimized).
```bash
cargo build --release
```

The app should compile into `./target/release/main.exe`.

To add rss feeds just use the `--add` option. You can also combine building and running the app

```bash
cargo run --release -- --add [FEED-URL]
```
After adding one or multiple feeds, just run the app without any additional parameters.
```bash
cargo run --release
```

## How To Use
While the list of feeds is displayed you can navigate the entries with the up and down arrow keys.

`Space` opens a popup to get more information on the selected feed.

`Esc` closes the popup or quits the app when no popup is open.

`Q`, `Ctrl+C` always quits the app.

Aktuell sind noch zwei rss-URLs hardcoded in `utility.rs`

## Features

