use super::{regime, Regime, Scheduler, Speed, System};

const WALL_FRAME_S: f64 = 1.0 / 60.0;

fn next_up(x: f64) -> f64 {
    f64::from_bits(x.to_bits() + 1)
}

#[test]
fn regime_thresholds() {
    let month = 1.0 / 12.0;
    for system in [System::Climate, System::Matter, System::Life] {
        assert_eq!(regime(system, month), Regime::Fine, "{system:?} à 1 mois");
        assert_eq!(
            regime(system, next_up(month)),
            Regime::Annual,
            "{system:?} au-delà de 1 mois"
        );
        assert_eq!(regime(system, 1.0), Regime::Annual, "{system:?} à 1 an");
        assert_eq!(
            regime(system, next_up(1.0)),
            Regime::Secular,
            "{system:?} au-delà de 1 an"
        );
        assert_eq!(regime(system, 1000.0), Regime::Secular, "{system:?} à 1 000 ans");
        assert_eq!(
            regime(system, next_up(1000.0)),
            Regime::Geological,
            "{system:?} au-delà de 1 000 ans"
        );
    }
    for dt in [1e-6, month, 1.0, 1000.0, 1e9] {
        assert_eq!(
            regime(System::Geology, dt),
            Regime::Frozen,
            "géologie figée à Δt = {dt}"
        );
    }
}

#[test]
fn pause_produces_no_tick() {
    let mut scheduler = Scheduler::new(Speed::Pause);
    scheduler.accrue(10.0);
    assert!(scheduler.next_tick().is_none());
    assert_eq!(scheduler.ticks(), 0);
    assert_eq!(scheduler.sim_years(), 0.0);
    assert_eq!(scheduler.effective_speed(), 0.0);
}

#[test]
fn ticks_per_second_bounded() {
    for (requested, expected) in [(0.5, 2.0), (10.0, 10.0), (50.0, 20.0)] {
        let mut scheduler = Scheduler::with_rate(Speed::Year, requested);
        let mut count = 0_u64;
        for _ in 0..7200 {
            scheduler.accrue(WALL_FRAME_S);
            while scheduler.next_tick().is_some() {
                count += 1;
            }
        }
        let rate = count as f64 / 120.0;
        assert!(
            (rate - expected).abs() < 0.05,
            "cadence {rate} attendue {expected} pour {requested} demandés"
        );
    }
}

#[test]
fn effective_speed_reported() {
    let tick_cost_s = 0.3;
    let step_s = 0.01;
    let mut scheduler = Scheduler::new(Speed::Year);
    let mut wall_s = 0.0;
    for _ in 0..3000 {
        scheduler.accrue(step_s);
        wall_s += step_s;
        while scheduler.next_tick().is_some() {
            scheduler.record_cost(tick_cost_s);
        }
    }
    let demanded = Speed::Year.years_per_second();
    let effective = scheduler.effective_speed();
    assert!(
        effective < demanded,
        "vitesse effective {effective} non rapportée sous {demanded}"
    );
    assert!(
        (effective / demanded - 1.0 / 3.0).abs() < 0.05,
        "ratio attendu 1/3, obtenu {effective}"
    );
    let measured = scheduler.sim_years() / wall_s;
    assert!(
        (measured - effective).abs() < 0.05 * demanded,
        "mesuré {measured} ≠ rapporté {effective}"
    );
}

#[test]
fn sim_time_monotonic() {
    let plan = [
        Speed::Day,
        Speed::Pause,
        Speed::Century,
        Speed::TenMyr,
        Speed::Pause,
        Speed::Year,
    ];
    let mut scheduler = Scheduler::new(Speed::Day);
    let mut last_sim_years = scheduler.sim_years();
    for speed in plan {
        scheduler.set_speed(speed);
        for _ in 0..600 {
            scheduler.accrue(WALL_FRAME_S);
            while let Some(tick) = scheduler.next_tick() {
                assert!(
                    tick.dt_years.is_finite() && tick.dt_years >= 0.0,
                    "Δt invalide : {}",
                    tick.dt_years
                );
            }
            assert!(
                scheduler.sim_years() >= last_sim_years,
                "temps simulé recule à {speed:?}"
            );
            last_sim_years = scheduler.sim_years();
        }
    }
    assert!(scheduler.sim_years() > 0.0);
}

#[test]
fn deterministic_schedule() {
    let run = || {
        let mut scheduler = Scheduler::new(Speed::TenKyr);
        let mut deltas = Vec::new();
        for i in 0..2000_u32 {
            scheduler.accrue(0.013 + f64::from(i % 7) * 0.002);
            while let Some(tick) = scheduler.next_tick() {
                deltas.push(tick.dt_years.to_bits());
            }
            if i % 5 == 0 {
                scheduler.record_cost(0.004 * f64::from(i % 3));
            }
        }
        deltas
    };
    let first = run();
    assert!(!first.is_empty());
    assert_eq!(first, run());
}
