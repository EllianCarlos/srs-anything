use crate::srs::{SrsSchedule, load_schedule};

pub trait ScheduleProvider: Send + Sync {
    fn load_schedule(&self) -> SrsSchedule;
}

#[derive(Clone, Default)]
pub struct EnvScheduleProvider;

impl ScheduleProvider for EnvScheduleProvider {
    fn load_schedule(&self) -> SrsSchedule {
        load_schedule(
            std::env::var("SRS_CONFIG_PATH").ok().as_deref(),
            std::env::var("SRS_PROFILE").ok().as_deref(),
        )
    }
}
