use super::{NotificationSender, NotificationError};
use crate::alerts::{AlertEvent, NotificationChannel, ChannelConfig};

pub struct SlackSender;

impl SlackSender {
    pub fn new() -> Self {
        Self
    }

    fn format_slack_message(&self, event: &AlertEvent, channel: &str, username: &str, icon_emoji: &str) -> String {
        let color = match event.severity {
            crate::alerts::AlertSeverity::Info => "#36a64f",
            crate::alerts::AlertSeverity::Warning => "#ff9900",
            crate::alerts::AlertSeverity::Error => "#ff0000",
            crate::alerts::AlertSeverity::Critical => "#ff0000",
        };

        format!(
            r#"{{"channel": "{}", "username": "{}", "icon_emoji": "{}", "attachments": [{{"color": "{}", "title": "{}", "text": "{}", "fields": [{{"title": "Metric Value", "value": "{}", "short": true}}, {{"title": "Threshold", "value": "{}", "short": true}}, {{"title": "Triggered At", "value": "{}", "short": false}}]}}]}}"#,
            channel,
            username,
            icon_emoji,
            color,
            event.rule_name,
            event.message,
            event.metric_value,
            event.threshold_value,
            event.triggered_at.to_rfc3339()
        )
    }
}

impl NotificationSender for SlackSender {
    fn send(&self, event: &AlertEvent, channel: &NotificationChannel) -> Result<(), NotificationError> {
        if let ChannelConfig::Slack { webhook_url, channel, username, icon_emoji } = &channel.config {
            let message = self.format_slack_message(event, channel, username, icon_emoji);
            
            // TODO: Implement actual Slack webhook call
            // For now, this is a placeholder that logs the webhook call
            println!("SLACK WEBHOOK: URL: {}, Message: {}", webhook_url, message);
            
            Ok(())
        } else {
            Err(NotificationError::ConfigurationError(
                "Invalid channel config for Slack".to_string()
            ))
        }
    }
}

impl Default for SlackSender {
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
    fn test_format_slack_message() {
        let sender = SlackSender::new();
        
        let event = AlertEvent::new(
            "rule-1".to_string(),
            "Test Rule".to_string(),
            AlertSeverity::Warning,
            150.0,
            100.0,
            "Test alert".to_string(),
            HashMap::new(),
        );

        let message = sender.format_slack_message(&event, "#alerts", "AlertBot", ":warning:");
        
        assert!(message.contains("#alerts"));
        assert!(message.contains("AlertBot"));
        assert!(message.contains("Test Rule"));
        assert!(message.contains("150"));
    }

    #[test]
    fn test_send_slack() {
        let sender = SlackSender::new();
        
        let channel = NotificationChannel::new(
            "test-channel".to_string(),
            ChannelType::Slack,
            ChannelConfig::Slack {
                webhook_url: "https://hooks.slack.com/services/xxx".to_string(),
                channel: "#alerts".to_string(),
                username: "AlertBot".to_string(),
                icon_emoji: ":warning:".to_string(),
            },
        );

        let event = AlertEvent::new(
            "rule-1".to_string(),
            "Test Rule".to_string(),
            AlertSeverity::Warning,
            150.0,
            100.0,
            "Test alert".to_string(),
            HashMap::new(),
        );

        let result = sender.send(&event, &channel);
        assert!(result.is_ok());
    }
}