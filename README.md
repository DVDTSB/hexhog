# hexhog

A configurable hex viewer/editor

![hexhog lol](hexhog.gif)

To run `hexhog`, use the following command:
```
hexhog <file>
```

## Installation
If you have cargo installed, you can run the following command:
```
cargo install hexhog
```

It is available on AUR and Homebrew, thanks to [@dhopcs](https://github.com/dhopcs) and [@chenrui333](https://github.com/chenrui333).
```
yay -S hexhog
```

```
brew install hexhog
```

I hope to make this tool available on other package managers soon.

## Features
For now, `hexhog` allows for basic hex editing features for files, such as editing/deleting/inserting bytes, as well as selecting and copy/pasting bytes. I'm look forward to adding other features, including (but not only):
- moving the selection
- find/replace
- bookmarks
- better navigation
- CP437
- other coloring options

While I do love (and use) modal editors, `hexhog` does not attempt to be one. I am trying to make it as intuitive as possible :)

## Configuration

You can find the configuration file in the following locations:
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

Copyright &copy; dvdtsb 2025

This project uses the MIT license ([LICENSE] or [http://opensource.org/licenses/MIT](http://opensource.org/licenses/MIT)).

[LICENSE]: ./LICENSE
