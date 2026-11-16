use crate::{Error, Score};

#[derive(Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct WideScore {
    pub hard: i128,
    pub soft: i128,
}

impl WideScore {
    pub fn add(self, other: Self) -> Result<Self, Error> {
        Ok(Self {
            hard: self
                .hard
                .checked_add(other.hard)
                .ok_or(Error::ScoreOverflow)?,
            soft: self
                .soft
                .checked_add(other.soft)
                .ok_or(Error::ScoreOverflow)?,
        })
    }

    pub fn remove(self, other: Self) -> Self {
        Self {
            hard: self.hard - other.hard,
            soft: self.soft - other.soft,
        }
    }

    pub fn narrow(self) -> Result<Score, Error> {
        Ok(Score {
            hard: i64::try_from(self.hard).map_err(|_| Error::ScoreOverflow)?,
            soft: i64::try_from(self.soft).map_err(|_| Error::ScoreOverflow)?,
        })
    }
}
