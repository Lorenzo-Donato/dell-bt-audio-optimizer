use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CircuitState {
    Closed,   // Normal healthy operation
    Open,     // Tripped - safety break activated, operations halted
    HalfOpen, // Testing cautiously after cooldown
}

pub struct CircuitBreaker {
    state: CircuitState,
    failure_count: u32,
    failure_threshold: u32,
    last_failure_time: Option<Instant>,
    cooldown_duration: Duration,
}

impl CircuitBreaker {
    pub fn new(failure_threshold: u32, cooldown_duration: Duration) -> Self {
        Self {
            state: CircuitState::Closed,
            failure_count: 0,
            failure_threshold,
            last_failure_time: None,
            cooldown_duration,
        }
    }

    /// Default airbag configuration: 3 failures trip the circuit breaker with a 60-second cooldown
    pub fn default_airbag() -> Self {
        Self::new(3, Duration::from_secs(60))
    }

    /// Checks if execution is permitted under current health conditions
    pub fn can_execute(&mut self) -> bool {
        match self.state {
            CircuitState::Closed => true,
            CircuitState::Open => {
                if let Some(last) = self.last_failure_time {
                    if last.elapsed() >= self.cooldown_duration {
                        self.state = CircuitState::HalfOpen;
                        true
                    } else {
                        false
                    }
                } else {
                    false
                }
            }
            CircuitState::HalfOpen => true,
        }
    }

    /// Records a successful operation, resetting the failure counter
    pub fn record_success(&mut self) {
        self.failure_count = 0;
        self.state = CircuitState::Closed;
    }

    /// Records a failure. If threshold is reached, trips the circuit breaker
    pub fn record_failure(&mut self) {
        self.failure_count += 1;
        self.last_failure_time = Some(Instant::now());

        if self.failure_count >= self.failure_threshold {
            self.state = CircuitState::Open;
        }
    }

    #[allow(dead_code)]
    pub fn is_tripped(&self) -> bool {
        matches!(self.state, CircuitState::Open)
    }

    #[allow(dead_code)]
    pub fn state(&self) -> CircuitState {
        self.state
    }

    pub fn remaining_cooldown(&self) -> Option<Duration> {
        if self.state == CircuitState::Open
            && let Some(last) = self.last_failure_time
        {
            return self.cooldown_duration.checked_sub(last.elapsed());
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_circuit_breaker_normal_flow() {
        let mut cb = CircuitBreaker::new(2, Duration::from_millis(50));
        assert!(cb.can_execute());
        cb.record_success();
        assert_eq!(cb.state(), CircuitState::Closed);
    }

    #[test]
    fn test_circuit_breaker_tripping_airbag() {
        let mut cb = CircuitBreaker::new(2, Duration::from_millis(100));
        assert!(cb.can_execute());

        cb.record_failure();
        assert_eq!(cb.state(), CircuitState::Closed);
        assert!(cb.can_execute());

        cb.record_failure(); // Threshold reached
        assert_eq!(cb.state(), CircuitState::Open);
        assert!(!cb.can_execute(), "Airbag deve impedir execução quando disparado");

        // Wait for cooldown
        std::thread::sleep(Duration::from_millis(110));
        assert!(cb.can_execute(), "Deve permitir tentativa HalfOpen após cooldown");
        assert_eq!(cb.state(), CircuitState::HalfOpen);

        // Success resets it
        cb.record_success();
        assert_eq!(cb.state(), CircuitState::Closed);
    }
}
