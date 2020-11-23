# OBS Controller
[![Gitlab pipeline status](https://img.shields.io/gitlab/pipeline/Beezig/obs-controller/master)](https://gitlab.com/Beezig/obs-controller/-/pipelines) 
[![Build Status](https://dev.azure.com/roccodevbusiness/Beezig/_apis/build/status/Beezig.obs-controller?branchName=master)](https://dev.azure.com/roccodevbusiness/Beezig/_build/latest?definitionId=2&branchName=master)

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
