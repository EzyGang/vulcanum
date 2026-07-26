use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Mutex;

use async_trait::async_trait;

use crate::models::project_configs::model::ProjectConfig;
use crate::models::provider_configs::model::IntegrationProvider;
use crate::models::providers::errors::IntegrationError;
use crate::models::providers::model::{CreateIntegrationTaskInput, IntegrationTask};
use crate::models::work_runs::errors::WorkRunsError;
use crate::services::work_runs::service::implementation_followup_ticket::ImplementationFollowupTicketClient;

#[derive(Default)]
pub(crate) struct MockFollowupTicketClient {
    tasks: Mutex<HashMap<String, IntegrationTask>>,
    create_count: AtomicUsize,
    update_count: AtomicUsize,
    fail_after_create: AtomicBool,
}

impl MockFollowupTicketClient {
    pub fn with_task(task: IntegrationTask) -> Self {
        Self::with_tasks(vec![task])
    }

    pub fn with_tasks(tasks: Vec<IntegrationTask>) -> Self {
        Self {
            tasks: Mutex::new(
                tasks
                    .into_iter()
                    .map(|task| (task.id.clone(), task))
                    .collect(),
            ),
            ..Self::default()
        }
    }

    pub fn fail_once_after_create(&self) {
        self.fail_after_create.store(true, Ordering::SeqCst);
    }

    pub fn task(&self, task_ref: &str) -> IntegrationTask {
        self.tasks
            .lock()
            .expect("task lock")
            .get(task_ref)
            .expect("stored task")
            .clone()
    }

    pub fn create_count(&self) -> usize {
        self.create_count.load(Ordering::SeqCst)
    }

    pub fn update_count(&self) -> usize {
        self.update_count.load(Ordering::SeqCst)
    }
}

#[async_trait]
impl ImplementationFollowupTicketClient for MockFollowupTicketClient {
    async fn find_existing(
        &self,
        _provider: &IntegrationProvider,
        _project: &ProjectConfig,
        marker: &str,
    ) -> Result<Option<IntegrationTask>, WorkRunsError> {
        Ok(self
            .tasks
            .lock()
            .expect("task lock")
            .values()
            .find(|task| {
                task.description
                    .as_deref()
                    .is_some_and(|description| description.contains(marker))
            })
            .cloned())
    }

    async fn fetch(
        &self,
        _provider: &IntegrationProvider,
        external_task_ref: &str,
    ) -> Result<IntegrationTask, WorkRunsError> {
        self.tasks
            .lock()
            .expect("task lock")
            .get(external_task_ref)
            .cloned()
            .ok_or(WorkRunsError::Provider(IntegrationError::Other(
                "missing mock task".to_owned(),
            )))
    }

    async fn create(
        &self,
        _provider: &IntegrationProvider,
        input: CreateIntegrationTaskInput,
    ) -> Result<IntegrationTask, WorkRunsError> {
        self.create_count.fetch_add(1, Ordering::SeqCst);
        let task = integration_task(
            "created-followup-ticket",
            &input.title,
            Some(input.body),
            &input.status,
        );
        self.tasks
            .lock()
            .expect("task lock")
            .insert(task.id.clone(), task.clone());
        if self.fail_after_create.swap(false, Ordering::SeqCst) {
            return Err(WorkRunsError::Provider(IntegrationError::Other(
                "simulated lost provider response".to_owned(),
            )));
        }
        Ok(task)
    }

    async fn update_description(
        &self,
        _provider: &IntegrationProvider,
        external_task_ref: &str,
        description: &str,
    ) -> Result<(), WorkRunsError> {
        self.update_count.fetch_add(1, Ordering::SeqCst);
        let mut tasks = self.tasks.lock().expect("task lock");
        let task = tasks.get_mut(external_task_ref).ok_or_else(|| {
            WorkRunsError::Provider(IntegrationError::Other("missing mock task".to_owned()))
        })?;
        task.description = Some(description.to_owned());
        Ok(())
    }
}

pub(super) fn integration_task(
    id: &str,
    title: &str,
    description: Option<String>,
    status: &str,
) -> IntegrationTask {
    IntegrationTask {
        id: id.to_owned(),
        title: title.to_owned(),
        project_id: "project-1".to_owned(),
        description,
        status: status.to_owned(),
        priority: "medium".to_owned(),
        number: None,
        project_slug: None,
        assignee_name: None,
        created_at: "2026-01-01T00:00:00Z".to_owned(),
        updated_at: None,
        labels: Vec::new(),
    }
}
