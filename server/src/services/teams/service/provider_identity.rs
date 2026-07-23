use uuid::Uuid;

use crate::models::teams::errors::TeamsError;
use crate::models::teams::model::ProviderIdentity;
use crate::services::teams::service::TeamsService;

impl TeamsService {
    pub async fn is_provider_identity_member(
        &self,
        team_id: Uuid,
        provider: &str,
        provider_user_id: &str,
    ) -> Result<bool, TeamsError> {
        self.repo
            .is_provider_identity_member(&self.db, team_id, provider, provider_user_id)
            .await
    }

    #[must_use = "identity list results should be handled"]
    pub async fn list_identities_for_user(
        &self,
        user_id: &str,
    ) -> Result<Vec<ProviderIdentity>, TeamsError> {
        self.repo.list_identities_for_user(&self.db, user_id).await
    }
}
