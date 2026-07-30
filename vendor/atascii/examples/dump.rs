use atascii::{decode_unicode, DecodePolicy};
fn main() {
    let data = b"HELLO\x9bATARI";
    print!("{}", decode_unicode(data, DecodePolicy::UnicodeApprox));
}
