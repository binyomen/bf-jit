use {
    plotters::drawing::DrawingAreaErrorKind,
    std::{error::Error, fmt, num::TryFromIntError},
};

#[derive(Debug)]
pub enum BfError {
    Bf(String),
    DrawingArea(String),
    TryFromInt(TryFromIntError),
}

impl fmt::Display for BfError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Bf(s) => write!(f, "{s}"),
            Self::DrawingArea(s) => write!(f, "{s}"),
            Self::TryFromInt(err) => write!(f, "{err}"),
        }
    }
}

impl Error for BfError {}

impl<E> From<DrawingAreaErrorKind<E>> for BfError
where
    E: Error + Send + Sync,
{
    fn from(err: DrawingAreaErrorKind<E>) -> Self {
        BfError::DrawingArea(format!("{err}"))
    }
}

impl From<TryFromIntError> for BfError {
    fn from(err: TryFromIntError) -> Self {
        BfError::TryFromInt(err)
    }
}
