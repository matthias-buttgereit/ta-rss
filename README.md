# Ta-RSS
![Screenshot 2024-03-06 151150](https://github.com/matthias-buttgereit/ta-rss/assets/65292669/0668bc50-c8ab-4c0c-b472-8f5e39e66004)

Terminal App for reading RSS Feeds

## Installation
First make sure to have Rust installed: https://www.rust-lang.org/tools/install

Then just clone this repository, change into the directory ...
```bash
git clone git@github.com:matthias-buttgereit/ta-rss.git
cd ta-rss
```

... and install the app into any directory by running
```bash
cargo install --root [DIRECTORY] --path .
```

To add rss feeds just use the `add` subcommand
```bash
ta-rss add [FEED-URL]
```

For a full list of commands use the `--help` argument
```bash
ta-rss --help
```

## How To Use
While the list of feeds is displayed you can navigate the entries with the up and down arrow keys.

`Space` opens a popup to get more information on the selected feed.

`Esc` closes the popup or quits the app when no popup is open.

`Q`, `Ctrl+C` always quits the app.

`O` opens the current feed in the browser.

