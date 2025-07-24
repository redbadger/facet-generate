/// This is a comment.
#[derive(Facet)]
pub enum Source {
    Embedded,
    GoogleFont,
    Custom,
    #[facet(other)]
    Unknown,
}
