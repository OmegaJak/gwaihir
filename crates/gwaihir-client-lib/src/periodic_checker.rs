use std::time::{Duration, Instant};

pub struct PeriodicChecker<R> {
    last_check_time: Instant,
    min_time_between_checks: Duration,
    get_check_result: Box<dyn FnMut() -> R>,
    last_check_result: R,
}

pub trait HasPeriodicChecker<R> {
    fn periodic_checker(&self) -> &PeriodicChecker<R>;
    fn periodic_checker_mut(&mut self) -> &mut PeriodicChecker<R>;
}

impl<R> PeriodicChecker<R> {
    pub fn check(&mut self) {
        let now = Instant::now();
        if now.duration_since(self.last_check_time) > self.min_time_between_checks {
            let result = (self.get_check_result)();
            self.last_check_time = now;
            self.last_check_result = result;
        }
    }
}

impl<R> PeriodicChecker<R>
where
    R: Default,
{
    pub fn new(get_check_result: Box<dyn FnMut() -> R>, min_time_between_checks: Duration) -> Self {
        Self {
            last_check_time: Instant::now() - min_time_between_checks,
            min_time_between_checks,
            get_check_result,
            last_check_result: Default::default(),
        }
    }
}

impl<R> PeriodicChecker<R>
where
    R: Clone,
{
    pub fn last_check_result(&self) -> R {
        self.last_check_result.clone()
    }
}
