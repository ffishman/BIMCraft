Thanks for the GIFs, they make it clear: Manage > Project Language should offer the languages the interface has.

On main today it is closer than in 0.11.0: Polish and Vietnamese are project languages now too, so the gap is Chinese (Simplified and Traditional), which has an interface catalog but no project one. As `docs/areas/translations.md` says, that is not only a catalog: the drawings' own lettering (`font.rs`, a stair's UP, a title block, tags) is stroke lettering with Latin glyphs only, so Chinese words would print as `?`.

@IPedrax, we'd be glad to take this on, and would like to know which way you prefer before starting:

1. CJK in the drawings' lettering: where a character has no stroke glyph, draw it with a system font (as text notes already can), in the PNG, PDF and on screen; then
2. `zh` and `zh_TW` as project languages (a new `Language` variant, so a format step with nothing to convert), with their project catalogs (~200 words), for a native speaker to review.

Or, if you'd rather keep the stroke lettering Latin-only for now, the menu could list the interface's languages that have no project catalog yet, greyed out with a short note, so it doesn't look like a bug.
