use super::world::{WorldError, WorldSlot, MAX_LEVEL};

const SEED: u32 = 42;
const L6_CELLS: usize = 40_962;

#[test]
fn create_level_6_reports_cell_count() {
    assert_eq!(WorldSlot::new().create(SEED, 6), Ok(L6_CELLS));
}

#[test]
fn create_rejects_level_above_max() {
    let level = MAX_LEVEL + 1;
    assert_eq!(
        WorldSlot::new().create(SEED, level),
        Err(WorldError::InvalidLevel(level))
    );
}

#[test]
fn step_and_snapshot_without_world_fail() {
    let slot = WorldSlot::new();
    assert_eq!(slot.step(1.0), Err(WorldError::NoWorld));
    assert_eq!(slot.snapshot(), Err(WorldError::NoWorld));
}

#[test]
fn step_accumulates_simulated_time() {
    let slot = WorldSlot::new();
    slot.create(SEED, 3).expect("L3 valide");
    assert_eq!(slot.step(1.5), Ok(1.5));
    assert_eq!(slot.step(0.25), Ok(1.75));
}

#[test]
fn step_rejects_negative_and_non_finite_durations() {
    let slot = WorldSlot::new();
    slot.create(SEED, 3).expect("L3 valide");
    for dt in [-1.0, f64::NAN, f64::INFINITY] {
        assert!(
            matches!(slot.step(dt), Err(WorldError::InvalidDuration(_))),
            "dt = {dt}"
        );
    }
}

#[test]
fn snapshot_holds_one_zero_per_cell() {
    let slot = WorldSlot::new();
    let cells = slot.create(SEED, 3).expect("L3 valide");
    let snapshot = slot.snapshot().expect("monde créé");
    assert_eq!(snapshot.len(), cells);
    assert!(snapshot.iter().all(|value| *value == 0.0));
}

#[test]
fn create_again_resets_simulated_time() {
    let slot = WorldSlot::new();
    slot.create(SEED, 3).expect("L3 valide");
    slot.step(5.0).expect("pas valide");
    slot.create(SEED, 3).expect("L3 valide");
    assert_eq!(slot.step(0.0), Ok(0.0));
}
