# font-feature-tester

See how text looks when written using specific font with specific OpenType features. Provides a playground to check different fonts and setups before modifying your terminal or editor.

<!-- blog post link goes here -->

## Installation

The main program is written in Rust. It requires rustc 1.85.0 or newer. It requires FreeType, HarfBuzz and Cairo, all of which are extremely common on Linux machines and should be already installed.

There's also pure-Python helper script that requires recent Python 3. It was tested using 3.11.

`cargo` takes care of fetching all dependencies and building a binary:

    cargo build --release

## Usage

    target/release/font-feature-tester --configuration-path resources/configs/OverpassMono.toml --output-path /tmp/OverpassMono.png

After running `cargo build`, the binary is available as `target/release/font-feature-tester`.

There are two mandatory arguments: `--configuration-path` and `--output-path`. `--configuration-path` is a path to font configuration file in TOML format (see below). `--output-path` is a path to image file that will be generated. The image will be in PNG format, existing files will be overwritten and directory must already exist.

There are also some optional arguments. See `--help` for a list.

Sample configuration files can be found in this repo, under `resources/configs/`.

Sample texts to print can be found under `resources/texts/`. Use `--input-path` to select one of them.

### Using Python helper

    python3 generate-all.py

Python script `generate-all.py` generates images in batch, for all configuration files in a single directory.

It supports similar set of arguments as Rust binary, which helps to maintain consistency.

All arguments are optional. By default it will look for configuration files inside `resources/configs/` **relative to script directory**, will use Rust executable in `target/release/` or `target/debug/` **relative to script directory**, and will save images in new temporary directory. See `--help` for all supported arguments.

### Configuration file

The minimal configuration file is:

```
[font]
file_path = "../fonts/OverpassMono-Regular.otf"
```

`file_path` is a path to font file. Relative paths are resolved **relative to configuration file**. Absolute paths are also supported, but they are not portable between machines.

`size` is font size in pixel. `15` is assumed by default.

`[font.features]` table is a mapping of OpenType features to use, and their values. Usually `0` means that feature is disabled and `1` means that feature is enabled, but meaning of features and their values are font-specific. Some fonts may support features with values larger than `1`, but these are rare.

Tools like [Wakamai Fondue](https://wakamaifondue.com/) and [Bulletproof Font Tester](https://www.adamjagosz.com/bulletproof/) may help if you want to learn what features are supported by a specific font.

See files inside `resources/configs/` for examples of configuration files.
