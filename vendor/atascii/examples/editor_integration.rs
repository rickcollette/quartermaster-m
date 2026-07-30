use atascii::{AtasciiByte, Charset, DecodeDomain, Parser, Screen};

fn main() {
    // An editor loads native ATASCII bytes without transport normalization.
    let document = b"ATASCII EDITOR\x9bSECOND LINE";
    let mut parser = Parser::new(DecodeDomain::TextFile, Charset::Standard);
    let mut screen = Screen::new(40, 24).expect("valid screen dimensions");

    for &byte in document {
        if let Some(token) = parser.feed(AtasciiByte(byte)) {
            screen.apply(token);
        }
    }

    println!("editor cursor: {:?}", screen.cursor());
}
