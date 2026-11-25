/// A byte range in the source. Start is inclusive. End is exclusive.
///
/// Both offsets must be within the source and on UTF-8 character boundaries.
/// Start must not exceed end. Equal offsets form an empty span.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

impl Span {
    /// Creates a span without validating its offsets.
    pub const fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }
}
