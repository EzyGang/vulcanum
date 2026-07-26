mod requests;
mod tickets;

use uuid::Uuid;

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum FollowupTicketReservation {
    Acquired {
        token: Uuid,
        external_task_ref: Option<String>,
        created_by_delivery_id: Option<String>,
    },
    Pending,
}

pub struct InsertFollowupRequestParams<'a> {
    pub delivery_id: &'a str,
    pub github_installation_id: i64,
    pub repo_full_name: &'a str,
    pub pr_number: i64,
    pub comment_id: i64,
    pub sender_id: &'a str,
    pub project_config_id: Uuid,
    pub ticket_selector: Option<&'a str>,
    pub request_body: &'a str,
}

pub struct FinishFollowupRequestParams<'a> {
    pub delivery_id: &'a str,
    pub external_task_ref: Option<&'a str>,
    pub work_run_id: Option<Uuid>,
    pub ticket_created: bool,
    pub outcome: &'a str,
    pub ambiguous_task_refs: &'a [String],
}
