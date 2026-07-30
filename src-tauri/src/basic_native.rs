#![allow(dead_code)]

use std::env;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

type Result<T> = std::result::Result<T, String>;

const HEADER_WORDS: usize = 7;
const HEADER_LEN: usize = HEADER_WORDS * 2;
const FILE_LOMEM: u16 = 0x0100;
const VAR_TOKEN_BASE: u8 = 0x80;

const STATEMENTS: [&str; 55] = [
    "REM", "DATA", "INPUT", "COLOR", "LIST", "ENTER", "LET", "IF", "FOR", "NEXT", "GOTO", "GO TO",
    "GOSUB", "TRAP", "BYE", "CONT", "?", "CLOSE", "CLR", "DEG", "DIM", "END", "NEW", "OPEN",
    "LOAD", "SAVE", "STATUS", "NOTE", "POINT", "XIO", "ON", "POKE", "PRINT", "RAD", "READ",
    "RESTORE", "RETURN", "RUN", "STOP", "POP", "?", "GET", "PUT", "GRAPHICS", "PLOT", "POSITION",
    "DOS", "DRAWTO", "SETCOLOR", "LOCATE", "SOUND", "LPRINT", "CSAVE", "CLOAD", "LET",
];

fn main() {
    if let Err(err) = run() {
        eprintln!("error: {err}");
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let mut args = env::args().skip(1);
    let Some(cmd) = args.next() else {
        return usage();
    };

    match cmd.as_str() {
        "tokenize" | "tok" => {
            let mut output = None;
            let mut source = None;
            while let Some(arg) = args.next() {
                match arg.as_str() {
                    "-o" | "--output" => output = args.next().map(PathBuf::from),
                    "--parser" => {
                        return Err(
                            "--parser is no longer supported; tokenization is native".into()
                        );
                    }
                    _ if arg.starts_with('-') => return Err(format!("unknown option: {arg}")),
                    _ => source = Some(PathBuf::from(arg)),
                }
            }
            let source = source.ok_or("missing source file")?;
            let output = output.unwrap_or_else(|| source.with_extension("BAS"));
            tokenize_file(&source, &output)?;
            println!("{}", output.display());
        }
        "detokenize" | "detok" => {
            let mut output = None;
            let mut input = None;
            while let Some(arg) = args.next() {
                match arg.as_str() {
                    "-o" | "--output" => output = args.next().map(PathBuf::from),
                    _ if arg.starts_with('-') => return Err(format!("unknown option: {arg}")),
                    _ => input = Some(PathBuf::from(arg)),
                }
            }
            let input = input.ok_or("missing tokenized BASIC file")?;
            let listing = detokenize_file(&input)?;
            if let Some(output) = output {
                fs::write(output, listing).map_err(|e| e.to_string())?;
            } else {
                print!("{listing}");
                io::stdout().flush().map_err(|e| e.to_string())?;
            }
        }
        "inspect" => {
            let input = args.next().ok_or("missing tokenized BASIC file")?;
            inspect_file(Path::new(&input))?;
        }
        "-h" | "--help" | "help" => usage()?,
        _ => return usage(),
    }

    Ok(())
}

fn usage() -> Result<()> {
    eprintln!(
        "usage:
  atari_basic_tool tokenize [-o OUT.BAS] SOURCE.bas
  atari_basic_tool detokenize [-o OUT.lst] FILE.BAS
  atari_basic_tool inspect FILE.BAS"
    );
    Err("invalid arguments".into())
}

fn tokenize_file(source: &Path, output: &Path) -> Result<()> {
    let text = fs::read_to_string(source).map_err(|e| e.to_string())?;
    let tokenized = tokenize_listing(&text)?;
    if let Some(parent) = output.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    fs::write(output, tokenized).map_err(|e| e.to_string())
}

pub(crate) fn tokenize_listing(text: &str) -> Result<Vec<u8>> {
    let source_lines = parse_source_lines(text)?;
    let variables = collect_variables(&source_lines);
    let mut statement_bytes = Vec::new();
    for (line_no, body) in &source_lines {
        let line = tokenize_line(*line_no, body, &variables)?;
        statement_bytes.extend(line);
    }
    statement_bytes.extend([0x00, 0x80, 0x06, 0x06, 0x34, 0x16]);

    let vnt_len: usize = variables.iter().map(String::len).sum();
    let vvt_len = variables.len() * 8;
    let vntd = FILE_LOMEM + vnt_len as u16;
    let vvtp = vntd + 1;
    let stmtab = vvtp + vvt_len as u16;
    let stmtcur = stmtab + (statement_bytes.len() as u16 - 6);
    let starp = stmtcur + 6;

    let mut out = Vec::new();
    for word in [0, FILE_LOMEM, vntd, vvtp, stmtab, stmtcur, starp] {
        out.extend(word.to_le_bytes());
    }
    for name in &variables {
        for (idx, byte) in name.bytes().enumerate() {
            let end = idx + 1 == name.len();
            out.push(if end { byte | 0x80 } else { byte });
        }
    }
    out.push(0);
    for (idx, name) in variables.iter().enumerate() {
        out.push(variable_type_byte(name));
        out.push(idx as u8);
        out.extend([0; 6]);
    }
    out.extend(statement_bytes);
    Ok(out)
}

fn parse_source_lines(text: &str) -> Result<Vec<(u16, String)>> {
    let mut lines = Vec::new();
    for (idx, raw) in text.lines().enumerate() {
        let trimmed = raw.trim_end();
        if trimmed.trim().is_empty() {
            continue;
        }
        let first_space = trimmed
            .find(char::is_whitespace)
            .ok_or_else(|| format!("line {} is missing BASIC text", idx + 1))?;
        let line_no = trimmed[..first_space]
            .parse::<u16>()
            .map_err(|_| format!("invalid line number on source line {}", idx + 1))?;
        lines.push((line_no, trimmed[first_space..].trim_start().to_string()));
    }
    Ok(lines)
}

fn collect_variables(lines: &[(u16, String)]) -> Vec<String> {
    let mut vars = Vec::new();
    for (_, body) in lines {
        for stmt in logical_statements(body) {
            let stmt = stmt.text.trim_start();
            let expr = if starts_keyword(stmt, "REM") || starts_keyword(stmt, "DATA") {
                ""
            } else {
                statement_body(stmt).1
            };
            collect_expr_variables(expr, &mut vars);
        }
    }
    vars
}

fn collect_expr_variables(expr: &str, vars: &mut Vec<String>) {
    let mut i = 0usize;
    let bytes = expr.as_bytes();
    while i < bytes.len() {
        let ch = bytes[i] as char;
        if ch == '"' {
            i = skip_string(expr, i);
        } else if ch.is_ascii_alphabetic() {
            let (word, next) = read_word(expr, i);
            i = next;
            let upper = word.to_ascii_uppercase();
            if is_expr_keyword(&upper) || statement_token(&upper).is_some() {
                continue;
            }
            let mut name = upper;
            if bytes.get(i) == Some(&b'$') {
                name.push('$');
                i += 1;
            }
            if !name.ends_with('$') && bytes.get(i) == Some(&b'(') {
                name.push('(');
            }
            if !vars.contains(&name) {
                vars.push(name);
            }
        } else {
            i += 1;
        }
    }
}

fn tokenize_line(line_no: u16, body: &str, vars: &[String]) -> Result<Vec<u8>> {
    let parts = logical_statements(body);
    let mut chunks = Vec::new();
    for stmt in parts {
        chunks.push(tokenize_statement(
            stmt.text.trim_start(),
            stmt.terminator,
            vars,
        )?);
    }
    let line_len = 4 + chunks.iter().map(Vec::len).sum::<usize>() + chunks.len().saturating_sub(1);
    if line_len > u8::MAX as usize {
        return Err(format!("line {line_no} is too long"));
    }
    let mut line = Vec::new();
    line.extend(line_no.to_le_bytes());
    line.push(line_len as u8);
    let mut next_end = 4 + chunks[0].len();
    line.push(next_end as u8);
    for (idx, chunk) in chunks.iter().enumerate() {
        if idx > 0 {
            next_end += 1 + chunk.len();
            line.push(next_end as u8);
        }
        line.extend(chunk);
    }
    Ok(line)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum StatementTerminator {
    None,
    Colon,
    Eol,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct LogicalStatement<'a> {
    text: &'a str,
    terminator: StatementTerminator,
}

fn logical_statements(body: &str) -> Vec<LogicalStatement<'_>> {
    let mut out = Vec::new();
    let parts = split_statements(body);
    for (part_idx, part) in parts.iter().enumerate() {
        let part_terminator = if part_idx + 1 == parts.len() {
            StatementTerminator::Eol
        } else {
            StatementTerminator::Colon
        };
        if let Some((if_part, then_part)) = split_then_statement(part) {
            out.push(LogicalStatement {
                text: if_part,
                terminator: StatementTerminator::None,
            });
            out.push(LogicalStatement {
                text: then_part,
                terminator: part_terminator,
            });
        } else {
            out.push(LogicalStatement {
                text: part,
                terminator: part_terminator,
            });
        }
    }
    out
}

fn split_then_statement(stmt: &str) -> Option<(&str, &str)> {
    if !starts_keyword(stmt.trim_start(), "IF") {
        return None;
    }
    let then_pos = find_word_outside_string(stmt, "THEN")?;
    let after_then = stmt[then_pos + 4..].trim_start();
    if after_then
        .bytes()
        .next()
        .is_some_and(|b| (b as char).is_ascii_digit())
    {
        return None;
    }
    for keyword in statement_keywords_by_len() {
        if starts_keyword(after_then, keyword) {
            let split_at = stmt.len() - after_then.len();
            return Some((&stmt[..split_at], &stmt[split_at..]));
        }
    }
    None
}

fn find_word_outside_string(text: &str, word: &str) -> Option<usize> {
    let upper = text.to_ascii_uppercase();
    let bytes = text.as_bytes();
    let mut in_string = false;
    let mut i = 0usize;
    while i + word.len() <= bytes.len() {
        if bytes[i] == b'"' {
            in_string = !in_string;
            i += 1;
            continue;
        }
        if !in_string && upper[i..].starts_with(word) {
            let before = i
                .checked_sub(1)
                .and_then(|idx| bytes.get(idx))
                .is_none_or(|b| !(*b as char).is_ascii_alphanumeric());
            let after = bytes
                .get(i + word.len())
                .is_none_or(|b| !(*b as char).is_ascii_alphanumeric());
            if before && after {
                return Some(i);
            }
        }
        i += 1;
    }
    None
}

fn tokenize_statement(
    stmt: &str,
    terminator: StatementTerminator,
    vars: &[String],
) -> Result<Vec<u8>> {
    let (token, body) = statement_body(stmt);
    let mut out = vec![token];
    if token == 0 || token == 1 {
        out.extend(body.bytes());
        out.push(0x9b);
    } else {
        out.extend(tokenize_expr(
            body,
            token == 6 || token == 8 || token == 54,
            token == 20,
            vars,
        )?);
        match terminator {
            StatementTerminator::None => {}
            StatementTerminator::Colon => out.push(0x14),
            StatementTerminator::Eol => out.push(0x16),
        }
    }
    Ok(out)
}

fn statement_body(stmt: &str) -> (u8, &str) {
    for keyword in statement_keywords_by_len() {
        if starts_keyword(stmt, keyword) {
            let rest = stmt[keyword.len()..].trim_start();
            return (statement_token(keyword).unwrap(), rest);
        }
    }
    (54, stmt)
}

fn statement_keywords_by_len() -> Vec<&'static str> {
    let mut items: Vec<_> = STATEMENTS.iter().copied().filter(|s| *s != "?").collect();
    items.sort_by_key(|s| std::cmp::Reverse(s.len()));
    items
}

fn statement_token(keyword: &str) -> Option<u8> {
    STATEMENTS
        .iter()
        .position(|stmt| *stmt == keyword)
        .map(|idx| idx as u8)
}

fn starts_keyword(text: &str, keyword: &str) -> bool {
    let upper = text.to_ascii_uppercase();
    if !upper.starts_with(keyword) {
        return false;
    }
    match upper.as_bytes().get(keyword.len()) {
        None => true,
        Some(ch) => !(*ch as char).is_ascii_alphanumeric() && *ch != b'$',
    }
}

fn split_statements(body: &str) -> Vec<&str> {
    let mut parts = Vec::new();
    let mut start = 0usize;
    let mut in_string = false;
    let upper = body.to_ascii_uppercase();
    let bytes = body.as_bytes();
    let mut i = 0usize;
    while i < bytes.len() {
        match bytes[i] {
            b'"' => in_string = !in_string,
            b':' if !in_string && !upper[start..].trim_start().starts_with("REM") => {
                parts.push(&body[start..i]);
                start = i + 1;
            }
            _ => {}
        }
        i += 1;
    }
    parts.push(&body[start..]);
    parts
}

fn tokenize_expr(expr: &str, assignment: bool, dim: bool, vars: &[String]) -> Result<Vec<u8>> {
    let mut out = Vec::new();
    let mut i = 0usize;
    let bytes = expr.as_bytes();
    while i < bytes.len() {
        let ch = bytes[i] as char;
        if ch.is_ascii_whitespace() {
            i += 1;
        } else if ch == '"' {
            let end = skip_string(expr, i);
            let text = &expr[i + 1..end - 1];
            if text.len() > u8::MAX as usize {
                return Err("string literal is too long".into());
            }
            out.push(0x0f);
            out.push(text.len() as u8);
            out.extend(text.bytes());
            i = end;
        } else if ch.is_ascii_digit() || ch == '.' {
            let (num, next) = read_number(expr, i);
            out.push(0x0e);
            out.extend(encode_number(num)?);
            i = next;
        } else if ch.is_ascii_alphabetic() {
            let (word, next) = read_word(expr, i);
            i = next;
            let upper = word.to_ascii_uppercase();
            if let Some(token) = expr_word_token(&upper) {
                out.push(token);
                continue;
            }
            let mut name = upper;
            if bytes.get(i) == Some(&b'$') {
                name.push('$');
                i += 1;
            }
            if !name.ends_with('$') && bytes.get(i) == Some(&b'(') {
                name.push('(');
            }
            let idx = vars
                .iter()
                .position(|v| v == &name)
                .ok_or_else(|| format!("unknown variable {name}"))?;
            out.push(VAR_TOKEN_BASE + idx as u8);
        } else {
            match ch {
                ',' => out.push(0x12),
                ';' => out.push(0x15),
                '#' => out.push(0x1c),
                '<' if bytes.get(i + 1) == Some(&b'=') => {
                    out.push(0x1d);
                    i += 1;
                }
                '<' if bytes.get(i + 1) == Some(&b'>') => {
                    out.push(0x1e);
                    i += 1;
                }
                '>' if bytes.get(i + 1) == Some(&b'=') => {
                    out.push(0x1f);
                    i += 1;
                }
                '<' => out.push(0x20),
                '>' => out.push(0x21),
                '=' => out.push(0x22),
                '^' => out.push(0x23),
                '*' => out.push(0x24),
                '+' => out.push(0x25),
                '-' => out.push(0x26),
                '/' => out.push(0x27),
                '(' => out.push(0x2b),
                ')' => out.push(0x2c),
                _ => return Err(format!("unsupported character in expression: {ch}")),
            }
            i += 1;
        }
    }
    normalize_expression_tokens(&mut out, assignment, dim, vars);
    Ok(out)
}

fn normalize_expression_tokens(tokens: &mut [u8], assignment: bool, dim: bool, vars: &[String]) {
    for idx in 0..tokens.len() {
        if tokens[idx] == 0x2b {
            let prev = idx.checked_sub(1).and_then(|i| tokens.get(i)).copied();
            tokens[idx] = match prev {
                Some(t) if dim && t >= VAR_TOKEN_BASE => 0x3b,
                Some(0x42) => 0x3a,
                Some(t) if t >= VAR_TOKEN_BASE => 0x37,
                _ => 0x2b,
            };
        } else if tokens[idx] == 0x12 {
            let in_subscript = tokens[..idx].iter().rev().any(|&t| t == 0x37);
            if in_subscript {
                tokens[idx] = 0x3c;
            }
        } else if tokens[idx] == 0x22 {
            let prev = idx.checked_sub(1).and_then(|i| tokens.get(i)).copied();
            tokens[idx] = if assignment {
                match prev.and_then(|t| variable_name_for_token(t, vars)) {
                    Some(name) if name.ends_with('$') => 0x2e,
                    Some(_) => 0x2d,
                    None if prev == Some(0x2c) => 0x2e,
                    None => 0x22,
                }
            } else {
                match prev {
                    Some(0x0f) | Some(0x2c) => 0x34,
                    Some(t)
                        if variable_name_for_token(t, vars)
                            .is_some_and(|name| name.ends_with('$')) =>
                    {
                        0x34
                    }
                    _ => 0x22,
                }
            };
        }
    }
}

fn variable_name_for_token(token: u8, vars: &[String]) -> Option<&str> {
    if token < VAR_TOKEN_BASE {
        return None;
    }
    vars.get(usize::from(token - VAR_TOKEN_BASE))
        .map(String::as_str)
}

fn expr_word_token(word: &str) -> Option<u8> {
    Some(match word {
        "TO" => 0x19,
        "STEP" => 0x1a,
        "THEN" => 0x1b,
        "NOT" => 0x28,
        "OR" => 0x29,
        "AND" => 0x2a,
        "STR$" => 0x3d,
        "CHR$" => 0x3e,
        "USR" => 0x3f,
        "ASC" => 0x40,
        "VAL" => 0x41,
        "LEN" => 0x42,
        "ADR" => 0x43,
        "ATN" => 0x44,
        "COS" => 0x45,
        "PEEK" => 0x46,
        "SIN" => 0x47,
        "RND" => 0x48,
        "FRE" => 0x49,
        "EXP" => 0x4a,
        "LOG" => 0x4b,
        "CLOG" => 0x4c,
        "SQR" => 0x4d,
        "SGN" => 0x4e,
        "ABS" => 0x4f,
        "INT" => 0x50,
        "PADDLE" => 0x51,
        "STICK" => 0x52,
        "PTRIG" => 0x53,
        "STRIG" => 0x54,
        _ => return None,
    })
}

fn is_expr_keyword(word: &str) -> bool {
    expr_word_token(word).is_some()
}

fn variable_type_byte(name: &str) -> u8 {
    if name.ends_with('$') {
        0x80
    } else if name.ends_with('(') {
        0x40
    } else {
        0x00
    }
}

fn skip_string(text: &str, start: usize) -> usize {
    let bytes = text.as_bytes();
    let mut i = start + 1;
    while i < bytes.len() {
        if bytes[i] == b'"' {
            return i + 1;
        }
        i += 1;
    }
    bytes.len()
}

fn read_word(text: &str, start: usize) -> (&str, usize) {
    let bytes = text.as_bytes();
    let mut i = start;
    while i < bytes.len() && (bytes[i] as char).is_ascii_alphanumeric() {
        i += 1;
    }
    if bytes.get(i) == Some(&b'$') {
        let word = text[start..i].to_ascii_uppercase();
        if matches!(word.as_str(), "STR" | "CHR") {
            i += 1;
        }
    }
    (&text[start..i], i)
}

fn read_number(text: &str, start: usize) -> (&str, usize) {
    let bytes = text.as_bytes();
    let mut i = start;
    while i < bytes.len() {
        let ch = bytes[i] as char;
        if ch.is_ascii_digit() || ch == '.' {
            i += 1;
        } else {
            break;
        }
    }
    (&text[start..i], i)
}

fn encode_number(text: &str) -> Result<[u8; 6]> {
    let mut raw = text.trim();
    let negative = raw.starts_with('-');
    if negative {
        raw = &raw[1..];
    }
    if raw == "0" || raw == "0.0" {
        return Ok([0; 6]);
    }
    let (whole, frac) = raw.split_once('.').unwrap_or((raw, ""));
    let whole_trimmed = whole.trim_start_matches('0');
    let mut digits = format!("{whole_trimmed}{frac}");
    while digits.ends_with('0') && !frac.is_empty() {
        digits.pop();
    }
    if digits.is_empty() || !digits.bytes().all(|b| b.is_ascii_digit()) {
        return Err(format!("invalid numeric literal: {text}"));
    }
    if digits.len() > 10 {
        return Err(format!("numeric literal has too many digits: {text}"));
    }
    let int_digits = whole_trimmed.len();
    let exponent = if int_digits == 0 {
        0
    } else {
        int_digits.div_ceil(2)
    };
    if int_digits % 2 == 1 {
        digits.insert(0, '0');
    }
    while digits.len() < 10 {
        digits.push('0');
    }
    let mut out = [0u8; 6];
    out[0] = 0x3f + exponent as u8;
    if negative {
        out[0] |= 0x80;
    }
    for idx in 0..5 {
        let hi = digits.as_bytes()[idx * 2] - b'0';
        let lo = digits.as_bytes()[idx * 2 + 1] - b'0';
        out[idx + 1] = (hi << 4) | lo;
    }
    Ok(out)
}

#[derive(Debug)]
struct AtariBasic {
    header: [u16; HEADER_WORDS],
    variables: Vec<String>,
    statements_offset: usize,
}

fn read_program(path: &Path) -> Result<(Vec<u8>, AtariBasic)> {
    let bytes = fs::read(path).map_err(|e| e.to_string())?;
    if bytes.len() < HEADER_LEN {
        return Err("file is too short for an Atari BASIC header".into());
    }
    let mut header = [0u16; HEADER_WORDS];
    for (idx, word) in header.iter_mut().enumerate() {
        let off = idx * 2;
        *word = u16::from_le_bytes([bytes[off], bytes[off + 1]]);
    }
    let vnt_start = HEADER_LEN;
    let vnt_end = ptr_to_file_offset(header[2])?;
    let statements_offset = ptr_to_file_offset(header[4])?;
    if vnt_end > bytes.len() || statements_offset > bytes.len() || vnt_start > vnt_end {
        return Err("header points outside the file".into());
    }
    let variables = parse_variables(&bytes[vnt_start..vnt_end]);
    Ok((
        bytes,
        AtariBasic {
            header,
            variables,
            statements_offset,
        },
    ))
}

fn ptr_to_file_offset(ptr: u16) -> Result<usize> {
    if ptr < FILE_LOMEM {
        return Err(format!("invalid Atari BASIC pointer ${ptr:04X}"));
    }
    Ok(HEADER_LEN + usize::from(ptr - FILE_LOMEM))
}

fn parse_variables(bytes: &[u8]) -> Vec<String> {
    let mut vars = Vec::new();
    let mut cur = String::new();
    for &byte in bytes {
        if byte == 0 {
            break;
        }
        let end = byte & 0x80 != 0;
        let ch = char::from(byte & 0x7f);
        cur.push(ch);
        if end {
            vars.push(std::mem::take(&mut cur));
        }
    }
    vars
}

pub(crate) fn detokenize_bytes(bytes: &[u8]) -> Result<String> {
    if bytes.len() < HEADER_LEN {
        return Err("file is too short for an Atari BASIC header".into());
    }
    let mut header = [0u16; HEADER_WORDS];
    for (idx, word) in header.iter_mut().enumerate() {
        let off = idx * 2;
        *word = u16::from_le_bytes([bytes[off], bytes[off + 1]]);
    }
    let vnt_start = HEADER_LEN;
    let vnt_end = ptr_to_file_offset(header[2])?;
    let statements_offset = ptr_to_file_offset(header[4])?;
    if vnt_end > bytes.len() || statements_offset > bytes.len() || vnt_start > vnt_end {
        return Err("header points outside the file".into());
    }
    let program = AtariBasic {
        header,
        variables: parse_variables(&bytes[vnt_start..vnt_end]),
        statements_offset,
    };
    let mut out = String::new();
    let mut offset = program.statements_offset;
    while offset + 3 < bytes.len() {
        let line_no = u16::from_le_bytes([bytes[offset], bytes[offset + 1]]);
        if line_no == 0x8000 {
            break;
        }
        let line_len = usize::from(bytes[offset + 2]);
        if line_len == 0 {
            break;
        }
        if offset + line_len > bytes.len() {
            return Err(format!("line {line_no} extends beyond EOF"));
        }
        let line = &bytes[offset..offset + line_len];
        out.push_str(&format!(
            "{line_no} {}",
            render_line(line, &program.variables)
        ));
        out.push('\n');
        offset += line_len;
    }
    Ok(out)
}

fn detokenize_file(path: &Path) -> Result<String> {
    let bytes = fs::read(path).map_err(|e| e.to_string())?;
    detokenize_bytes(&bytes)
}

fn render_line(line: &[u8], vars: &[String]) -> String {
    let mut rendered = String::new();
    let mut stmt_start = 4usize;
    let mut stmt_end = usize::from(line[3]);
    while stmt_start < line.len() && stmt_end <= line.len() && stmt_start < stmt_end {
        if !rendered.is_empty() && rendered.ends_with("THEN ") {
            while rendered.ends_with(' ') {
                rendered.pop();
            }
            rendered.push(' ');
        } else if !rendered.is_empty() {
            rendered.push(':');
        }
        let chunk = &line[stmt_start..stmt_end];
        if !chunk.is_empty() {
            rendered.push_str(&render_statement(chunk[0], &chunk[1..], vars));
        }
        stmt_start = stmt_end;
        if stmt_start < line.len() {
            stmt_end = usize::from(line[stmt_start]);
            stmt_start += 1;
        }
    }
    rendered
}

fn render_statement(stmt_token: u8, body: &[u8], vars: &[String]) -> String {
    match stmt_token {
        0 => render_rem_data("REM", body),
        1 => render_rem_data("DATA", body),
        54 => render_expr(body, vars),
        t => {
            let name = STATEMENTS.get(usize::from(t)).copied().unwrap_or("?");
            let rest = render_expr(body, vars);
            if rest.is_empty() || rest == "<EOL>" {
                name.to_string()
            } else if name == "?" {
                format!("? {rest}")
            } else {
                format!("{name} {rest}")
            }
        }
    }
}

fn render_rem_data(name: &str, body: &[u8]) -> String {
    let mut s = String::from(name);
    if !body.is_empty() {
        s.push(' ');
    }
    for &byte in body {
        if byte == 0x9b || byte == 0x16 {
            break;
        }
        s.push(atascii_to_char(byte));
    }
    s
}

fn render_expr(body: &[u8], vars: &[String]) -> String {
    let mut out = String::new();
    let mut i = 0usize;
    while i < body.len() {
        let token = body[i];
        i += 1;
        match token {
            0x0e if i + 6 <= body.len() => {
                push_atom(&mut out, &decode_number(&body[i..i + 6]));
                i += 6;
            }
            0x0f if i < body.len() => {
                let len = usize::from(body[i]);
                i += 1;
                let end = (i + len).min(body.len());
                let text: String = body[i..end].iter().map(|&b| atascii_to_char(b)).collect();
                push_atom(&mut out, &format!("\"{text}\""));
                i = end;
            }
            0x10 => out.push(','),
            0x12 => out.push(','),
            0x14 => break,
            0x15 => {
                if out.trim_end().ends_with("THEN") {
                    while out.ends_with(' ') {
                        out.pop();
                    }
                    out.push(' ');
                } else {
                    out.push(';');
                }
            }
            0x16 => break,
            0x17 => out.push(')'),
            0x19 => push_word(&mut out, "TO"),
            0x1a => push_word(&mut out, "STEP"),
            0x1b => push_word(&mut out, "THEN"),
            0x1c => out.push('#'),
            0x1d => push_op(&mut out, "<="),
            0x1e => push_op(&mut out, "<>"),
            0x1f => push_op(&mut out, ">="),
            0x20 => push_op(&mut out, "<"),
            0x21 => push_op(&mut out, ">"),
            0x22 => push_op(&mut out, "="),
            0x23 => push_op(&mut out, "^"),
            0x24 => push_op(&mut out, "*"),
            0x25 => push_op(&mut out, "+"),
            0x26 => push_op(&mut out, "-"),
            0x27 => push_op(&mut out, "/"),
            0x28 => push_word(&mut out, "NOT"),
            0x29 => push_word(&mut out, "OR"),
            0x2a => push_word(&mut out, "AND"),
            0x2b => out.push('('),
            0x2c => out.push(')'),
            0x2d => push_op(&mut out, "="),
            0x2e => push_op(&mut out, "="),
            0x2f => push_op(&mut out, "<="),
            0x30 => push_op(&mut out, "<>"),
            0x31 => push_op(&mut out, ">="),
            0x32 => push_op(&mut out, "<"),
            0x33 => push_op(&mut out, ">"),
            0x34 => push_op(&mut out, "="),
            0x37 => out.push('('),
            0x38 => out.push('('),
            0x39 => out.push('('),
            0x3a => out.push('('),
            0x3b => out.push('('),
            0x3c => out.push(','),
            0x3d => push_func(&mut out, "STR$"),
            0x3e => push_func(&mut out, "CHR$"),
            0x3f => push_func(&mut out, "USR"),
            0x40 => push_func(&mut out, "ASC"),
            0x41 => push_func(&mut out, "VAL"),
            0x42 => push_func(&mut out, "LEN"),
            0x43 => push_func(&mut out, "ADR"),
            0x44 => push_func(&mut out, "ATN"),
            0x45 => push_func(&mut out, "COS"),
            0x46 => push_func(&mut out, "PEEK"),
            0x47 => push_func(&mut out, "SIN"),
            0x48 => push_func(&mut out, "RND"),
            0x49 => push_func(&mut out, "FRE"),
            0x4a => push_func(&mut out, "EXP"),
            0x4b => push_func(&mut out, "LOG"),
            0x4c => push_func(&mut out, "CLOG"),
            0x4d => push_func(&mut out, "SQR"),
            0x4e => push_func(&mut out, "SGN"),
            0x4f => push_func(&mut out, "ABS"),
            0x50 => push_func(&mut out, "INT"),
            0x51 => push_func(&mut out, "PADDLE"),
            0x52 => push_func(&mut out, "STICK"),
            0x53 => push_func(&mut out, "PTRIG"),
            0x54 => push_func(&mut out, "STRIG"),
            t if t >= VAR_TOKEN_BASE => {
                let idx = usize::from(t - VAR_TOKEN_BASE);
                push_atom(&mut out, vars.get(idx).map(String::as_str).unwrap_or("<?>"));
            }
            other => push_atom(&mut out, &format!("<${other:02X}>")),
        }
    }
    out
}

fn push_atom(out: &mut String, atom: &str) {
    out.push_str(atom);
}

fn push_op(out: &mut String, op: &str) {
    out.push_str(op);
}

fn push_word(out: &mut String, word: &str) {
    if !out.is_empty() && !out.ends_with([' ', '(', ',', ';', ':']) {
        out.push(' ');
    }
    out.push_str(word);
    if !word.ends_with('(') {
        out.push(' ');
    }
}

fn push_func(out: &mut String, word: &str) {
    if !out.is_empty() && !out.ends_with([' ', '(', ',', ';', ':', '=', '+', '-', '*', '/', '^']) {
        out.push(' ');
    }
    out.push_str(word);
}

fn atascii_to_char(byte: u8) -> char {
    match byte {
        0x9b => '\n',
        0x20..=0x7e => char::from(byte),
        _ => '.',
    }
}

fn decode_number(bytes: &[u8]) -> String {
    if bytes.iter().all(|&b| b == 0) {
        return "0".into();
    }
    let negative = bytes[0] & 0x80 != 0;
    let exponent = i16::from(bytes[0] & 0x7f) - 0x3f;
    let mut digits = String::new();
    for &byte in &bytes[1..] {
        digits.push(char::from(b'0' + ((byte >> 4) & 0x0f)));
        digits.push(char::from(b'0' + (byte & 0x0f)));
    }
    let int_digits = (exponent * 2).max(0) as usize;
    let mut text = if int_digits >= digits.len() {
        let mut s = digits;
        s.push_str(&"0".repeat(int_digits - s.len()));
        s
    } else if int_digits == 0 {
        format!("0.{}", digits)
    } else {
        let (whole, frac) = digits.split_at(int_digits);
        format!("{whole}.{frac}")
    };
    while text.contains('.') && text.ends_with('0') {
        text.pop();
    }
    if text.ends_with('.') {
        text.pop();
    }
    while text.starts_with('0') && text.len() > 1 && !text.starts_with("0.") {
        text.remove(0);
    }
    if negative {
        text.insert(0, '-');
    }
    text
}

fn inspect_file(path: &Path) -> Result<()> {
    let (bytes, program) = read_program(path)?;
    println!("file: {}", path.display());
    println!("bytes: {}", bytes.len());
    println!("header:");
    for (name, ptr) in [
        "VNTP", "VNTD", "VVTP", "STMTAB", "STMCUR", "STARP", "RUNSTK",
    ]
    .iter()
    .zip(program.header)
    {
        let off = if ptr >= FILE_LOMEM {
            format!("{}", HEADER_LEN + usize::from(ptr - FILE_LOMEM))
        } else {
            "-".into()
        };
        println!("  {name:<6} ${ptr:04X} file_offset={off}");
    }
    println!("variables: {}", program.variables.len());
    for (idx, name) in program.variables.iter().enumerate() {
        println!("  ${:02X} {}", VAR_TOKEN_BASE + idx as u8, name);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decodes_atari_bcd_numbers() {
        assert_eq!(decode_number(&[0, 0, 0, 0, 0, 0]), "0");
        assert_eq!(decode_number(&[0x40, 0x01, 0, 0, 0, 0]), "1");
        assert_eq!(decode_number(&[0x40, 0x90, 0, 0, 0, 0]), "90");
        assert_eq!(decode_number(&[0x41, 0x10, 0x04, 0, 0, 0]), "1004");
        assert_eq!(decode_number(&[0x3f, 0x50, 0, 0, 0, 0]), "0.5");
        assert_eq!(decode_number(&[0xc0, 0x12, 0x34, 0, 0, 0]), "-12.34");
    }

    #[test]
    fn decodes_variable_name_table() {
        assert_eq!(
            parse_variables(&[
                b'M',
                b'B',
                b'A',
                b'S' | 0x80,
                b'N',
                b'$' | 0x80,
                b'A',
                b'(' | 0x80,
            ]),
            vec!["MBAS".to_string(), "N$".to_string(), "A(".to_string()]
        );
    }

    #[test]
    fn splits_then_statement() {
        assert_eq!(
            logical_statements(r#"IF S=55 THEN PRINT "PASS":GOTO 100"#),
            vec![
                LogicalStatement {
                    text: r#"IF S=55 THEN "#,
                    terminator: StatementTerminator::None,
                },
                LogicalStatement {
                    text: r#"PRINT "PASS""#,
                    terminator: StatementTerminator::Colon,
                },
                LogicalStatement {
                    text: "GOTO 100",
                    terminator: StatementTerminator::Eol,
                },
            ]
        );
    }
}
