<p align="center">
	<img alt="Logo" src="https://i.imgur.com/VX8fAQI.png"/>
</p>

## Blackhole

[![GitHub Actions Build Status](https://github.com/WilliamVenner/blackhole/workflows/build/badge.svg)](https://github.com/WilliamVenner/blackhole/actions?query=workflow%3Abuild)

Blackhole is a simple program that creates a folder in your computer's home directory where **_files may not return_**.

Every time you start your computer/log into your user account, if contents are present, the Blackhole directory is moved to your computer's Recycle Bin/Trash, where you can restore it if needed.

## Use Cases

* Temporary downloads
* Temporary torrents
* Temporary extractions of specific files in archives (`.zip`, `.tar`, `.rar`, etc)
* Temporary storage for files waiting to be uploaded elsewhere, e.g. via FTP
* Temporary downloads of files from instant messaging
* A recoverable/very large/non-volatile (but slower) ramdisk
* A glorified temp folder/recycle bin

_And much more..._

## Releases

[Click here](https://github.com/WilliamVenner/blackhole/releases) for pre-built binaries.

## Windows

* Blackhole will automatically add itself to your startup programs via the registry.
* The `BLACKHOLE` directory will be moved to the Recycle Bin every time you login, if contents are present.
* The `BLACKHOLE` directory will automatically be added to the Quick Access locations.

#### File Location

`%USERPROFILE%/BLACKHOLE`

## macOS

* Blackhole will automatically add itself to your login items when you first run the program.
* The `BLACKHOLE` directory will be moved to the Trash every time you login, if contents are present.
* The `BLACKHOLE` directory will automatically be added to the Finder sidebar.

#### File Location

`$HOME/BLACKHOLE`

## Screenshots

<p align="center">
	<img alt="Windows" src="https://i.imgur.com/LwHRoH5.png/">
</p>

## Credits

Icon made by [Flat Icons](https://www.flaticon.com/authors/flat-icons) from [www.flaticon.com](https://www.flaticon.com)