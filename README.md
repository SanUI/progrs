An alternative to [warcraftrecorder](https://warcraftrecorder.com) on linux.

## Status

* First release, basically works, but little more
* Error handling and printing pretty rough around the edges
* Configuration need (suggestion: install gpu-screen-recorder, gpu-screen-recorder-gtk
  and mkvmerge to make your life easier)

## Installation

* Download a release artifact, this binary should run on basically every system running linux
* Alternative: Clone the repository and build yourself with `cargo build --release`

## Configuration

On the first run, `progrs` will note you don't have a configuration file, and
create one for you. It's probably `~/.config/progrs/config.tml`. You will need
to edit that, at least put in the necessary values for `watchdir` and `viddir`.
It is well commented, please look around and ajust as necessary.

## Contributing

Everything's welcome, just open an issue.

## License

MIT or Apache at your leisure.

## CoC

Wherever applicable, this project follows the [rust code of
conduct](https://www.rust-lang.org/en-US/conduct.html).
