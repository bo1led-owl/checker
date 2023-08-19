use termion::color;

pub struct CheckerError {
    pub error: String,
    pub error_description: Option<String>,
}

impl std::fmt::Display for CheckerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(error_description) = &self.error_description {
            write!(
                f,
                "{}{}\n{}{}",
                color::Fg(color::Red),
                self.error,
                color::Fg(color::Reset),
                error_description
            )
        } else {
            write!(f, "{}{}", color::Fg(color::Red), self.error)
        }
    }
}
