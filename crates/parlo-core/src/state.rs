#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum State {
    Idle,
    Armed,
    Listening,
    Paused,
    Processing,
    Inserting,
    ReadyToCopy,
    Success,
    Error,
    Cancelled,
}

impl State {
    pub fn parse(text: &str) -> Option<Self> {
        match text.trim().to_ascii_lowercase().as_str() {
            "idle" => Some(Self::Idle),
            "armed" => Some(Self::Armed),
            "listening" => Some(Self::Listening),
            "paused" => Some(Self::Paused),
            "processing" => Some(Self::Processing),
            "inserting" => Some(Self::Inserting),
            "ready-to-copy" => Some(Self::ReadyToCopy),
            "success" => Some(Self::Success),
            "error" => Some(Self::Error),
            "cancelled" => Some(Self::Cancelled),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Armed => "armed",
            Self::Listening => "listening",
            Self::Paused => "paused",
            Self::Processing => "processing",
            Self::Inserting => "inserting",
            Self::ReadyToCopy => "ready-to-copy",
            Self::Success => "success",
            Self::Error => "error",
            Self::Cancelled => "cancelled",
        }
    }

    pub fn can_move_to(self, next: Self) -> bool {
        use State::*;
        matches!(
            (self, next),
            (Idle, Armed | ReadyToCopy | Error)
                | (Armed, Listening | ReadyToCopy | Error | Cancelled)
                | (Listening, Paused | Processing | Error | Cancelled)
                | (Paused, Listening | Processing | Error | Cancelled)
                | (Processing, Inserting | ReadyToCopy | Error | Cancelled)
                | (ReadyToCopy, Armed | Idle | Error)
                | (Inserting, Success | Error | Cancelled)
                | (Success | Error | Cancelled, Idle | Armed)
        )
    }

    pub fn is_quiet(self) -> bool {
        matches!(
            self,
            Self::Idle | Self::Success | Self::Error | Self::Cancelled
        )
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Machine {
    state: State,
}

impl Machine {
    pub const fn new() -> Self {
        Self { state: State::Idle }
    }

    pub fn state(&self) -> State {
        self.state
    }

    pub fn enter(&mut self, next: State) -> Result<State, String> {
        if !self.state.can_move_to(next) {
            return Err(format!(
                "{} cannot follow {}",
                next.as_str(),
                self.state.as_str()
            ));
        }
        let previous = self.state;
        self.state = next;
        Ok(previous)
    }
}

impl Default for Machine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_name_round_trips() {
        for state in [
            State::Idle,
            State::Armed,
            State::Listening,
            State::Paused,
            State::Processing,
            State::Inserting,
            State::ReadyToCopy,
            State::Success,
            State::Error,
            State::Cancelled,
        ] {
            assert_eq!(State::parse(state.as_str()), Some(state));
        }
    }

    #[test]
    fn an_unknown_name_is_refused() {
        assert_eq!(State::parse("sleeping"), None);
    }

    #[test]
    fn the_happy_path_walks_from_idle_to_success() {
        let mut machine = Machine::new();
        for next in [
            State::Armed,
            State::Listening,
            State::Processing,
            State::Inserting,
            State::Success,
            State::Idle,
        ] {
            assert!(machine.enter(next).is_ok());
            assert_eq!(machine.state(), next);
        }
    }

    #[test]
    fn a_pause_holds_the_same_session_open() {
        let mut machine = Machine::new();
        machine.enter(State::Armed).unwrap();
        machine.enter(State::Listening).unwrap();
        assert_eq!(machine.enter(State::Paused), Ok(State::Listening));
        assert!(machine.enter(State::Success).is_err());
        assert_eq!(machine.enter(State::Listening), Ok(State::Paused));
        assert_eq!(machine.enter(State::Processing), Ok(State::Listening));
    }

    #[test]
    fn a_paused_dictation_is_not_quiet() {
        assert!(!State::Paused.is_quiet());
    }

    #[test]
    fn transcription_can_be_ready_to_copy_without_insertion() {
        let mut machine = Machine::new();
        for state in [
            State::Armed,
            State::Listening,
            State::Processing,
            State::ReadyToCopy,
            State::Armed,
        ] {
            assert!(machine.enter(state).is_ok());
        }
    }

    #[test]
    fn saved_recovery_and_load_failures_are_visible_at_startup() {
        let mut restored = Machine::new();
        assert!(restored.enter(State::ReadyToCopy).is_ok());
        assert!(restored.enter(State::Armed).is_ok());
        let mut failed = Machine::new();
        assert!(failed.enter(State::Error).is_ok());
        assert!(failed.enter(State::Armed).is_ok());
    }

    #[test]
    fn success_cannot_skip_insertion() {
        let mut machine = Machine::new();
        machine.enter(State::Armed).unwrap();
        machine.enter(State::Listening).unwrap();
        machine.enter(State::Processing).unwrap();
        assert!(machine.enter(State::Success).is_err());
        assert_eq!(machine.state(), State::Processing);
    }

    #[test]
    fn a_finished_state_only_returns_to_idle() {
        let mut machine = Machine::new();
        machine.enter(State::Armed).unwrap();
        machine.enter(State::Cancelled).unwrap();
        assert!(machine.enter(State::Listening).is_err());
        assert_eq!(machine.enter(State::Idle), Ok(State::Cancelled));
    }

    #[test]
    fn a_finished_state_can_start_another_dictation() {
        let mut machine = Machine::new();
        machine.enter(State::Armed).unwrap();
        machine.enter(State::Cancelled).unwrap();
        assert_eq!(machine.enter(State::Armed), Ok(State::Cancelled));
    }

    #[test]
    fn a_second_dictation_follows_a_success() {
        let mut machine = Machine::new();
        for next in [
            State::Armed,
            State::Listening,
            State::Processing,
            State::Inserting,
            State::Success,
            State::Armed,
        ] {
            assert!(machine.enter(next).is_ok(), "{next:?} was refused");
        }
    }

    #[test]
    fn cancel_is_reachable_from_every_working_state() {
        for start in [
            State::Armed,
            State::Listening,
            State::Paused,
            State::Processing,
            State::Inserting,
        ] {
            let mut machine = Machine::new();
            let mut state = State::Idle;
            for next in [
                State::Armed,
                State::Listening,
                State::Paused,
                State::Processing,
                State::Inserting,
            ] {
                if state == start {
                    break;
                }
                machine.enter(next).unwrap();
                state = next;
            }
            assert_eq!(machine.enter(State::Cancelled), Ok(start));
        }
    }

    #[test]
    fn quiet_states_are_the_four_that_can_settle() {
        assert!(State::Success.is_quiet());
        assert!(State::Cancelled.is_quiet());
        assert!(!State::Listening.is_quiet());
        assert!(!State::Processing.is_quiet());
    }
}
