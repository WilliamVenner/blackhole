<p align="center">
	<img alt="Logo" src="https://i.imgur.com/VX8fAQI.png"/>
</p>

## Blackhole

[![Travis CI Build Status](https://travis-ci.com/WilliamVenner/blackhole.svg?token=GXuyFsyVxqMmbV5zG6B4&branch=master)](https://travis-ci.com/github/WilliamVenner/blackhole)

Blackhole is a simple program that creates a folder in your computer's home directory where **files may not return**.

Every time you start your computer/log into your user account, the Blackhole directory is moved to your computer's Recycle Bin/Trash, where you can restore it if needed.

## Releases

[Click here](https://github.com/WilliamVenner/blackhole/releases) for pre-built binaries.

## Requirements

* Your operating system must have some form of "Recycle Bin" or "Trash"
* Your operating system must provide you a home directory
* The program may require administrative/elevated privileges on some operating systems

## Windows

Blackhole will automatically add itself to your startup programs via the registry.

If contents are present, the `$BLACKHOLE` directory will be moved to the Recycle Bin every time you start up your computer.

The `$BLACKHOLE` directory will automatically be added to the Quick Access locations.

#### File Location

`%USERPROFILE%/$BLACKHOLE`

## Linux & MacOS

Purging the Blackhole at startup is not yet supported on these operating systems.

If you know Rust and a bit about your favourite OS, pull requests are appreciated.

If you simply want to do it yourself, just run the program with the `--purge` flag at startup.

#### File Location

`$HOME/$BLACKHOLE`

## Screenshots

<p align="center">
	<img alt="Windows" src="https://i.imgur.com/LwHRoH5.png/">
</p>

## Credits

Icon made by [Flat Icons](https://www.flaticon.com/authors/flat-icons) from [www.flaticon.com](https://www.flaticon.com)
