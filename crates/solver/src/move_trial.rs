use super::wide_score::WideScore;
use crate::{Error, IncrementalScorer, Score};

/// Holds a tentative score and assignment until explicitly committed.
pub struct MoveTrial<'state, 'compiled> {
    scorer: &'state mut IncrementalScorer<'compiled>,
    variable: usize,
    previous_value: i64,
    previous_total: WideScore,
    previous_scores: Vec<(usize, WideScore)>,
    committed: bool,
}

impl<'state, 'compiled> MoveTrial<'state, 'compiled> {
    pub(super) fn new(
        scorer: &'state mut IncrementalScorer<'compiled>,
        variable: usize,
        previous_value: i64,
        previous_total: WideScore,
        previous_scores: Vec<(usize, WideScore)>,
    ) -> Self {
        Self {
            scorer,
            variable,
            previous_value,
            previous_total,
            previous_scores,
            committed: false,
        }
    }

    pub fn values(&self) -> &[i64] {
        self.scorer.values()
    }

    pub fn score(&self) -> Result<Score, Error> {
        self.scorer.score()
    }

    pub(super) fn wide_score(&self) -> WideScore {
        self.scorer.total
    }

    pub fn is_feasible(&self) -> bool {
        self.scorer.is_feasible()
    }

    /// Keeps the tentative value and score instead of restoring them on drop.
    pub fn commit(mut self) {
        self.committed = true;
    }
}

impl Drop for MoveTrial<'_, '_> {
    fn drop(&mut self) {
        if self.committed {
            return;
        }

        self.scorer.values[self.variable] = self.previous_value;
        self.scorer.total = self.previous_total;
        for &(index, score) in &self.previous_scores {
            self.scorer.scores[index] = score;
        }
    }
}
