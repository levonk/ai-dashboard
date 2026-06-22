use super::{NotificationSender, NotificationError};
use crate::alerts::{AlertEvent, NotificationChannel, ChannelConfig};

pub struct EmailSender;

impl EmailSender {
    pub fn new() -> Self {
        Self
    }

    fn render_template(&self, template: &str, event: &AlertEvent) -> String {
        template
            .replace("{{rule_name}}", &event.rule_name)
            .replace("{{severity}}", &format!("{:?}", event.severity))
            .replace("{{message}}", &event.message)
            .replace("{{metric_value}}", &event.metric_value.to_string())
            .replace("{{threshold_value}}", &event.threshold_value.to_string())
            .replace("{{triggered_at}}", &event.triggered_at.to_rfc3339())
    }
}

impl NotificationSender for EmailSender {
    fn send(&self, event: &AlertEvent, channel: &NotificationChannel) -> Result<(), NotificationError> {
        if let ChannelConfig::Email { recipients, subject_template, body_template } = &channel.config {
            let subject = self.render_template(subject_template, event);
            let body = self.render_template(body_template, event);
            
            // TODO: Implement actual email sending
            // For now, this is a placeholder that logs the email content
            println!("EMAIL: To: {:?}, Subject: {}, Body: {}", recipients, subject, body);
            
            Ok(())
        } else {
            Err(NotificationError::ConfigurationError(
                "Invalid channel config for Email".to_string()
            ))
        }
    }
}

impl Default for EmailSender {
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
    fn test_render_template() {
        let sender = EmailSender::new();
        
        let event = AlertEvent::new(
            "rule-1".to_string(),
            "Test Rule".to_string(),
            AlertSeverity::Warning,
            150.0,
            100.0,
            "Test alert".to_string(),
            HashMap::new(),
        );

        let template = "Alert: {{rule_name}} - {{message}}";
        let rendered = sender.render_template(template, &event);
        
        assert!(rendered.contains("Test Rule"));
        assert!(rendered.contains("Test alert"));
    }

    #[test]
    fn test_send_email() {
        let sender = EmailSender::new();
        
        let channel = NotificationChannel::new(
            "test-channel".to_string(),
            ChannelType::Email,
            ChannelConfig::Email {
                recipients: vec!["test@example.com".to_string()],
                subject_template: "Alert: {{rule_name}}".to_string(),
                body_template: "{{message}}".to_string(),
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