use serde::Deserialize;

use crate::srs::Grade;

#[derive(Debug, Deserialize)]
pub struct GradeRequest {
    pub grade: Grade,
}
