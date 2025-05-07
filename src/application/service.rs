use crate::core::{
    token::{Honeytoken, TokenRepository},
    notification::{NotificationService, NotificationConfig},
    injection::{FileInjector, InjectionConfig},
};
use anyhow::Result;
use log::{info, error};

pub struct RedTokenService {
    token_repo: Box<dyn TokenRepository>,
    file_injector: Box<dyn FileInjector>,
    notification_service: Box<dyn NotificationService>,
}

impl RedTokenService {
    pub fn new(
        token_repo: Box<dyn TokenRepository>,
        file_injector: Box<dyn FileInjector>,
        notification_service: Box<dyn NotificationService>,
    ) -> Self {
        Self {
            token_repo,
            file_injector,
            notification_service,
        }
    }

    pub async fn inject_token(&self, file_path: &str, value: String) -> Result<Honeytoken> {
        let token = Honeytoken::new(value, file_path.to_string());
        
        // Inject the token into the file
        self.file_injector.inject_token(file_path, &token).await?;
        
        // Save the token to repository
        self.token_repo.save(&token).await?;
        
        info!("Successfully injected token into {}", file_path);
        Ok(token)
    }

    pub async fn check_token(&self, token_value: &str) -> Result<()> {
        if let Some(mut token) = self.token_repo.find_by_value(token_value).await? {
            if !token.is_triggered {
                token.mark_as_triggered();
                self.token_repo.update(&token).await?;
                
                // Send notification
                if let Err(e) = self.notification_service.send_alert(&token).await {
                    error!("Failed to send notification: {}", e);
                }
                
                info!("Token {} has been triggered!", token.id);
            }
        }
        Ok(())
    }

    pub async fn list_tokens(&self) -> Result<Vec<Honeytoken>> {
        self.token_repo.find_all().await
    }

    pub async fn remove_token(&self, token_id: uuid::Uuid) -> Result<()> {
        if let Some(token) = self.token_repo.find_by_id(token_id).await? {
            self.file_injector.remove_token(&token.file_path, &token).await?;
            info!("Successfully removed token {}", token_id);
        }
        Ok(())
    }
} 