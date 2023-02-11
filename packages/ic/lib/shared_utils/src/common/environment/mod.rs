use std::time::Duration;

use ic_cdk::timer::TimerId;

pub trait ExecutionEnvironment {
    fn set_timer_interval(interval: Duration, func: impl FnMut() + 'static) -> TimerId;
}
