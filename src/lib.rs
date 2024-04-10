use std::io::{self, Read, Write};

/// When writing bytes, all bytes are assumed valid utf-8 char(s).
struct FilterWriter<'a, W: Write> {
    /// If true, write only ASCII letters, ASCII punctuations, ASCII digits,
    /// space, tab, and '\n'.
    ascii_only: bool,
    backend: &'a mut W,
}

impl<'a, W: Write> FilterWriter<'a, W> {
    fn new(backend: &'a mut W, ascii_only: bool) -> Self {
        Self {
            ascii_only,
            backend,
        }
    }
}

impl<'a, W: Write> Write for FilterWriter<'a, W> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        if self.ascii_only {
            let utf8_buf = unsafe { std::str::from_utf8_unchecked(buf) };
            let mut written: usize = 0;
            for (j, c) in utf8_buf.char_indices() {
                let c_len = c.len_utf8();
                if c_len > 1 {
                    // `c` is not ASCII. Drop directly.
                    written += c_len;
                } else {
                    let c_byte = buf[j];
                    if (c_byte < 11 && c_byte >= 9)
                        || (c_byte < 127 && c_byte >= 32)
                    {
                        self.backend.write_all(&buf[j..j + 1]).unwrap();
                        written += 1;
                    } else {
                        // `c` is not in the ASCII subset. Drop directly.
                        written += 1;
                    }
                }
            }
            Ok(written)
        } else {
            self.backend.write(buf)
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        self.backend.flush()
    }
}

/// Attempt to group bytes into valid utf-8 chars and write them to writer.
/// `taken_limit` is used to upper bound the bytes taken. Return the number of
/// bytes actually taken, which is larger than or equal to `taken_limit`.
///
/// Dynamic programming is used to find the solution.
///
/// Arguments:
///
/// - `cbuf`: buffer
/// - `m`: data size, where m <= cbuf.len()
/// - `taken_limit`: the index of the last char taken <= this
/// - `w`: writer to write utf-8 chars
fn take_from_buffer<W: Write>(
    cbuf: &[u8],
    m: usize,
    taken_limit: usize,
    w: &mut W,
) -> usize {
    let mut cost: Vec<usize> = vec![0; m + 1];
    let mut backtrack: Vec<usize> = vec![0; m];
    // valid_utf8[(m + 1) * i + j - (i + 2) * (i + 1) / 2] = true if cbuf[i..j]
    // is a valid utf-8 char(s).
    let mut valid_utf8: Vec<bool> = vec![false; (m + 1) * m - (m + 1) * m / 2];
    for i in (0..m).rev() {
        let mut min_cost_i = usize::MAX;
        for j in i + 1..=m {
            // check if cbuf[i..j] is valid utf-8 char(s)
            let valid_utf8_ij = std::str::from_utf8(&cbuf[i..j]).is_ok();
            valid_utf8[(m + 1) * i + j - (i + 2) * (i + 1) / 2] = valid_utf8_ij;
            // update min_cost_i & backtrack_i
            let cost_ij = if valid_utf8_ij { 0 } else { j - i };
            let cost_j = cost[j];
            if cost_ij + cost_j < min_cost_i {
                min_cost_i = cost_ij + cost_j;
                backtrack[i] = j;
            }
        }
        cost[i] = min_cost_i;
    }

    let mut i: usize = 0;
    while i <= taken_limit && i < m {
        let j = backtrack[i];
        let valid_utf8_ij = valid_utf8[(m + 1) * i + j - (i + 2) * (i + 1) / 2];
        if valid_utf8_ij {
            w.write_all(&cbuf[i..j]).unwrap();
        }
        i = j;
    }

    i
}

/// Return `Ok(())` if `buf` is filled. Return `Err(n)` if EOF is reached the
/// `buf` is not filled -- only `n` bytes are read in.
fn fill_buf<R: Read>(buf: &mut [u8], r: &mut R) -> Result<(), usize> {
    let byte = buf.len();
    let mut in_bytes_total: usize = 0;
    while in_bytes_total < byte {
        let in_bytes = r.read(&mut buf[in_bytes_total..]).unwrap();
        if in_bytes == 0 {
            return Err(in_bytes_total);
        }
        in_bytes_total += in_bytes;
    }

    Ok(())
}

fn buffer_filter<R: Read, W: Write>(
    buf_size: usize,
    mut taken_limit: usize,
    r: &mut R,
    w: &mut W,
) {
    let mut buf = vec![0u8; buf_size];
    let mut m = match fill_buf(&mut buf, r) {
        Ok(()) => buf_size,
        Err(n) => {
            taken_limit = n;
            n
        }
    };
    while m > 0 {
        let taken = take_from_buffer(&buf, m, taken_limit, w);
        buf.copy_within(taken..m, 0);
        m = match fill_buf(&mut buf[m - taken..], r) {
            Ok(()) => buf_size,
            Err(n) => {
                taken_limit = m - taken + n;
                taken_limit
            }
        };
    }
}

pub fn stdin_stdout_buffer_filter(buf_size: usize, ascii_only: bool) {
    let mut stdin = io::stdin();
    let mut stdout = io::stdout();
    let mut fw = FilterWriter::new(&mut stdout, ascii_only);
    buffer_filter(buf_size, buf_size / 2, &mut stdin, &mut fw);
}

#[cfg(test)]
mod tests {
    use crate::{fill_buf, take_from_buffer, FilterWriter};
    use std::io::{Cursor, Write};

    #[test]
    fn test_take_from_buffer() {
        let mut w: Vec<u8> = Vec::new();
        assert_eq!(take_from_buffer(b"abcdef", 5, 2, &mut w), 3);
        assert_eq!(w, vec![b'a', b'b', b'c']);
    }

    #[test]
    fn test_fill_buf() {
        let mut buf = vec![0u8; 5];
        let data = vec![b'h', b'e', b'l'];
        let mut r = Cursor::new(data);
        assert_eq!(fill_buf(&mut buf, &mut r), Err(3));

        let mut buf = vec![0u8; 3];
        let data = vec![b'h', b'e', b'l', b'l'];
        let mut r = Cursor::new(data);
        assert_eq!(fill_buf(&mut buf, &mut r), Ok(()));
    }

    #[test]
    fn test_filter_writer() {
        let mut w: Vec<u8> = Vec::new();
        let mut fw = FilterWriter::new(&mut w, true);
        write!(fw, "abc你好 wor").unwrap();
        assert_eq!(w, vec![b'a', b'b', b'c', b' ', b'w', b'o', b'r']);
    }
}
