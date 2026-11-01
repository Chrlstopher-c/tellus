use super::encode;
use super::slot::WorldSlot;
use crate::world::{World, WorldError, WorldEvent, WorldRequest};

fn small_world() -> World {
    let request = WorldRequest::from_json(
        r#"{"source":{"kind":"generated","seed":42,"waterPercent":70.8,"sizePercent":100,"distancePercent":100},"level":3}"#,
    )
    .expect("requête valide");
    World::create(&request).expect("monde créé")
}

#[test]
fn slot_without_world_reports_no_world() {
    let slot = WorldSlot::new();
    assert_eq!(slot.with(|w| Ok(w.info().cells)), Err(WorldError::NoWorld));
    assert_eq!(slot.with_mut(|w| w.step(1.0)), Err(WorldError::NoWorld));
}

#[test]
fn slot_keeps_the_replaced_world() {
    let slot = WorldSlot::new();
    slot.replace(small_world()).expect("verrou sain");
    assert_eq!(slot.with(|w| Ok(w.info().cells)), Ok(642));
    assert_eq!(slot.with_mut(|w| w.step(2.5)), Ok(2.5));
}

#[test]
fn world_info_uses_interface_keys() {
    let value = encode::world_info(small_world().info());
    assert_eq!(value["cells"], 642);
    assert_eq!(value["seed"], 42);
    assert_eq!(value["name"], "Selmimi");
    assert_eq!(value["lookupWidth"], 2048);
    assert_eq!(value["lookupHeight"], 1024);
}

#[test]
fn events_encode_missing_cell_as_null() {
    let event = WorldEvent {
        tick: 0,
        years: 0.0,
        kind: "geology",
        title: "Formation du monde".to_owned(),
        body: "corps".to_owned(),
        cell: None,
    };
    let value = encode::events(&[event]);
    assert!(value[0]["cell"].is_null());
    assert_eq!(value[0]["kind"], "geology");
}

#[test]
fn vitals_and_region_keep_interface_fields() {
    let world = small_world();
    let vitals = encode::vitals(&world.vitals());
    assert_eq!(vitals["species"], 0);
    assert!(vitals["era"].as_str().is_some_and(|era| era.contains("sans vie")));
    let region = encode::cell(&world.inspect(0).expect("cellule 0"));
    assert!(region["soil"]["water"].is_number());
    assert!(region["biome"].is_null() || region["biome"].is_string());
}
