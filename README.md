# Base64 Encoder and Decoder

My little project to implement Base64 encoding and decoding, through which I practiced bitwise operations and using byte buffers in Rust. I built a little interactive CLI application with [crossterm](https://github.com/crossterm-rs/crossterm) do print out both encoded and decoded strings based the current user input, and added other features:

1. Switch highlighted strings (input, encoded, decoded) using the up/down arrow keys
2. Copy highlighted strings to clipboard using Enter key
3. Paste input string from clipboard to input field

And yes, this CLI handles UTF-8 strings properly! So emojis, non-Latin characters, etc. are all supported.

I have a little demo here:

[![asciicast](https://asciinema.org/a/XKLIgbiAc6oTBhIphAsKjqm55.svg)](https://asciinema.org/a/XKLIgbiAc6oTBhIphAsKjqm55)
