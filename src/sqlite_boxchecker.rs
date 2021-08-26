use crate::boxchecker::{ActionId, ActivityId, AnnotationId, BoxChecker, BoxMaker, BoxSearcher};
use normal::{IdPairs, Normal};
use std::time::{SystemTime, UNIX_EPOCH};

const ACTION_HIERARCHY_TAB: &str = "actionHierarchy";
const ACTION_TAB: &str = "actions";
const ACTIVITY_TAB: &str = "activities";
const NOTATIONS_TAB: &str = "notations";
const NOTE_TAB: &str = "notes";

const ACTION_COL: &str = "actionName";
const CHILD_COL: &str = "child";
const NOTE_COL: &str = "note";
const PARENT_COL: &str = "parent";
const TIME_COL: &str = "time";

pub struct SqliteBoxes<'a> {
    action_hierarchy: IdPairs<'a>,
    actions: Normal<'a>,
    activities: IdPairs<'a>,
    notations: IdPairs<'a>,
    notes: Normal<'a>,
}

impl<'a> SqliteBoxes<'a> {
    pub fn new(path: &str) -> Self {
        SqliteBoxes {
            action_hierarchy: IdPairs::new(path, ACTION_HIERARCHY_TAB, PARENT_COL, CHILD_COL)
                .unwrap(),
            actions: Normal::new(path, ACTION_TAB, ACTION_COL).unwrap(),
            activities: IdPairs::new(path, ACTIVITY_TAB, TIME_COL, ACTION_COL).unwrap(),
            notations: IdPairs::new(path, NOTATIONS_TAB, TIME_COL, NOTE_COL).unwrap(),
            notes: Normal::new(path, NOTE_TAB, NOTE_COL).unwrap(),
        }
    }
}

impl<'a> BoxMaker for SqliteBoxes<'a> {
    fn create_action(&mut self, action_name: &str) -> ActionId {
        match self.actions.create(action_name) {
            Ok(id) => id,
            Err(e) => {
                log::error!("create_action: {}", e.msg);
                0
            }
        }
    }

    fn make_action_parent_of(&mut self, parent: ActionId, child: ActionId) {
        match self.action_hierarchy.insert(parent, child) {
            Ok(_) => (),
            Err(e) => {
                log::error!("make_action_parent_of: {}", e.msg)
            }
        }
    }
}

fn get_time() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap() // XXX understand this failure
        .as_millis() as i64
}

impl<'a> BoxChecker for SqliteBoxes<'a> {
    fn annotate_activity(&mut self, activity: ActivityId, text: &str) -> AnnotationId {
        match self.notes.create(text) {
            Ok(note_id) => {
                match self.notations.insert(activity, note_id) {
                    Ok(_) => note_id,
                    Err(e) => {
                        log::error!("cannot notate activity {}: {}", activity, e.msg);
                        0 // recover?
                    }
                }
            }
            Err(e) => {
                log::error!("cannot create note for activity {}: {}", activity, e.msg);
                0
            }
        }
    }

    fn log_activities(&mut self, actions: &Vec<ActionId>) -> ActivityId {
        let time_millis = get_time();
        for a in actions {
            self.log_activity_at_time(*a, time_millis);
        }
        time_millis
    }

    fn log_activity(&mut self, action: ActionId) -> ActivityId {
        let time_millis = get_time();
        self.log_activity_at_time(action, time_millis)
    }

    fn log_activity_at_time(&mut self, action: ActionId, time_millis: i64) -> ActivityId {
        match self.activities.insert(time_millis, action) {
            Ok(_) => time_millis,
            Err(e) => {
                log::error!("log_activity_at_time: {}", e.msg);
                0
            }
        }
    }
}

impl<'a> BoxSearcher<'a> for SqliteBoxes<'a> {
    /// Wrap call to action lookup name, logging an error and returning ""
    /// if necessary.
    fn get_action_name(&self, action: ActionId) -> String {
        match self.actions.get(action) {
            Ok(name) => name,
            Err(e) => {
                log::error!("get_notations: {}", e.msg);
                "".to_string()
            }
        }
    }

    /// Wrap call to lookup associated notations, logging an error and
    /// returning 0 when necessary.
    fn get_notations(
        &self,
        activity: ActivityId,
        last_idx: AnnotationId,
        dest: &mut Vec<AnnotationId>,
    ) -> usize {
        match self.notations.get_page(activity, last_idx, dest) {
            Ok(count) => count,
            Err(e) => {
                log::error!("get_notations: {}", e.msg);
                0
            }
        }
    }

    fn get_note(&self, annotation: AnnotationId) -> String {
        match self.notes.get(annotation) {
            Ok(note) => note,
            Err(e) => {
                log::error!("get_note: {}", e.msg);
                "".to_string()
            }
        }
    }

    fn get_note_bulk(
        &self,
        ids: &Vec<AnnotationId>,
        dest: &mut Vec<(AnnotationId, String)>,
    ) -> usize {
        match self.notes.get_bulk(ids, dest) {
            Ok(count) => count,
            Err(e) => {
                log::error!("get_note_bulk: {}", e.msg);
                0
            }
        }
    }

    fn search_action_names(
        &self,
        substr: &str,
        last_id: ActionId,
        dest: &mut Vec<(ActionId, String)>,
    ) -> usize {
        match self.actions.search_page(substr, last_id, dest) {
            Ok(count) => count,
            Err(e) => {
                log::error!("search_action_names: {}", e.msg);
                0
            }
        }
    }

    fn search_activity_by_time(
        &self,
        from: usize,
        to: usize,
        dest: &mut Vec<(ActivityId, ActionId)>,
    ) -> usize {
        match self.activities.page_left(from as i64, to as i64, dest) {
            Ok(count) => count,
            Err(e) => {
                log::error!("search_activity_by_time: {}", e.msg);
                0
            }
        }
    }
}

#[cfg(test)]
#[path = "./sqlite_boxchecker_test.rs"]
mod sqlite_boxchecker_test;
