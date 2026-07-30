use atascii::*;

#[test]
fn inverse_roundtrip() {
    for byte in 0u8..=255 {
        let atascii = AtasciiByte(byte);
        assert_eq!(atascii.with_inverse(atascii.is_inverse()), atascii);
    }
}

#[test]
fn raw_byte_domain_preserves_every_value() {
    let mut parser = Parser::new(DecodeDomain::RawBytes, Charset::Standard);
    for byte in 0u8..=255 {
        assert_eq!(parser.feed(byte.into()), Some(Token::Raw(byte.into())));
    }
}

#[test]
fn screen_code_roundtrip() {
    for byte in 0u8..=255 {
        assert_eq!(
            screen_to_atascii(atascii_to_screen(AtasciiByte(byte))),
            AtasciiByte(byte)
        );
    }
}

#[test]
fn escape_quotes_cursor() {
    let mut parser = Parser::new(DecodeDomain::ScreenEditor, Charset::Standard);
    assert_eq!(parser.feed(0x1B.into()), None);
    assert!(matches!(parser.feed(0x1C.into()), Some(Token::Glyph(_))));
}

#[test]
fn escape_state_survives_chunks() {
    let profile = TerminalProfile::default();
    let mut decoder = TerminalDecoder::new(profile, DecodeDomain::ScreenEditor, Charset::Standard);
    let mut tokens = Vec::new();
    decoder.push(&[0x1B], &mut |token| tokens.push(token));
    assert!(tokens.is_empty());
    assert!(decoder.has_pending_escape());
    decoder.push(&[0x1F], &mut |token| tokens.push(token));
    assert!(matches!(tokens.as_slice(), [Token::Glyph(_)]));
}

#[test]
fn eol_cannot_be_quoted() {
    let mut parser = Parser::new(DecodeDomain::ScreenEditor, Charset::Standard);
    assert_eq!(parser.feed(0x1B.into()), None);
    assert_eq!(
        parser.feed(0x9B.into()),
        Some(Token::Control(Control::EndOfLine))
    );
}

#[test]
fn native_ascii_controls_remain_glyphs() {
    let mut parser = Parser::new(DecodeDomain::ScreenEditor, Charset::Standard);
    assert!(matches!(parser.feed(0x08.into()), Some(Token::Glyph(_))));
    assert!(matches!(parser.feed(0x07.into()), Some(Token::Glyph(_))));
    assert_eq!(parser.feed(0x7F.into()), Some(Token::Control(Control::Tab)));
}

#[test]
fn native_atari_controls_are_distinct() {
    let mut parser = Parser::new(DecodeDomain::ScreenEditor, Charset::Standard);
    assert_eq!(
        parser.feed(0x7E.into()),
        Some(Token::Control(Control::Delete))
    );
    assert_eq!(
        parser.feed(0xFD.into()),
        Some(Token::Control(Control::Buzzer))
    );
    assert_eq!(
        parser.feed(0xFE.into()),
        Some(Token::Control(Control::DeleteCharacter))
    );
}

#[test]
fn crlf_collapses_to_one_eol_across_chunks() {
    let profile = TerminalProfile {
        incoming_newlines: IncomingNewlinePolicy::CrLfToAtasciiEol,
        ..TerminalProfile::default()
    };
    let mut decoder = TerminalDecoder::new(profile, DecodeDomain::ScreenEditor, Charset::Standard);
    let mut tokens = Vec::new();
    decoder.push(&[0x0D], &mut |token| tokens.push(token));
    assert!(tokens.is_empty());
    decoder.push(&[0x0A], &mut |token| tokens.push(token));
    assert_eq!(tokens, vec![Token::Control(Control::EndOfLine)]);
}

#[test]
fn unmatched_cr_is_preserved_in_crlf_mode() {
    let mut decoder = IncomingNewlineDecoder::new(IncomingNewlinePolicy::CrLfToAtasciiEol);
    assert!(decoder.feed(0x0D.into()).as_slice().is_empty());
    assert_eq!(decoder.finish().as_slice(), &[0x0D]);
}

#[test]
fn outgoing_keys_are_profile_driven() {
    assert_eq!(
        encode_return(OutgoingNewlinePolicy::AtasciiEol).as_slice(),
        &[0x9B]
    );
    assert_eq!(
        encode_return(OutgoingNewlinePolicy::CrLf).as_slice(),
        &[0x0D, 0x0A]
    );
    assert_eq!(
        encode_backspace(OutgoingBackspace::AtasciiDelete).as_slice(),
        &[0x7E]
    );
    assert_eq!(
        encode_backspace(OutgoingBackspace::AsciiBackspace).as_slice(),
        &[0x08]
    );
}

#[test]
fn editor_text_file_mode_recognizes_only_native_eol() {
    let mut parser = Parser::new(DecodeDomain::TextFile, Charset::Standard);
    assert!(matches!(parser.feed(0x0D.into()), Some(Token::Glyph(_))));
    assert!(matches!(parser.feed(0x0A.into()), Some(Token::Glyph(_))));
    assert_eq!(
        parser.feed(0x9B.into()),
        Some(Token::Control(Control::EndOfLine))
    );
}

#[test]
fn ascii_newline_encoding_does_not_duplicate_crlf() {
    assert_eq!(
        encode_ascii("A\r\nB", EncodePolicy::Strict).expect("encodable"),
        vec![65, 155, 66]
    );
}
