# hexhog

configurable hex viewer/editor

![hexhog lol](meow.gif)

To run use:
```
hexhog <file>
```

Current limitations:
- loads whole file into memory
- can't insert or create new files

To do:
- add selections
- add copy/paste
- add find

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

[charset]
null = "."
ascii_whitespace = "·"
ascii_other = "°"
non_ascii = "×"
```

## License

Copyright (c) dvdtsb <2025>

This project is licensed under the MIT license ([LICENSE] or <http://opensource.org/licenses/MIT>)

[LICENSE]: ./LICENSE
