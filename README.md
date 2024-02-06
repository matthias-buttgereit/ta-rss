# Ta-RSS

## Installation
Zunächst Rust installieren: https://www.rust-lang.org/tools/install

Anschließend das Repository klonen und in das Verzeichnis wechseln ...
```bash
~> git clone git@github.com:matthias-buttgereit/ta-rss.git
~> cd ta-rss
```

... und das Projekt kompilieren + ausführen.
```bash
~\ta-rss> cargo run
```

# How To Use
In der Liste kann mit den Pfeiltasten navigiert werden.

`Space` öffnet ein Popup bzw. schließt es wieder.

`Esc` schließt das Popup bzw. beendet das Programm.

`Q`, `Ctrl+C` beendet das Programm.

Aktuell sind noch zwei rss-URLs hardcoded in `utility.rs`

## Features
+ neue Feeds hinzufügen mit `ta-rss --add "https://url-to-rss-feed.com"`
