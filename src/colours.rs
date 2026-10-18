#[derive(Debug, Clone)]
pub struct Pallet<const N: usize> {
    pub colours: [[u8; 3]; N],
}
#[derive(Copy, Clone)]
#[repr(transparent)]
struct AsciiChar(pub u8);
impl std::fmt::Display for AsciiChar {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.0 {
            b' '..=b'~' => write!(f, "{}", self.0 as char),
            _ => write!(f, "\\x{:02X}", self.0),
        }
    }
}
#[derive(Debug, Copy, Clone)]
pub struct InvalidHexDigitError(pub u8);
#[derive(Debug, Clone)]
pub enum InvalidColorCodeError {
    InvalidDigit(InvalidHexDigitError),
    BadLength,
    BadFirstChar(u8),
}
impl std::error::Error for InvalidHexDigitError {}
impl std::fmt::Display for InvalidHexDigitError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "invalid hex digit '{}'", AsciiChar(self.0))
    }
}
impl std::error::Error for InvalidColorCodeError {}
impl std::fmt::Display for InvalidColorCodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use InvalidColorCodeError::*;
        match self {
            InvalidDigit(e) => e.fmt(f),
            BadLength => f.write_str(
                "incorrect length for colour code, should be 6 bytes or 7 with starting '#'",
            ),
            BadFirstChar(c) => {
                write!(
                    f,
                    "expected '#' as the first character, got '{}'",
                    AsciiChar(*c)
                )
            }
        }
    }
}
const fn nibble_from_hex(char: u8) -> Result<u8, InvalidHexDigitError> {
    match char {
        b'a'..=b'f' => Ok(char - b'a' + 10),
        b'A'..=b'F' => Ok(char - b'A' + 10),
        b'0'..=b'9' => Ok(char - b'0'),
        _ => Err(InvalidHexDigitError(char)),
    }
}
const fn byte_from_hex(str: [u8; 2]) -> Result<u8, InvalidHexDigitError> {
    let [upper, lower] = match str.try_map(nibble_from_hex) {
        Ok(x) => x,
        Err(e) => return Err(e),
    };
    Ok((upper << 4) | lower)
}
pub const fn color_from_hex(hex_code: &[u8]) -> Result<[u8; 3], InvalidColorCodeError> {
    let hex: &[u8; 6] = match hex_code {
        [b'#', tail @ ..] if tail.len() == 6 => tail.as_array().unwrap(),
        entire if entire.len() == 6 => entire.as_array().unwrap(),
        [x, tail @ ..] if tail.len() == 6 => return Err(InvalidColorCodeError::BadFirstChar(*x)),
        _ => return Err(InvalidColorCodeError::BadLength),
    };

    let hex: &[[u8; 2]; 3] = unsafe { hex.as_chunks_unchecked() }.as_array().unwrap();
    match hex.try_map(byte_from_hex) {
        Ok(x) => Ok(x),
        Err(e) => Err(InvalidColorCodeError::InvalidDigit(e)),
    }
}
impl<const N: usize> Pallet<N> {
    pub const fn from(hex_codes: [&[u8]; N]) -> Result<Self, InvalidColorCodeError> {
        Ok(Pallet {
            colours: match hex_codes.try_map(color_from_hex) {
                Ok(x) => x,
                Err(e) => return Err(e),
            },
        })
    }
    pub const fn get(&self, val: u8) -> [u8; 3] {
        let index_f = (val as f64 / 255.) * (N - 1) as f64;
        let index_f_trunc = index_f.trunc();
        let index = index_f_trunc as usize;
        let t = index_f - index_f_trunc;
        if t < 1e-10 {
            return self.colours[index];
        }
        let a = self.colours[index];
        let b = self.colours[index + 1];
        [
            (a[0] as f64 * (1. - t) + b[0] as f64 * t) as u8,
            (a[1] as f64 * (1. - t) + b[1] as f64 * t) as u8,
            (a[2] as f64 * (1. - t) + b[2] as f64 * t) as u8,
        ]
    }
}
