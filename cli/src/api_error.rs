#[derive(Debug)]
pub struct ApiError {
    pub status: u16,
    pub body: String,
}

impl std::fmt::Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.body.is_empty() {
            write!(f, "HTTP {}", self.status)
        } else {
            write!(f, "{}: {}", self.status, self.body)
        }
    }
}

impl std::error::Error for ApiError {}

impl ApiError {
    #[must_use]
    pub fn is_fatal(&self) -> bool {
        matches!(self.status, 401 | 403)
    }
}
