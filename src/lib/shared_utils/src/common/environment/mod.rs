use std::time::Duration;

use ic_cdk_timers::TimerId;

pub trait ExecutionEnvironment {
    fn set_timer_interval(interval: Duration, func: impl FnMut() + 'static) -> TimerId;
}
