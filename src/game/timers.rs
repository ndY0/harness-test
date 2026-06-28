use crate::game::GhostMode;

const FRIGHTENED_TICKS: u32 = 100;
const FRIGHTENED_BLINK_AT: u32 = 30;
const SCATTER_TICKS: u32 = 70;
const CHASE_TICKS: u32 = 200;
const DEATH_PAUSE_TICKS: u32 = 10;
const EXTRA_LIFE_TICKS: u32 = 20;

#[derive(Debug, Clone)]
pub enum TimerEvent {
    FrightenedExpired,
    FrightenedBlinking,
    DeathPauseExpired,
    ExtraLifeDisplayExpired,
    PhaseChanged(GhostMode),
}

pub struct GameTimers {
    pub frightened_ticks: u32,
    pub scatter_ticks: u32,
    pub chase_ticks: u32,
    pub cycle_count: u8,
    pub death_pause_ticks: u32,
    pub extra_life_display_ticks: u32,
    pub current_phase: GhostMode,
    interrupted_phase_ticks: u32,
}

impl GameTimers {
    pub fn new() -> Self {
        GameTimers {
            frightened_ticks: 0,
            scatter_ticks: SCATTER_TICKS,
            chase_ticks: 0,
            cycle_count: 0,
            death_pause_ticks: 0,
            extra_life_display_ticks: 0,
            current_phase: GhostMode::Scatter,
            interrupted_phase_ticks: 0,
        }
    }

    pub fn tick(&mut self) -> Vec<TimerEvent> {
        let mut events = Vec::new();

        // Death pause takes priority — skip all other timer logic.
        if self.death_pause_ticks > 0 {
            self.death_pause_ticks -= 1;
            if self.death_pause_ticks == 0 {
                events.push(TimerEvent::DeathPauseExpired);
            }
            return events;
        }

        // Frightened countdown.
        if self.frightened_ticks > 0 {
            self.frightened_ticks -= 1;
            if self.frightened_ticks == FRIGHTENED_BLINK_AT {
                events.push(TimerEvent::FrightenedBlinking);
            }
            if self.frightened_ticks == 0 {
                events.push(TimerEvent::FrightenedExpired);
                self.end_frightened();
            }
        } else {
            // Scatter / chase cycle (only when not frightened).
            self.tick_phase(&mut events);
        }

        // Extra life display (independent of the above).
        if self.extra_life_display_ticks > 0 {
            self.extra_life_display_ticks -= 1;
            if self.extra_life_display_ticks == 0 {
                events.push(TimerEvent::ExtraLifeDisplayExpired);
            }
        }

        events
    }

    pub fn start_frightened(&mut self) {
        // Save remaining ticks for the active underlying phase.
        self.interrupted_phase_ticks = match self.current_phase {
            GhostMode::Scatter => self.scatter_ticks,
            GhostMode::Chase => self.chase_ticks,
            GhostMode::Frightened => self.interrupted_phase_ticks,
        };
        self.frightened_ticks = FRIGHTENED_TICKS;
    }

    pub fn end_frightened(&mut self) {
        // Restore the interrupted phase.
        match self.current_phase {
            GhostMode::Scatter => self.scatter_ticks = self.interrupted_phase_ticks,
            GhostMode::Chase => self.chase_ticks = self.interrupted_phase_ticks,
            GhostMode::Frightened => {}
        }
        self.frightened_ticks = 0;
    }

    pub fn start_death_pause(&mut self) {
        self.death_pause_ticks = DEATH_PAUSE_TICKS;
    }

    pub fn start_extra_life_display(&mut self) {
        self.extra_life_display_ticks = EXTRA_LIFE_TICKS;
    }

    pub fn is_in_death_pause(&self) -> bool {
        self.death_pause_ticks > 0
    }

    pub fn is_frightened(&self) -> bool {
        self.frightened_ticks > 0
    }

    pub fn frightened_ticks_remaining(&self) -> u32 {
        self.frightened_ticks
    }

    fn tick_phase(&mut self, events: &mut Vec<TimerEvent>) {
        // After 2 complete scatter→chase cycles, stay in Chase permanently.
        if self.cycle_count >= 2 {
            return;
        }

        match self.current_phase {
            GhostMode::Scatter => {
                if self.scatter_ticks > 0 {
                    self.scatter_ticks -= 1;
                    if self.scatter_ticks == 0 {
                        self.current_phase = GhostMode::Chase;
                        self.chase_ticks = CHASE_TICKS;
                        events.push(TimerEvent::PhaseChanged(GhostMode::Chase));
                    }
                }
            }
            GhostMode::Chase => {
                if self.chase_ticks > 0 {
                    self.chase_ticks -= 1;
                    if self.chase_ticks == 0 {
                        // Increment first; only switch back to Scatter if still < 2.
                        self.cycle_count += 1;
                        if self.cycle_count < 2 {
                            self.current_phase = GhostMode::Scatter;
                            self.scatter_ticks = SCATTER_TICKS;
                            events.push(TimerEvent::PhaseChanged(GhostMode::Scatter));
                        }
                        // else: cycle_count >= 2, permanent Chase.
                    }
                }
            }
            GhostMode::Frightened => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run_ticks(t: &mut GameTimers, n: u32) -> Vec<TimerEvent> {
        (0..n).flat_map(|_| t.tick()).collect()
    }

    fn has<F: Fn(&TimerEvent) -> bool>(events: &[TimerEvent], f: F) -> bool {
        events.iter().any(f)
    }

    #[test]
    fn new_starts_in_scatter() {
        let t = GameTimers::new();
        assert_eq!(t.current_phase, GhostMode::Scatter);
        assert_eq!(t.scatter_ticks, 70);
        assert_eq!(t.cycle_count, 0);
        assert_eq!(t.frightened_ticks, 0);
    }

    #[test]
    fn tick_70_emits_phase_changed_chase() {
        let mut t = GameTimers::new();
        let ev = run_ticks(&mut t, 70);
        assert!(has(&ev, |e| matches!(e, TimerEvent::PhaseChanged(GhostMode::Chase))));
        assert_eq!(t.current_phase, GhostMode::Chase);
    }

    #[test]
    fn tick_270_emits_phase_changed_scatter_cycle_count_1() {
        let mut t = GameTimers::new();
        let ev = run_ticks(&mut t, 270);
        assert!(has(&ev, |e| matches!(e, TimerEvent::PhaseChanged(GhostMode::Scatter))));
        assert_eq!(t.cycle_count, 1);
        assert_eq!(t.current_phase, GhostMode::Scatter);
    }

    #[test]
    fn after_2_full_cycles_no_scatter_phase() {
        let mut t = GameTimers::new();
        // 2 full cycles: Scatter70 + Chase200 + Scatter70 + Chase200 = 540 ticks.
        let ev = run_ticks(&mut t, 540);
        let scatter_count = ev
            .iter()
            .filter(|e| matches!(e, TimerEvent::PhaseChanged(GhostMode::Scatter)))
            .count();
        // PhaseChanged(Scatter) fires once (at tick 270); NOT again at tick 540.
        assert_eq!(scatter_count, 1);
        assert_eq!(t.cycle_count, 2);
    }

    #[test]
    fn start_frightened_100_ticks_emits_expired() {
        let mut t = GameTimers::new();
        t.start_frightened();
        let ev = run_ticks(&mut t, 100);
        assert!(has(&ev, |e| matches!(e, TimerEvent::FrightenedExpired)));
    }

    #[test]
    fn start_frightened_emits_blinking_at_tick_70() {
        let mut t = GameTimers::new();
        t.start_frightened();
        // FrightenedBlinking fires when ticks drop to 30, i.e. after 70 decrements.
        let ev = run_ticks(&mut t, 70);
        assert!(has(&ev, |e| matches!(e, TimerEvent::FrightenedBlinking)));
    }

    #[test]
    fn death_pause_10_ticks_emits_expired() {
        let mut t = GameTimers::new();
        t.start_death_pause();
        let ev = run_ticks(&mut t, 10);
        assert!(has(&ev, |e| matches!(e, TimerEvent::DeathPauseExpired)));
    }

    #[test]
    fn extra_life_display_20_ticks_emits_expired() {
        let mut t = GameTimers::new();
        t.start_extra_life_display();
        let ev = run_ticks(&mut t, 20);
        assert!(has(&ev, |e| matches!(e, TimerEvent::ExtraLifeDisplayExpired)));
    }
}
