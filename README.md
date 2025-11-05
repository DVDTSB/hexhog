# hexhog

configurable hex viewer/editor

![hexhog lol](hexhog.gif)

To run use:
```
hexhog <file>
```

## Instalation
If you have cargo installed, just run:
```
cargo install hexhog
```

Alternatevely, it is available on AUR (many thanks to @dhopcs).
```
yay -S hexhog
```

I hope I can make this tool available on other package managers soon.

## Features
For now `hexhog` allows for basic hex editing features for files, such as editing/deleting/inserting bytes, as well as selecting and copy/pasting bytes. I'm look forward to adding other features, including (but not only):
- moving the selection
- find/replace
- bookmarks
- better navigation
- CP437
- other coloring options

While I do love(and use) modal editors, `hexhog` does not attempt to be one. I am trying to make it as intuitive as possible :)

## Configuration

Configuration file is located at:
- Linux: `/home/user/.config/hexhog/config.toml`
- Windows: `C:\Users\user\AppData\Roaming\hexhog\config.toml`
- MacOS: `/Users/user/Library/Application Support/hexhog/config.toml`

An example configuration file:
```toml
[theme]
null = "dark_gray"
ascii_printable = "blue"
ascii_whitespace = [67, 205, 128] # rgb
ascii_other = 162 # ansi
non_ascii = "red"
accent = "blue"
primary = "green"
background = "black"
border = "cyan"

[charset]
null = "."
ascii_whitespace = "·"
ascii_other = "°"
non_ascii = "×"
```

## Feedback

Feedback on `hexhog` is highly appreciated. Thanks! :D

## License

Copyright (c) dvdtsb <2025>

This project is licensed under the MIT license ([LICENSE] or <http://opensource.org/licenses/MIT>)

[LICENSE]: ./LICENSE
