# OBS Controller
An OBS plugin to control recording status over HTTP.

## Building
Clone the project and run
```
cargo build --release
```
You'll find a compiled shared library in the `target/release` folder.

## Contributing
Install `clippy` with
```
rustup component add clippy
```
and run
```
RUSTFLAGS=-Dwarnings cargo clippy
```
to check if your code passes the quality check.

## License
[<img align="right" src="http://www.gnu.org/graphics/gplv3-127x51.png"/>][license]

OBS Controller is licensed under the Terms and Conditions of the **GNU GPL v3+**.<br/>
Read [the LICENSE file][license] for more.

[license]: https://gitlab.com/Beezig/obs-controller/-/blob/master/LICENSE
