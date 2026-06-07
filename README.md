# Vimms-DL

Vimms-DL is a tool for downloading ROMs from the site [Vimm's Lair](https://vimm.net/)

![alt text](./imgs/welcome.png)

## Installation

Requires [Rust](https://www.rust-lang.org/tools/install) (latest stable).

```bash
git clone https://github.com/lballore/vimms-downloader.git
cd vimms-downloader
cargo run --release
```

Or build and run directly:

```bash
cargo build --release
./target/release/vimms
```

## Features
- Automatic extraction and deletion if specified for both modes

### Search Mode
- Search can be system specific or a general search across the whole site
- Example query below\
  ![alt text](./imgs/search.png)

### Bulk Mode
- Can be used to download specified systems or all of them
- Creation of a console based alpha-numeric directory structure in the root project directory where the files are downloaded to

## Usage
- Follow the on screen instructions to go into either (Bulk/Search) mode
- From there it will prompt you on what criteria to search for or what systems you want to bulk download

## Contributing
Pull requests are welcome. For major changes, please open an issue first to discuss what you would like to change.

## License
[MIT](https://choosealicense.com/licenses/mit/)

## Credits
The script is a fork of [this project](https://github.com/TrendingTechnology/VimmsDownloader), by [Brian Tipton](https://github.com/BrianTipton1)
just adapted too the current [Vimm's Lair website](https://vimm.net/)
