use std::{
    ops::Add,
    time::{Duration, Instant},
};

pub struct BackoffExecutor {
    backoff: exponential_backoff::Backoff,
    current_retry_attempt: u32,
    next_attempt: Instant,
}

pub enum BackoffExecutionAction {
    KeepTrying,
    Reset,
}

impl BackoffExecutor {
    pub fn new(min_duration: Duration, max_duration: Duration) -> Self {
        Self {
            backoff: exponential_backoff::Backoff::new(u32::MAX, min_duration, max_duration),
            current_retry_attempt: 0,
            next_attempt: Instant::now(),
        }
    }

    pub fn maybe_execute(
        &mut self,
        mut action: impl FnMut() -> BackoffExecutionAction,
        now: Instant,
    ) {
        if now > self.next_attempt {
            let result = action();
            self.current_retry_attempt += 1;

            match result {
                BackoffExecutionAction::KeepTrying => {
                    let backoff_duration = self.backoff.next(self.current_retry_attempt);
                    self.next_attempt =
                        now.add(backoff_duration.unwrap_or_else(|| Duration::from_secs(100)));
                }
                BackoffExecutionAction::Reset => {
                    self.current_retry_attempt = 0;
                }
            }
        }
    }
}
