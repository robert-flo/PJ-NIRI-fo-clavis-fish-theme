use crate::config::Config;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PromptState {
    pub status: i32,
    /// Kept as part of the renderer state so Fish pipeline information is not
    /// lost while prompt internals run. The current visual contract uses the
    /// aggregate Fish status for arrow colouring.
    pub pipestatus: Vec<i32>,
    pub duration_ms: Option<u64>,
    pub command_seen: bool,
}

impl PromptState {
    pub fn failed(&self) -> bool {
        self.status != 0 || self.pipestatus.iter().any(|status| *status != 0)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Target {
    Full,
    TransientLeft,
    TransientRight,
}

pub fn render(config: &Config, target: Target, state: &PromptState, columns: usize) -> String {
    match target {
        Target::Full => crate::layout::render_full(config, state, columns),
        Target::TransientLeft => crate::layout::render_transient_left(config, state, columns),
        Target::TransientRight => crate::layout::render_transient_right(config, columns),
    }
}

#[cfg(test)]
mod tests {
    use super::PromptState;

    #[test]
    fn pipeline_failure_is_not_lost_when_last_status_is_zero() {
        let state = PromptState {
            status: 0,
            pipestatus: vec![1, 0],
            duration_ms: Some(0),
            command_seen: true,
        };
        assert!(state.failed());
    }
}
