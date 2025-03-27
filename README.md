# crumble

A robust, minimal library for parsing mime documents. Supports UTF-8, multipart and nested structures. Will try (usually successfully) to parse noncompliant documents.

# Usage

Just add `crumble = "0.11.1"` to your dependencies. Then, given a String rep of a MIME document, parse with `Message::new(&mime)`.

Documentation: <https://docs.rs/crumble/>
Crate: <https://crates.io/crates/crumble>

# Notes
- Only returns a simple "AST". You should wrap this in something e.g. [crinkle](https://git.sr.ht/~happy_shredder/crinkle) for it to be useful.
- Example MIME documents which fail parsing welcome!
- Mirrored on GitHub, upstream is on [sr.ht](https://git.sr.ht/~happy_shredder/crumble).

# Licence
GPLv3+
