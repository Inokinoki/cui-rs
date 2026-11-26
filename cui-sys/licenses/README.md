# Licenses of what `cui-sys` embeds

The binding is MIT (the repository's `LICENSE`). The vendored CUI sources
in `c/` carry third-party work under their own terms. These files are the
license texts as the upstream release ships them (`assets/fonts/`), copied
unchanged by `vendor.sh --pin`:

- `LICENSE-Inter.txt` — the Inter font, SIL Open Font License 1.1.
- `LICENSE-Geist.txt` — the Geist Sans and Geist Mono fonts, SIL OFL 1.1.
- `LICENSE-Lucide.txt` — the Lucide icon glyphs, ISC (and MIT for the icons
  the file lists).

They cover the font data embedded in `c/cui.c` (`cui_font_inter_*`,
`cui_font_geist_*`, `cui_font_geist_mono_*` and `cui_font_lucide_*`,
compiled in unless the `default-fonts` feature is off) and the `CUI_ICON_*`
strings in `c/cui.h`.

The stb libraries in `c/cui.c` (stb_truetype, stb_image, stb_image_write)
and the small public-domain decoders keep their license text inline in the
source, at the end of each file.
