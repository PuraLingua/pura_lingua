#[cfg(test)]
mod tests;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Formatting<'a> {
    pub pieces: Vec<Piece<'a>>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Piece<'a> {
    Raw(&'a str),
    Placeholder(Placeholder<'a>),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Placeholder<'a> {
    pub index: u32,
    pub width: Option<i32>,
    pub fmt_string: &'a str,
}

#[derive(thiserror::Error, derive_more::Display, Debug, PartialEq, Eq)]
pub enum Error<'a> {
    NoClosingBraceFor(usize),
    NoOpeningBraceFor(usize),

    ParseIntFailed(#[from] std::num::ParseIntError),
    UnknownPlaceholder(&'a str),
}

impl<'a> Formatting<'a> {
    // Partially translated from Zig's Io.Writer.print(https://codeberg.org/ziglang/zig/src/commit/845b6a8efe5ac71c9df02373fe3716814cb7b92d/lib/std/Io/Writer.zig#L616)
    pub fn parse(source: &'a str) -> Result<Self, Error<'a>> {
        let mut pieces = Vec::new();

        let mut i = 0;

        loop {
            let start_index = i;

            while i < source.len() {
                match source.as_bytes()[i] {
                    b'{' | b'}' => break,
                    _ => (),
                }
                i += 1;
            }

            let mut end_index = i;
            let mut unescape_brace = false;

            // Handle {{ and }}
            if (i + 1 < source.len()) && (source.as_bytes()[i + 1] == source.as_bytes()[i]) {
                unescape_brace = true;
                // skip first brace
                end_index += 1;
                // skip both braces
                i += 2;
            }

            if unescape_brace {
                pieces.push(Piece::Raw(&source[start_index..end_index]));
                continue;
            } else if start_index != end_index {
                if let literal = &source[start_index..end_index]
                    && !literal.is_empty()
                {
                    pieces.push(Piece::Raw(literal));
                }

                continue;
            }

            if i >= source.len() {
                break;
            }

            if source.as_bytes()[i] == b'}' {
                return Err(Error::NoOpeningBraceFor(i));
            }

            debug_assert_eq!(source.as_bytes()[i], b'{');
            i += 1;

            let fmt_begin = i;
            while i < source.len() && source.as_bytes()[i] != b'}' {
                i += 1;
            }
            let fmt_end = i;

            if i >= source.len() {
                return Err(Error::NoClosingBraceFor(fmt_begin - 1));
            }

            debug_assert_eq!(source.as_bytes()[i], b'}');
            i += 1;

            let placeholder = &source[fmt_begin..fmt_end];
            pieces.push(Piece::Placeholder(Placeholder::parse(placeholder)?));
        }

        Ok(Self { pieces })
    }
}

impl<'a> Placeholder<'a> {
    pub fn parse(source: &'a str) -> Result<Self, Error<'a>> {
        let mut is_rest_format_string = false;
        let (index, rest) = match source.split_once(',') {
            Some(x) => x,
            None => match source.split_once(':') {
                Some(x) => {
                    is_rest_format_string = true;
                    x
                }
                None => {
                    return Ok(Self {
                        index: source.parse()?,
                        width: None,
                        fmt_string: "",
                    });
                }
            },
        };
        let index = index.parse::<u32>()?;
        if is_rest_format_string {
            return Ok(Self {
                index,
                width: None,
                fmt_string: rest,
            });
        }

        let Some((width, fmt_string)) = rest.split_once(':') else {
            return Ok(Self {
                index,
                width: Some(rest.parse()?),
                fmt_string: "",
            });
        };

        Ok(Self {
            index,
            width: Some(width.parse()?),
            fmt_string,
        })
    }
}
