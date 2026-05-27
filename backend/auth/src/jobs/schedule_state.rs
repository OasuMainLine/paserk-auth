use std::sync::Arc;

use apalis::prelude::Data;

use crate::config::Config;

#[derive(Clone, Debug)]
pub struct InnerScheduleState {
    pub redis: redis::Client,
    pub config: Config,
}

pub type ScheduleState = Data<Arc<InnerScheduleState>>;
