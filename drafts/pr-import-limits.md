## Import CAD: up to 512 MB of DXF and 128 MB of DWG

The DXF reader refused a file past 128 MB, and a DWG was refused past 64 MB before the decoder ran. On real projects both are too tight. A DWG's DXF is 7 to 16 times its size as acadrust writes it, so a plumbing plan of a residential tower (29.5 MB of DWG, 541,478 entities) became 197 MB of DXF and couldn't come in. Large structural and fire-protection sets of public hospitals, of 66 to 77 MB, didn't reach the decoder at all.

This raises `dxf::MAX_BYTES` to 512 MB and `Limits::DEFAULT.dwg_bytes` to 128 MB. `dxf_bytes` is now `dxf::MAX_BYTES` itself, so the two can't drift apart. Nothing else changes: `MAX_WORK` (4 million entities placed) still bounds the work, and none of the files below comes near it.

### Measured (release build, Apple M4, 16 GB)

| Drawing | DWG | its DXF | DWG → DXF | import + save | peak memory | entities | project file |
|---|---|---|---|---|---|---|---|
| plumbing plan, residential tower | 29.5 MB | 197 MB | 1.9 s | 3.0 s | 1.1 GB | 541,478 | 47 MB |
| electrical plan (saved by #394 and read back) | 28.7 MB | 187 MB | 1.6 s | 2.4 s | 1.5 GB | 238,189 | 32 MB |
| lightning protection (the same) | 23.9 MB | 166 MB | 1.3 s | 2.8 s | 1.3 GB | 213,163 | 28 MB |
| architecture (the same) | 38.2 MB | 301 MB | 2.5 s | 4.1 s | 2.1 GB | 465,196 | 42 MB |
| plumbing, public maternity hospital (the same) | 17.3 MB | 162 MB | 1.6 s | 2.6 s | 1.3 GB | 469,832 | 17 MB |

Opening that first project again takes about as long as importing (3.6 s of CPU), and drawing its whole plan to PNG another 0.3 s. With the old limits, 15 public-body drawings and the plumbing plan above were refused. With these, the largest DXF measured (301 MB) has room to spare.

### Next, if wanted

Reading acadrust's document straight into the reader's entities, without the DXF text in between, would cut the memory to about a third. Holding the parsed drawing in the project would make opening a large one almost instant. Both are bigger changes, worth it only when a real drawing needs them.

### Checked

- The tests that held the limits now check the new ones without allocating them: `too_big` says why a size is refused, and a test pins `dxf_bytes` to `MAX_BYTES`.
- `cargo test`, clippy, fmt and `cargo i18n check` pass.
