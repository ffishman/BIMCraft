## Translations: Run Script and Import 3D Model, back in Portuguese and Spanish and new in the other languages

When #391 (Run Script) and #393 (Import 3D Model) were merged, the catalogs were regenerated from main, and the Portuguese and Spanish their branches carried were lost: on main today the 16 labels they added (11 of Run Script's dialog, 5 of Import 3D Model) have an empty `msgstr` in every language, so both show in English everywhere.

This puts them back, as the branches had them, and adds the other nine languages (de, fr, it, nl, pl, cs, vi, zh, zh_TW), using each catalog's existing words where it has them (Undo, Left out, Families, Import). Only `translations/ui/*.po` changes; no code.

- The 16 labels in 11 languages: 175 strings (Polish already had "Run").
- Placeholders (`{}`) are kept in number and order in every one.
- `cargo i18n check` passes.

The Portuguese and Spanish are reviewed; the other languages would welcome a native speaker's look, Czech, Polish, Vietnamese and Chinese most of all.
