//! The protected-copy v1 contract, shared with its writer. Receipts are
//! unauthenticated local consistency inputs, never fresh hardware evidence.
use std::collections::BTreeMap;
pub const SCHEMA: &str = "fmr-protected-copies-v1";
pub const MAX_PROVENANCE: usize = 1024 * 1024;
// A control byte can expand to six JSON bytes; leave room for the fixed v1
// envelope. Every receipt the existing writer accepts must fit this bound.
pub const MAX_RECEIPT: usize = 7 * 1024 * 1024;
#[derive(Debug, PartialEq)]
enum Value { Text(String), Number(u64), Bool(bool) }
#[derive(Debug)]
pub struct Receipt { pub length: u64, pub sha256: String, pub stage: &'static str }

struct Json<'a> { text: &'a str, at: usize }
impl Json<'_> {
    fn ws(&mut self) { while self.text.as_bytes().get(self.at).is_some_and(|b| matches!(b, b' ' | b'\n' | b'\r' | b'\t')) { self.at += 1; } }
    fn consume(&mut self, b: u8) -> Result<(), ()> {
        self.ws();
        if self.text.as_bytes().get(self.at) != Some(&b) { return Err(()); }
        self.at += 1; Ok(())
    }
    fn hex4(&mut self) -> Result<u32, ()> {
        let end = self.at.checked_add(4).ok_or(())?;
        let s = self.text.get(self.at..end).ok_or(())?;
        if !s.bytes().all(|b| b.is_ascii_hexdigit()) { return Err(()); }
        self.at = end;
        u32::from_str_radix(s, 16).map_err(|_| ())
    }
    fn string(&mut self) -> Result<String, ()> {
        self.consume(b'"')?;
        let mut out = String::new();
        loop {
            let ch = self.text.get(self.at..).and_then(|s| s.chars().next()).ok_or(())?;
            self.at += ch.len_utf8();
            match ch {
                '"' => return Ok(out),
                '\\' => {
                    let escape = *self.text.as_bytes().get(self.at).ok_or(())?;
                    self.at += 1;
                    out.push(match escape {
                        b'"' => '"', b'\\' => '\\', b'/' => '/', b'b' => '\u{8}',
                        b'f' => '\u{c}', b'n' => '\n', b'r' => '\r', b't' => '\t',
                        b'u' => {
                            let mut code = self.hex4()?;
                            if (0xD800..=0xDBFF).contains(&code) {
                                if self.text.as_bytes().get(self.at..self.at + 2) != Some(b"\\u") { return Err(()); }
                                self.at += 2;
                                let low = self.hex4()?;
                                if !(0xDC00..=0xDFFF).contains(&low) { return Err(()); }
                                code = 0x10000 + ((code - 0xD800) << 10) + low - 0xDC00;
                            }
                            char::from_u32(code).ok_or(())?
                        }
                        _ => return Err(()),
                    });
                }
                c if c < ' ' => return Err(()),
                c => out.push(c),
            }
        }
    }
    fn value(&mut self) -> Result<Value, ()> {
        self.ws();
        match self.text.as_bytes().get(self.at) {
            Some(b'"') => self.string().map(Value::Text),
            Some(b't') if self.text[self.at..].starts_with("true") => { self.at += 4; Ok(Value::Bool(true)) }
            Some(b'f') if self.text[self.at..].starts_with("false") => { self.at += 5; Ok(Value::Bool(false)) }
            Some(b'0'..=b'9') => {
                let start = self.at;
                while self.text.as_bytes().get(self.at).is_some_and(u8::is_ascii_digit) { self.at += 1; }
                let s = &self.text[start..self.at];
                if s.len() > 1 && s.starts_with('0') { return Err(()); }
                s.parse().map(Value::Number).map_err(|_| ())
            }
            _ => Err(()), // v1 is a flat object; no recursive/unbounded values.
        }
    }
    fn object(&mut self) -> Result<BTreeMap<String, Value>, ()> {
        self.consume(b'{')?;
        let mut map = BTreeMap::new();
        loop {
            let key = self.string()?;
            self.consume(b':')?;
            let value = self.value()?;
            if map.insert(key, value).is_some() { return Err(()); }
            self.ws();
            if self.text.as_bytes().get(self.at) == Some(&b'}') { self.at += 1; break; }
            self.consume(b',')?;
        }
        self.ws();
        if self.at != self.text.len() { return Err(()); }
        Ok(map)
    }
}
impl Receipt {
    pub fn parse(bytes: &[u8]) -> Result<Self, &'static str> {
        let parse = || -> Result<Self, ()> {
            if bytes.len() > MAX_RECEIPT { return Err(()); }
            let text = std::str::from_utf8(bytes).map_err(|_| ())?;
            let mut fields = Json { text, at: 0 }.object()?;
            let mut take = |key: &str| fields.remove(key).ok_or(());
            if take("schema")? != Value::Text(SCHEMA.into()) { return Err(()); }
            let stage = match take("stage")? {
                Value::Text(s) if s == "storage-only" => "storage-only-bound-locally",
                Value::Text(s) if s == "final-held-session-checkpoint" => "held-session-receipt-bound-locally",
                _ => return Err(()),
            };
            let length = match take("length")? { Value::Number(n) if n == 0x100000 => n, _ => return Err(()) };
            let sha256 = match take("sha256")? {
                Value::Text(s) if s.len() == 64 && s.bytes().all(|b| b.is_ascii_hexdigit()) => s.to_ascii_lowercase(),
                _ => return Err(()),
            };
            if take("copies")? != Value::Number(2) { return Err(()); }
            for name in ["independent_media", "physical_media_independence_verified", "returned_to_stock"] {
                if take(name)? != Value::Bool(false) { return Err(()); }
            }
            let different = take("different_filesystems")?;
            let policy = take("policy")?;
            if !matches!((&different, &policy),
                (Value::Bool(true), Value::Text(p)) if p == "different-filesystems" || p == "separate-directories")
                && !matches!((&different, &policy), (Value::Bool(false), Value::Text(p)) if p == "separate-directories") {
                return Err(());
            }
            for name in ["first_filesystem", "second_filesystem", "first_directory_inode", "second_directory_inode"] {
                if !matches!(take(name)?, Value::Number(_)) { return Err(()); }
            }
            if !matches!(take("provenance_json")?, Value::Text(s) if s.len() <= MAX_PROVENANCE) { return Err(()); }
            if !fields.is_empty() { return Err(()); }
            Ok(Self { length, sha256, stage })
        };
        parse().map_err(|_| "invalid-protected-copy-receipt")
    }
    pub fn bound(&self, len: usize, sha256: &str) -> bool {
        self.length == len as u64 && self.sha256 == sha256
    }
}
