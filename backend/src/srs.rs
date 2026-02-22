use serde::{Deserialize, Serialize};

pub const INTERVALS_DAYS: [i64; 5] = [1, 3, 7, 14, 30];

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Grade {
    Again,
    Hard,
    Good,
    Easy,
}

pub fn next_interval_index(current_index: usize, grade: Grade) -> usize {
    let max = INTERVALS_DAYS.len() - 1;
    match grade {
        Grade::Again => 0,
        Grade::Hard => current_index.saturating_sub(1),
        Grade::Good => (current_index + 1).min(max),
        Grade::Easy => (current_index + 2).min(max),
    }
}

pub fn interval_days_from_index(index: usize) -> i64 {
    INTERVALS_DAYS[index.min(INTERVALS_DAYS.len() - 1)]
}

#[cfg(test)]
mod tests {
    use super::{Grade, interval_days_from_index, next_interval_index};

    #[test]
    fn resets_on_again() {
        assert_eq!(next_interval_index(4, Grade::Again), 0);
    }

    #[test]
    fn hard_moves_back() {
        assert_eq!(next_interval_index(3, Grade::Hard), 2);
        assert_eq!(next_interval_index(0, Grade::Hard), 0);
    }

    #[test]
    fn easy_jumps_forward() {
        assert_eq!(next_interval_index(1, Grade::Easy), 3);
        assert_eq!(next_interval_index(4, Grade::Easy), 4);
    }

    #[test]
    fn maps_days() {
        assert_eq!(interval_days_from_index(0), 1);
        assert_eq!(interval_days_from_index(2), 7);
        assert_eq!(interval_days_from_index(9), 30);
    }
}
