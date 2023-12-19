# Microchip's SVDs from ATPACKs Harvester

The intention of this software is to scrape the website with ATPACKs to obtain SVDs from them for some families of ATSAM chips.

## Usage

Install using `cargo install atpacks-svd-harvester` command and then:
```sh
atpacks-svd-harvester -r https://packs.download.microchip.com -f same51 -d svd/ -m svd-versions.json
```
to obtain SVD files from the _Microchip's_ website for ATSAME51 family. Omitting `-f` will obtain all SVD for all supported families. They're shown in help text:
```sh
atpacks-svd-harvester --help
```
The `-f` option can be used multiple times.

The `-m svd-versions.json` generates file where each SVD obtained contains the version of the ATPACK it was extracted from. _Microchip_ doesn't version their SVD files internally.

## Legalities

### Licensing

Use of this software implies acceptance of APACHE 2.0 license of the content that _Microchip_ offers on their website.

This software is licensed under MIT or Apache 2.0 license, as stated in [`Cargo.toml`](Cargo.toml).

### Affiliation

The author of this software is **not** affiliated with _Microchip_ (or former _Atmel_) in any way. This software exists for convenience of people working with SVD files for chips manufactured by _Microchip_.

### Copyrights

Microchip's name and logo are registered trademarks of Microchip Technology Inc. and are used by this software and its documentation for informational purpose only.

Moral rights to this software remain with the author, every contributor has right to attribution matching the license of the software.
