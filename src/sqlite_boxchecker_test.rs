use super::*;

#[test]
fn creates_instance() {
    let _boxer = SqliteBoxes::new(":memory:");
}

#[test]
fn logs_activity() {
    let mut boxer = SqliteBoxes::new(":memory:");
    let action = boxer.create_action("unit testing");
    let activity = boxer.log_activity(action);
    assert!(activity > 0);
}

#[test]
fn logs_activities() {
    let mut boxer = SqliteBoxes::new(":memory:");
    let actions = vec![
        boxer.create_action("unit testing"),
        boxer.create_action("linting"),
    ];
    let activity = boxer.log_activities(&actions);
    assert!(activity > 0);
}

#[test]
fn annotates_activities() {
    let mut boxer = SqliteBoxes::new(":memory:");
    let action = boxer.create_action("unit testing");
    let activity = boxer.log_activity(action);
    let note = boxer.annotate_activity(activity, "this one passes");
    assert!(note > 0);
}

#[test]
fn retrieves_action_name() {
    let mut boxer = SqliteBoxes::new(":memory:");
    let action = boxer.create_action("unit testing");
    let name = boxer.get_action_name(action);
    assert_eq!(name, "unit testing");
}

#[test]
fn retrieves_notations() {
    let mut boxer = SqliteBoxes::new(":memory:");
    let action = boxer.create_action("unit testing");
    let activity = boxer.log_activity(action);
    let note = boxer.annotate_activity(activity, "this one passes");

    let mut notations = vec![0; 2];
    let mut notes = vec![(0, "".to_string()); 2];
    assert_eq!(boxer.get_notations(activity, 0, &mut notations), 1);
    assert_eq!(notations[0], note);
    assert_eq!(boxer.get_note(note), "this one passes");
    assert_eq!(boxer.get_note_bulk(&vec![note], &mut notes), 1);
    assert_eq!(notes[0], (1, "this one passes".to_string()));
}
