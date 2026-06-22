pub mod email;
pub mod slack;
pub mod webhook;
pub mod sms;

use super::{NotificationChannel, ChannelType, AlertEvent};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum NotificationError {
    #[error("Channel not configured: {0}")]
    ChannelNotConfigured(String),
    
    #[error("Send failed: {0}")]
    SendFailed(String),
    
    #[error("Template error: {0}")]
    TemplateError(String),
    
    #[error("Configuration error: {0}")]
    ConfigurationError(String),
    
    #[error("Rate limit exceeded")]
    RateLimitExceeded,
}

pub trait NotificationSender: Send + Sync {
    fn send(&self, event: &AlertEvent, channel: &NotificationChannel) -> Result<(), NotificationError>;
}

pub struct NotificationManager {
    email_sender: Box<dyn NotificationSender>,
    slack_sender: Box<dyn NotificationSender>,
    webhook_sender: Box<dyn NotificationSender>,
    sms_sender: Box<dyn NotificationSender>,
}

impl NotificationManager {
    pub fn new() -> Self {
        Self {
            email_sender: Box::new(email::EmailSender::new()),
            slack_sender: Box::new(slack::SlackSender::new()),
            webhook_sender: Box::new(webhook::WebhookSender::new()),
            sms_sender: Box::new(sms::SmsSender::new()),
        }
    }

    pub fn send_notification(
        &self,
        event: &AlertEvent,
        channel: &NotificationChannel,
    ) -> Result<(), NotificationError> {
        if !channel.enabled {
            return Err(NotificationError::ChannelNotConfigured(
                "Channel is disabled".to_string()
            ));
        }

        match channel.channel_type {
            ChannelType::Email => self.email_sender.send(event, channel),
            ChannelType::Slack => self.slack_sender.send(event, channel),
            ChannelType::Webhook => self.webhook_sender.send(event, channel),
            ChannelType::Sms => self.sms_sender.send(event, channel),
        }
    }

    pub fn send_to_channels(
        &self,
        event: &AlertEvent,
        channels: &[NotificationChannel],
    ) -> Vec<Result<(), NotificationError>> {
        channels
            .iter()
            .map(|channel| self.send_notification(event, channel))
            .collect()
    }
}

impl Default for NotificationManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::alerts::{AlertSeverity, NotificationChannel, ChannelType, ChannelConfig};
    use std::collections::HashMap;

    #[test]
    fn test_notification_manager_disabled_channel() {
        let manager = NotificationManager::new();
        
        let channel = NotificationChannel::new(
            "test-channel".to_string(),
            ChannelType::Email,
            ChannelConfig::Email {
                recipients: vec!["test@example.com".to_string()],
                subject_template: "Alert".to_string(),
                body_template: "Test".to_string(),
            },
        );
        
        let disabled_channel = NotificationChannel {
            enabled: false,
            ..channel
        };

        let event = AlertEvent::new(
            "rule-1".to_string(),
            "Test Rule".to_string(),
            AlertSeverity::Warning,
            150.0,
            100.0,
            "Test alert".to_string(),
            HashMap::new(),
        );

        let result = manager.send_notification(&event, &disabled_channel);
        assert!(result.is_err());
    }
}