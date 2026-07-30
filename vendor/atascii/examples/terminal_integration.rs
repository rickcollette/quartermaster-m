use atascii::{
    encode_profile_backspace, encode_profile_return, Charset, DecodeDomain, IncomingNewlinePolicy,
    TerminalDecoder, TerminalProfile,
};

fn main() {
    // A terminal chooses compatibility at the connection-profile boundary.
    let profile = TerminalProfile {
        incoming_newlines: IncomingNewlinePolicy::CrLfToAtasciiEol,
        ..TerminalProfile::default()
    };

    let mut decoder = TerminalDecoder::new(profile, DecodeDomain::ScreenEditor, Charset::Standard);
    decoder.push(b"REMOTE LINE\r\n", &mut |event| println!("{event:?}"));
    decoder.finish(&mut |event| println!("{event:?}"));

    println!(
        "return bytes: {:02X?}",
        encode_profile_return(profile).as_slice()
    );
    println!(
        "backspace bytes: {:02X?}",
        encode_profile_backspace(profile).as_slice()
    );
}
