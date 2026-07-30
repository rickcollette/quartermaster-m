# atascii

A lossless-first Rust ATASCII library for Atari 8-bit editors, BBS clients,
terminal emulators, converters, and asset tooling.

The crate is intentionally reusable by two different application classes:

- **ATASCII Editor Project** — use `AtasciiByte`, `DecodeDomain::TextFile` or
  `DecodeDomain::RawGlyphs`, `Parser`, `Screen`, screen-code conversion, and
  translation APIs. Editor file loading does not perform network CR/LF
  normalization unless the application explicitly requests it.
- **ATASCII Terminal Project** — use `TerminalProfile`, `TerminalDecoder`,
  incoming newline policies, ASCII compatibility switches, and outgoing Return
  and Backspace encoders. Telnet/SSH protocol handling remains outside this
  crate.

## Build

```sh
cargo test --all-features
cargo test --no-default-features
cargo run --example editor_integration
cargo run --example terminal_integration
cargo run --example dump
```

Read [`SPEC.md`](SPEC.md) before integrating. The specification explains the
critical distinctions between ATASCII bytes, editor commands, glyphs, screen
codes, Unicode approximations, and transport compatibility.

## Core integration rule

```text
Editor:   file bytes -> Parser(TextFile/RawGlyphs) -> document/screen model
Terminal: transport protocol -> TerminalDecoder -> token/screen/event model
```

Never feed raw Telnet IAC bytes directly into the ATASCII decoder. Never apply
terminal CR/LF normalization automatically when loading a native ATASCII file.
