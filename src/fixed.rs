//! Fixed-capacity text and lists for per-frame hook paths (0.10.7): no
//! allocation, no drop glue. Text past the capacity is cut at a character
//! boundary and items past it are dropped, both counted in [`truncated`], so a
//! too-small capacity shows as a short label, never as a fault.
//!
//! `tools/hook_wait_check.py` fails when a per-frame hook reaches the
//! allocator; these replace `String`, `format!` and `Vec` on those paths.

use std::fmt;
use std::mem::MaybeUninit;
use std::ops::{Deref, DerefMut};
use std::sync::atomic::{AtomicU64, Ordering};

static TRUNCATED: AtomicU64 = AtomicU64::new(0);

/// Text cut or items dropped for capacity since boot.
pub fn truncated() -> u64 {
    TRUNCATED.load(Ordering::Relaxed)
}

fn note_truncated() {
    TRUNCATED.fetch_add(1, Ordering::Relaxed);
}

/// A label or value: a menu cell, a plugin choice (the ABI's 24 bytes), a ping.
pub type Label = FixedStr<48>;
/// One line of the window: a footer, a strip line, joined facts.
pub type Line = FixedStr<256>;

/// Formats into a [`FixedStr`] whose capacity the context gives.
#[macro_export]
macro_rules! fixed {
    ($($arg:tt)*) => {{
        let mut text = $crate::fixed::FixedStr::new();
        let _ = ::core::fmt::Write::write_fmt(&mut text, ::core::format_args!($($arg)*));
        text
    }};
}

/// UTF-8 text of at most `N` bytes, stored inline.
#[derive(Clone, Copy)]
pub struct FixedStr<const N: usize> {
    len: usize,
    bytes: [u8; N],
}

impl<const N: usize> FixedStr<N> {
    pub const fn new() -> Self {
        Self { len: 0, bytes: [0; N] }
    }

    /// `text`, cut at the last character boundary that fits.
    pub fn from_str_lossy(text: &str) -> Self {
        let mut out = Self::new();
        out.push_str(text);
        out
    }

    pub fn as_str(&self) -> &str {
        // SAFETY: only whole UTF-8 characters are ever copied in.
        unsafe { std::str::from_utf8_unchecked(&self.bytes[..self.len]) }
    }

    pub const fn capacity(&self) -> usize {
        N
    }

    pub fn clear(&mut self) {
        self.len = 0;
    }

    /// Appends what fits, cut at a character boundary; `false` when cut.
    pub fn push_str(&mut self, text: &str) -> bool {
        let room = N - self.len;
        let take = if text.len() <= room {
            text.len()
        } else {
            let mut end = room;
            while !text.is_char_boundary(end) {
                end -= 1;
            }
            end
        };
        self.bytes[self.len..self.len + take].copy_from_slice(&text.as_bytes()[..take]);
        self.len += take;
        if take < text.len() {
            note_truncated();
            return false;
        }
        true
    }

    pub fn push(&mut self, ch: char) -> bool {
        self.push_str(ch.encode_utf8(&mut [0; 4]))
    }

    /// Keeps the first `len` bytes, backing up to a character boundary.
    pub fn truncate(&mut self, len: usize) {
        if len < self.len {
            let mut end = len;
            while !self.as_str().is_char_boundary(end) {
                end -= 1;
            }
            self.len = end;
        }
    }

    /// The text followed by a NUL, for C callers; `None` when it does not fit
    /// or the text holds a NUL.
    pub fn with_nul<const M: usize>(&self) -> Option<FixedStr<M>> {
        if self.len >= M || self.as_str().contains('\0') {
            return None;
        }
        let mut out = FixedStr::<M>::new();
        out.push_str(self.as_str());
        out.push('\0');
        Some(out)
    }
}

impl<const N: usize> Default for FixedStr<N> {
    fn default() -> Self {
        Self::new()
    }
}

impl<const N: usize> Deref for FixedStr<N> {
    type Target = str;
    fn deref(&self) -> &str {
        self.as_str()
    }
}

impl<const N: usize> AsRef<str> for FixedStr<N> {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl<const N: usize> std::borrow::Borrow<str> for FixedStr<N> {
    fn borrow(&self) -> &str {
        self.as_str()
    }
}

impl<const N: usize> fmt::Write for FixedStr<N> {
    /// Never `Err`: text past the capacity is cut and counted, and the rest of
    /// the format still runs.
    fn write_str(&mut self, text: &str) -> fmt::Result {
        self.push_str(text);
        Ok(())
    }
}

impl<const N: usize> fmt::Display for FixedStr<N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.pad(self.as_str())
    }
}

impl<const N: usize> fmt::Debug for FixedStr<N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self.as_str(), f)
    }
}

impl<const N: usize> From<&str> for FixedStr<N> {
    fn from(text: &str) -> Self {
        Self::from_str_lossy(text)
    }
}

impl<const N: usize, const M: usize> PartialEq<FixedStr<M>> for FixedStr<N> {
    fn eq(&self, other: &FixedStr<M>) -> bool {
        self.as_str() == other.as_str()
    }
}

impl<const N: usize> Eq for FixedStr<N> {}

impl<const N: usize> PartialEq<str> for FixedStr<N> {
    fn eq(&self, other: &str) -> bool {
        self.as_str() == other
    }
}

impl<const N: usize> PartialEq<&str> for FixedStr<N> {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}

impl<const N: usize> PartialEq<String> for FixedStr<N> {
    fn eq(&self, other: &String) -> bool {
        self.as_str() == other.as_str()
    }
}

impl<const N: usize> PartialEq<FixedStr<N>> for &str {
    fn eq(&self, other: &FixedStr<N>) -> bool {
        *self == other.as_str()
    }
}

impl<const N: usize> PartialEq<FixedStr<N>> for str {
    fn eq(&self, other: &FixedStr<N>) -> bool {
        self == other.as_str()
    }
}

impl<const N: usize> std::hash::Hash for FixedStr<N> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.as_str().hash(state);
    }
}

/// At most `N` `Copy` items, stored inline.
pub struct FixedVec<T: Copy, const N: usize> {
    len: usize,
    items: [MaybeUninit<T>; N],
}

impl<T: Copy, const N: usize> Clone for FixedVec<T, N> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T: Copy, const N: usize> Copy for FixedVec<T, N> {}

impl<T: Copy, const N: usize> FixedVec<T, N> {
    pub const fn new() -> Self {
        Self {
            len: 0,
            items: [const { MaybeUninit::uninit() }; N],
        }
    }

    /// Appends `item`; `false` (counted) when full.
    pub fn push(&mut self, item: T) -> bool {
        if self.len == N {
            note_truncated();
            return false;
        }
        self.items[self.len] = MaybeUninit::new(item);
        self.len += 1;
        true
    }

    pub fn pop(&mut self) -> Option<T> {
        if self.len == 0 {
            return None;
        }
        self.len -= 1;
        // SAFETY: items below `len` are initialized.
        Some(unsafe { self.items[self.len].assume_init() })
    }

    pub fn clear(&mut self) {
        self.len = 0;
    }

    pub fn truncate(&mut self, len: usize) {
        self.len = self.len.min(len);
    }

    pub fn as_slice(&self) -> &[T] {
        // SAFETY: items below `len` are initialized.
        unsafe { std::slice::from_raw_parts(self.items.as_ptr().cast::<T>(), self.len) }
    }

    pub fn as_mut_slice(&mut self) -> &mut [T] {
        // SAFETY: items below `len` are initialized.
        unsafe { std::slice::from_raw_parts_mut(self.items.as_mut_ptr().cast::<T>(), self.len) }
    }

    pub fn retain(&mut self, mut keep: impl FnMut(&T) -> bool) {
        let mut kept = 0;
        for i in 0..self.len {
            // SAFETY: items below `len` are initialized.
            let item = unsafe { self.items[i].assume_init() };
            if keep(&item) {
                self.items[kept] = MaybeUninit::new(item);
                kept += 1;
            }
        }
        self.len = kept;
    }
}

impl<T: Copy, const N: usize> Default for FixedVec<T, N> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Copy, const N: usize> Deref for FixedVec<T, N> {
    type Target = [T];
    fn deref(&self) -> &[T] {
        self.as_slice()
    }
}

impl<T: Copy, const N: usize> DerefMut for FixedVec<T, N> {
    fn deref_mut(&mut self) -> &mut [T] {
        self.as_mut_slice()
    }
}

impl<'a, T: Copy, const N: usize> IntoIterator for &'a FixedVec<T, N> {
    type Item = &'a T;
    type IntoIter = std::slice::Iter<'a, T>;
    fn into_iter(self) -> Self::IntoIter {
        self.as_slice().iter()
    }
}

impl<T: Copy, const N: usize> FromIterator<T> for FixedVec<T, N> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let mut out = Self::new();
        out.extend(iter);
        out
    }
}

impl<T: Copy, const N: usize> Extend<T> for FixedVec<T, N> {
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        for item in iter {
            if !self.push(item) {
                break;
            }
        }
    }
}

impl<T: Copy + fmt::Debug, const N: usize> fmt::Debug for FixedVec<T, N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.as_slice()).finish()
    }
}

impl<T: Copy + PartialEq, const N: usize> PartialEq for FixedVec<T, N> {
    fn eq(&self, other: &Self) -> bool {
        self.as_slice() == other.as_slice()
    }
}

impl<T: Copy + Eq, const N: usize> Eq for FixedVec<T, N> {}

/// `parts` separated by `sep`, as one text.
pub fn join<const N: usize, S: AsRef<str>>(parts: &[S], sep: &str) -> FixedStr<N> {
    let mut out = FixedStr::new();
    for (i, part) in parts.iter().enumerate() {
        if i > 0 {
            out.push_str(sep);
        }
        out.push_str(part.as_ref());
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn text_is_cut_at_a_character_boundary_and_counted() {
        let before = truncated();
        let text: FixedStr<5> = FixedStr::from("abテd");
        assert_eq!(text, "abテ");
        assert!(truncated() > before);
        let text: FixedStr<4> = FixedStr::from("abテ");
        assert_eq!(text, "ab");
    }

    #[test]
    fn formatting_fills_what_fits() {
        let text: FixedStr<16> = fixed!("Auto ({}f) {:>3}", 3, "x");
        assert_eq!(text, "Auto (3f)   x");
        let cut: FixedStr<6> = fixed!("{}{}", "abcd", 12345);
        assert_eq!(cut, "abcd12");
    }

    #[test]
    fn a_list_keeps_order_and_drops_past_its_capacity() {
        let mut list: FixedVec<u8, 3> = (1..=5).collect();
        assert_eq!(list.as_slice(), &[1, 2, 3]);
        assert!(!list.push(9));
        list.retain(|v| *v != 2);
        assert_eq!(list.as_slice(), &[1, 3]);
        assert_eq!(list.pop(), Some(3));
        assert_eq!(list.len(), 1);
    }

    #[test]
    fn join_and_nul() {
        let parts: [FixedStr<8>; 3] = [FixedStr::from("a"), FixedStr::from("bc"), FixedStr::from("")];
        let joined: FixedStr<16> = join(&parts, "   ");
        assert_eq!(joined, "a   bc   ");
        let c: FixedStr<8> = FixedStr::<8>::from("ab").with_nul().unwrap();
        assert_eq!(c.as_bytes(), b"ab\0");
        assert!(FixedStr::<8>::from("abcdefgh").with_nul::<8>().is_none());
    }
}
