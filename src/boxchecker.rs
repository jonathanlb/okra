// Break up into
// - activity creation
// - activity log
// - activity search

pub type ActionId = i64;
pub type ActivityId = i64;
pub type AnnotationId = i64;

pub struct BoxCheckerError {
    pub msg: String,
}

pub trait BoxMaker {
    fn create_action(&mut self, action_name: &str) -> ActionId;
    fn make_action_parent_of(&mut self, parent: ActionId, child: ActionId);
}

pub trait BoxChecker {
    fn annotate_activity(&mut self, activity: ActivityId, text: &str) -> AnnotationId;
    fn log_activities(&mut self, actions: &Vec<ActionId>) -> ActivityId;
    fn log_activity(&mut self, action: ActionId) -> ActivityId;
    fn log_activity_at_time(&mut self, action: ActionId, epoch_millis: i64) -> ActivityId;
}

pub trait BoxSearcher<'a> {
    fn get_action_name(&self, action: ActionId) -> String;

    fn get_notations(
        &self,
        activity: ActivityId,
        last_idx: AnnotationId,
        dest: &mut Vec<AnnotationId>,
    ) -> usize;
    fn get_note(&self, annotation: AnnotationId) -> String;
    fn get_note_bulk(
        &self,
        ids: &Vec<AnnotationId>,
        dest: &mut Vec<(AnnotationId, String)>,
    ) -> usize;

    fn search_action_names(
        &self,
        substring: &str,
        last_idx: ActionId,
        dest: &mut Vec<(ActionId, String)>,
    ) -> usize;

    fn search_activity_by_time(
        &self,
        from: usize,
        to: usize,
        dest: &mut Vec<(ActivityId, ActionId)>,
    ) -> usize;

    // activity search criteria
    // - min/max time
    // - action ids
    // - parent action ids
    // - annotation id
}
