use std::fmt;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Side {
    Buy,
    Sell
}

impl fmt::Display for Side {
    fn fmt(&self, f: &mut fmt:: Formatter) -> fmt::Result {
        match *self {
            Side::Buy => write!(f, "Buy"),
            Side::Sell => write!(f, "Sell")
        }
    }
}
