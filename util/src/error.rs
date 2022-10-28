use {
    plotters::drawing::DrawingAreaErrorKind,
    std::{error::Error, fmt, io, num::TryFromIntError},
};

#[derive(Debug)]
pub enum BfError {
    Bf(String),
    DrawingArea(String),
    TryFromInt(TryFromIntError),
    Io(io::Error),
}

impl fmt::Display for BfError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Bf(s) => write!(f, "{s}"),
            Self::DrawingArea(s) => write!(f, "{s}"),
            Self::TryFromInt(err) => write!(f, "{err}"),
            Self::Io(err) => write!(f, "{err}"),
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

impl From<io::Error> for BfError {
    fn from(err: io::Error) -> Self {
        BfError::Io(err)
    }
}
