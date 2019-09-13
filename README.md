# crumble

A robust, minimal library for parsing mime documents. Supports UTF-8, multipart and nested structures. Will try (usually successfully) to parse noncompliant documents.

# Notes
- Only returns a simple "AST". You should wrap this in something e.g. [crinkle](https://git.sr.ht/~happy_shredder/crinkle) for it to be useful.
- Example MIME documents which fail parsing welcome!

# TODO
- Add function to construct MIME documents.

# Licence
GPLv3+
