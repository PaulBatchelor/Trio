use std::collections::HashMap;

#[allow(dead_code)]
#[derive(Default, Debug, PartialEq)]
pub struct SchedulerState {
    pitch: u32,
    pub time: u32,
    gate: bool,
}

#[allow(dead_code)]
#[derive(Default)]
struct Trigger {
    trig: bool,
}

#[allow(dead_code)]
#[derive(Clone, Copy, Eq, Hash, PartialEq)]
pub enum EventType {
    /// Lower voice has been turned on,
    LowerOn,
    /// Lower voice is on, and needs to change pitch,
    LowerChange,
    /// Upper voice has been turned on
    UpperOn,
    /// Upper voice is on, and needs to change pitch
    UpperChange,
}

#[allow(dead_code)]
#[derive(Default, Debug, PartialEq)]
pub struct AutoVoiceState {
    upper: bool,
    lower: bool,
}

#[allow(dead_code)]
impl Trigger {
    pub fn fire(&mut self) {
        self.trig = true;
    }

    pub fn has_fired(&mut self) -> bool {
        self.trig
    }

    pub fn reset(&mut self) {
        self.trig = false;
    }

    pub fn check(&mut self) -> bool {
        let fired = self.has_fired();
        self.reset();
        fired
    }
}

#[allow(dead_code)]
#[derive(Default)]
pub struct VoiceScheduler {
    pub state: SchedulerState,
    autovoice: AutoVoiceState,
    //hooks: Hooks,
    hooks: HashMap<EventType, Trigger>,
}

#[allow(dead_code)]
impl VoiceScheduler {
    pub fn on(&mut self) {
        let state = &mut self.state;
        state.gate = true;
        state.time = 0;
    }

    pub fn off(&mut self) {
        let state = &mut self.state;
        let av = &mut self.autovoice;

        state.gate = false;
        state.time = 0;

        av.upper = false;
        av.lower = false;
    }

    pub fn tick(&mut self) {
        let state = &mut self.state;

        if !state.gate {
            return;
        }

        let av = &mut self.autovoice;

        if state.time == 1 {
            if av.lower {
                // TODO: abstract this?
                if let Some(e) = self.hooks.get_mut(&EventType::LowerChange) {
                    e.fire();
                }
            } else {
                av.lower = true;
                if let Some(e) = self.hooks.get_mut(&EventType::LowerOn) {
                    e.fire();
                }
            }
        } else if state.time == 2 {
            if av.upper {
                // TODO: abstract this?
                if let Some(e) = self.hooks.get_mut(&EventType::UpperChange) {
                    e.fire();
                }
            } else {
                av.upper = true;
                if let Some(e) = self.hooks.get_mut(&EventType::UpperOn) {
                    e.fire();
                }
            }
        }

        state.time += 1;
    }

    pub fn change(&mut self, pitch: u32) {
        let state = &mut self.state;
        if state.gate && state.pitch != pitch {
            state.pitch = pitch;
            state.time = 0;
        }
    }

    pub fn pop_next_event(&mut self) -> Option<EventType> {
        for (evt, hook) in self.hooks.iter_mut() {
            let s = hook.check();
            if s {
                return Some(*evt);
            }
        }

        None
    }

    pub fn populate_hooks(&mut self) {
        self.hooks.insert(EventType::LowerOn, Trigger::default());
        self.hooks
            .insert(EventType::LowerChange, Trigger::default());
        self.hooks.insert(EventType::UpperOn, Trigger::default());
        self.hooks
            .insert(EventType::UpperChange, Trigger::default());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_defaults() {
        let ss = SchedulerState::default();

        let cmp = SchedulerState {
            pitch: 0,
            time: 0,
            gate: false,
        };

        assert_eq!(&ss, &cmp);
    }

    #[test]
    fn test_on() {
        let mut vs = VoiceScheduler::default();

        let cmp = SchedulerState {
            pitch: 0,
            time: 0,
            gate: true,
        };

        vs.on();
        assert_eq!(&vs.state, &cmp);

        // make sure time resets when "on" is called

        vs.state.time = 4;

        vs.on();
        assert_eq!(&vs.state, &cmp);
    }

    #[test]
    fn test_tick() {
        let mut vs = VoiceScheduler::default();

        let cmp = SchedulerState {
            pitch: 0,
            time: 0,
            gate: false,
        };

        // only tick when gate is on
        vs.tick();
        assert_eq!(&vs.state, &cmp);

        let mut vs = VoiceScheduler::default();
        let cmp = SchedulerState {
            pitch: 0,
            time: 1,
            gate: true,
        };

        vs.on();
        vs.tick();
        assert_eq!(&vs.state, &cmp);
    }

    #[test]
    fn test_off() {
        let mut vs = VoiceScheduler::default();

        let cmp = SchedulerState {
            pitch: 0,
            time: 0,
            gate: false,
        };

        let av = AutoVoiceState {
            upper: false,
            lower: false,
        };

        vs.on();
        vs.tick();
        vs.tick();
        vs.tick();
        vs.tick();

        vs.off();
        assert_eq!(&vs.state, &cmp);
        assert_eq!(&vs.autovoice, &av);
    }

    #[test]
    fn test_change() {
        let mut vs = VoiceScheduler::default();

        let cmp = SchedulerState {
            pitch: 0,
            time: 0,
            gate: false,
        };

        // do not change on gate off
        vs.change(60);
        assert_eq!(&vs.state, &cmp, "Should not set pitch on gate off");

        // note on and change
        let cmp = SchedulerState {
            pitch: 60,
            time: 0,
            gate: true,
        };
        vs.on();
        vs.change(60);
        assert_eq!(&vs.state, &cmp, "Note on change didn't work");

        // attempts to change pitch to current pitch shouldn't change
        // clock
        let cmp = SchedulerState {
            pitch: 60,
            time: 3,
            gate: true,
        };
        vs.tick();
        vs.tick();
        vs.tick();
        vs.change(60);
        assert_eq!(&vs.state, &cmp, "don't change clock if pitch is the same");

        let cmp = SchedulerState {
            pitch: 62,
            time: 0,
            gate: true,
        };
        vs.change(62);
        assert_eq!(&vs.state, &cmp, "did not change pitch as expected");

        // TODO: retriggers should work somehow, more state needed?
    }

    #[test]
    fn test_trigger() {
        let mut trig = Trigger::default();
        assert!(!trig.trig, "Default sanity check failed");

        assert!(!trig.has_fired(), "Trigger has not yet fired");

        trig.fire();

        assert!(trig.has_fired(), "Trigger was supposed to fire.");

        trig.reset();

        assert!(!trig.has_fired(), "Trigger did not reset as expected.");

        trig.fire();

        let fired = trig.check();

        assert!(fired, "Trigger was supposed to have fired");

        assert!(
            !trig.has_fired(),
            "Check() did not reset state as expected."
        );
    }

    #[test]
    fn test_autovoice() {
        let av = AutoVoiceState::default();

        assert!(!av.upper && !av.lower, "Sanity check failed");

        let mut vs = VoiceScheduler::default();
        vs.populate_hooks();

        vs.on();
        vs.tick();
        assert!(!av.lower, "Lower voice is supposed to be off");

        // two ticks needed for at least 1 full tick
        vs.tick();

        let av = &vs.autovoice;
        assert!(av.lower, "Lower voice is supposed to be on");
        assert!(!av.upper, "Upper voice is supposed to be off");

        vs.tick();
        let av = &vs.autovoice;

        assert!(av.lower, "Lower voice is supposed to be on");
        assert!(av.upper, "Upper voice is supposed to be on");

        vs.off();
        let av = &vs.autovoice;
        assert!(!av.lower, "Lower voice is supposed to be off");
        assert!(!av.upper, "Upper voice is supposed to be off");
    }

    fn pop_events(vs: &mut VoiceScheduler) {
        while vs.pop_next_event().is_some() {}
    }

    #[test]
    fn test_lower_initial() {
        let mut vs = VoiceScheduler::default();

        vs.populate_hooks();

        // Turn on lead before first tick
        vs.on();
        vs.change(60);
        vs.tick();

        let mut nevents = 0;

        while vs.pop_next_event().is_some() {
            nevents += 1;
        }

        assert_eq!(nevents, 0, "Expected no events on first tick");

        vs.tick();

        // After two ticks, expect lower voice to turn on
        let mut found_lower = false;

        while let Some(evt) = vs.pop_next_event() {
            found_lower = matches!(evt, EventType::LowerOn);
        }

        assert!(found_lower, "Expected lower voice event");

        vs.tick();

        // This lower voice should have been removed by this next tick
        let mut found_lower = false;

        while let Some(evt) = vs.pop_next_event() {
            found_lower = matches!(evt, EventType::LowerOn);
        }

        assert!(!found_lower, "Expected not to find lower voice event");
    }

    #[test]
    fn test_lower_changed() {
        let mut vs = VoiceScheduler::default();

        vs.populate_hooks();

        // Turn on lead before first tick
        vs.on();
        vs.change(60);

        // Tick 1
        vs.tick();
        pop_events(&mut vs);

        // Tick 2: expect lower voice to turn on
        vs.tick();
        pop_events(&mut vs);

        // Tick 3
        vs.tick();
        pop_events(&mut vs);

        // Tick 4
        vs.tick();

        // Change lower pitch
        pop_events(&mut vs);
        vs.change(65);

        // Tick 5
        vs.tick();
        pop_events(&mut vs);

        // Tick 6 lower pitch expected to change
        vs.tick();

        // This lower voice should have been removed by this next tick
        let mut found_lower = false;

        while let Some(evt) = vs.pop_next_event() {
            found_lower = matches!(evt, EventType::LowerChange);
        }

        assert!(found_lower, "Expected to find change lower voice event");
    }

    #[test]
    fn test_upper_initial() {
        let mut vs = VoiceScheduler::default();
        vs.populate_hooks();

        // Turn on lead before first tick
        vs.on();
        vs.change(60);
        pop_events(&mut vs);

        // tick 1
        vs.tick();
        pop_events(&mut vs);

        // tick 2: expect lower voice to turn on
        vs.tick();
        pop_events(&mut vs);

        // tick 3: expect upper voice to turn on
        vs.tick();
        let mut found = false;

        while let Some(evt) = vs.pop_next_event() {
            found = matches!(evt, EventType::UpperOn);
        }

        assert!(found, "Expected upper voice event");
    }

    #[test]
    fn test_upper_changed() {
        let mut vs = VoiceScheduler::default();
        vs.populate_hooks();

        // Turn on lead before first tick
        vs.on();
        vs.change(60);
        pop_events(&mut vs);

        // tick 1
        vs.tick();
        pop_events(&mut vs);

        // tick 2: expect lower voice to turn on
        vs.tick();
        pop_events(&mut vs);

        // tick 3: expect upper voice to turn on
        vs.tick();
        pop_events(&mut vs);

        // tick 4
        vs.tick();
        pop_events(&mut vs);

        vs.change(65);

        // tick 5: Pitch change has happened during previous tick
        vs.tick();
        pop_events(&mut vs);

        // tick 6: Lower voice change happens
        vs.tick();
        pop_events(&mut vs);

        // tick 7: Upper voice change now happens
        vs.tick();

        let mut found = false;

        while let Some(evt) = vs.pop_next_event() {
            found = matches!(evt, EventType::UpperChange);
        }

        assert!(found, "Expected upper voice change event");
    }
}
