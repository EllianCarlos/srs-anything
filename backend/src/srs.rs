use std::{collections::HashMap, fs};

use chrono::Duration;
use serde::{Deserialize, Serialize};

pub const DEFAULT_CONFIG_PATH: &str = "config/srs_schedule.yaml";
pub const DEFAULT_PROD_INTERVALS: [i64; 5] = [1, 3, 7, 14, 30];

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Grade {
    Again,
    Hard,
    Good,
    Easy,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum IntervalUnit {
    Days,
    Seconds,
    Minutes,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ScheduleProfile {
    pub unit: IntervalUnit,
    pub intervals: Vec<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SrsScheduleFile {
    pub active_profile: String,
    pub profiles: HashMap<String, ScheduleProfile>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SrsSchedule {
    profile: ScheduleProfile,
}

impl SrsSchedule {
    pub fn prod_default() -> Self {
        Self {
            profile: ScheduleProfile {
                unit: IntervalUnit::Days,
                intervals: DEFAULT_PROD_INTERVALS.to_vec(),
            },
        }
    }

    pub fn from_profile(profile: ScheduleProfile) -> Option<Self> {
        let valid =
            !profile.intervals.is_empty() && profile.intervals.iter().all(|value| *value > 0);
        valid.then_some(Self { profile })
    }

    pub fn from_file_and_profile(
        file: &SrsScheduleFile,
        profile_name: Option<&str>,
    ) -> Option<Self> {
        let selected = profile_name.unwrap_or(&file.active_profile);
        let profile = file.profiles.get(selected)?.clone();
        Self::from_profile(profile)
    }

    pub fn max_index(&self) -> usize {
        self.profile.intervals.len() - 1
    }

    pub fn duration_for_index(&self, index: usize) -> Duration {
        let value = self.profile.intervals[index.min(self.max_index())];
        match self.profile.unit {
            IntervalUnit::Days => Duration::days(value),
            IntervalUnit::Seconds => Duration::seconds(value),
            IntervalUnit::Minutes => Duration::minutes(value),
        }
    }
}

impl Default for SrsSchedule {
    fn default() -> Self {
        Self::prod_default()
    }
}

pub fn next_interval_index(current_index: usize, grade: Grade, max_index: usize) -> usize {
    match grade {
        Grade::Again => 0,
        Grade::Hard => current_index.saturating_sub(1),
        Grade::Good => (current_index + 1).min(max_index),
        Grade::Easy => (current_index + 2).min(max_index),
    }
}

pub fn load_schedule(config_path: Option<&str>, profile_name: Option<&str>) -> SrsSchedule {
    let path = config_path.unwrap_or(DEFAULT_CONFIG_PATH);
    let file = fs::read_to_string(path)
        .ok()
        .and_then(|contents| serde_yaml::from_str::<SrsScheduleFile>(&contents).ok());
    file.as_ref()
        .and_then(|parsed| SrsSchedule::from_file_and_profile(parsed, profile_name))
        .unwrap_or_else(SrsSchedule::prod_default)
}

#[cfg(test)]
mod tests {
    use super::{
        Grade, IntervalUnit, ScheduleProfile, SrsSchedule, SrsScheduleFile, load_schedule,
        next_interval_index,
    };
    use std::{collections::HashMap, fs};

    #[test]
    fn resets_on_again() {
        assert_eq!(next_interval_index(4, Grade::Again, 4), 0);
    }

    #[test]
    fn hard_moves_back() {
        assert_eq!(next_interval_index(3, Grade::Hard, 4), 2);
        assert_eq!(next_interval_index(0, Grade::Hard, 4), 0);
    }

    #[test]
    fn easy_jumps_forward() {
        assert_eq!(next_interval_index(1, Grade::Easy, 4), 3);
        assert_eq!(next_interval_index(4, Grade::Easy, 4), 4);
    }

    #[test]
    fn maps_profile_duration_days() {
        let schedule = SrsSchedule::from_profile(ScheduleProfile {
            unit: IntervalUnit::Days,
            intervals: vec![1, 3, 7],
        })
        .expect("valid schedule");
        assert_eq!(schedule.duration_for_index(0).num_days(), 1);
        assert_eq!(schedule.duration_for_index(2).num_days(), 7);
        assert_eq!(schedule.duration_for_index(9).num_days(), 7);
    }

    #[test]
    fn maps_profile_duration_minutes() {
        let schedule = SrsSchedule::from_profile(ScheduleProfile {
            unit: IntervalUnit::Minutes,
            intervals: vec![1, 3, 5],
        })
        .expect("valid schedule");
        assert_eq!(schedule.duration_for_index(1).num_minutes(), 3);
    }

    #[test]
    fn maps_profile_duration_seconds() {
        let schedule = SrsSchedule::from_profile(ScheduleProfile {
            unit: IntervalUnit::Seconds,
            intervals: vec![1, 3, 5],
        })
        .expect("valid schedule");
        assert_eq!(schedule.duration_for_index(2).num_seconds(), 5);
    }

    #[test]
    fn resolves_schedule_from_yaml_file() {
        let mut profiles = HashMap::new();
        profiles.insert(
            "prod".to_owned(),
            ScheduleProfile {
                unit: IntervalUnit::Days,
                intervals: vec![1, 3, 7],
            },
        );
        profiles.insert(
            "test".to_owned(),
            ScheduleProfile {
                unit: IntervalUnit::Seconds,
                intervals: vec![1, 3, 5],
            },
        );
        let yaml = serde_yaml::to_string(&SrsScheduleFile {
            active_profile: "prod".to_owned(),
            profiles,
        })
        .expect("serialize yaml");
        let path = std::env::temp_dir().join("srs_schedule_test.yaml");
        fs::write(&path, yaml).expect("write schedule file");

        let test_profile = load_schedule(path.to_str(), Some("test"));
        assert_eq!(test_profile.duration_for_index(2).num_seconds(), 5);

        let default_profile = load_schedule(path.to_str(), None);
        assert_eq!(default_profile.duration_for_index(2).num_days(), 7);

        fs::remove_file(path).expect("cleanup temp schedule file");
    }

    #[test]
    fn falls_back_to_prod_on_missing_or_unknown_profile() {
        let unknown_profile = load_schedule(Some("/no/such/path.yaml"), Some("unknown"));
        assert_eq!(unknown_profile.duration_for_index(0).num_days(), 1);
    }
}
