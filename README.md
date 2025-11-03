# Basic PNG encoder/decoder library writen in Rust without any dependencies

>Example:
```rust

let result = read_png("./image.png").unwrap(); // Vec<Vec<(u8, u8, u8)>> of pixels
```

## Add library to your project:
```
[dependencies]
regex = { git = "git@github.com:romanpolishchuk/png_is_simple.git" }
```
## Warning!
At the moment this library support only decoding of uncompressed pallet png files
