use super::{NotificationSender, NotificationError};
use crate::alerts::{AlertEvent, NotificationChannel, ChannelConfig};

pub struct WebhookSender;

impl WebhookSender {
    pub fn new() -> Self {
        Self
    }

    fn render_template(&self, template: &str, event: &AlertEvent) -> String {
        template
            .replace("{{rule_id}}", &event.id)
            .replace("{{rule_name}}", &event.rule_name)
            .replace("{{severity}}", &format!("{:?}", event.severity))
            .replace("{{message}}", &event.message)
            .replace("{{metric_value}}", &event.metric_value.to_string())
            .replace("{{threshold_value}}", &event.threshold_value.to_string())
            .replace("{{triggered_at}}", &event.triggered_at.to_rfc3339())
            .replace("{{resolved}}", &event.resolved.to_string())
    }

    fn build_webhook_payload(&self, event: &AlertEvent, body_template: &str) -> String {
        self.render_template(body_template, event)
    }
}

impl NotificationSender for WebhookSender {
    fn send(&self, event: &AlertEvent, channel: &NotificationChannel) -> Result<(), NotificationError> {
        if let ChannelConfig::Webhook { url, method, headers, body_template } = &channel.config {
            let payload = self.build_webhook_payload(event, body_template);
            
            // TODO: Implement actual HTTP request
            // For now, this is a placeholder that logs the webhook call
            println!("WEBHOOK: URL: {}, Method: {}, Headers: {:?}, Body: {}", url, method, headers, payload);
            
            Ok(())
        } else {
            Err(NotificationError::ConfigurationError(
                "Invalid channel config for Webhook".to_string()
            ))
        }
    }
}

impl Default for WebhookSender {
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
        let sender = WebhookSender::new();
        
        let event = AlertEvent::new(
            "rule-1".to_string(),
            "Test Rule".to_string(),
            AlertSeverity::Warning,
            150.0,
            100.0,
            "Test alert".to_string(),
            HashMap::new(),
        );

        let template = r#"{{"rule": "{{rule_name}}", "severity": "{{severity}}"}}"#;
        let rendered = sender.render_template(template, &event);
        
        assert!(rendered.contains("Test Rule"));
        assert!(rendered.contains("Warning"));
    }

    #[test]
    fn test_send_webhook() {
        let sender = WebhookSender::new();
        
        let mut headers = HashMap::new();
        headers.insert("Content-Type".to_string(), "application/json".to_string());

        let channel = NotificationChannel::new(
            "test-channel".to_string(),
            ChannelType::Webhook,
            ChannelConfig::Webhook {
                url: "https://example.com/webhook".to_string(),
                method: "POST".to_string(),
                headers,
                body_template: r#"{{"rule": "{{rule_name}}", "message": "{{message}}"}}"#.to_string(),
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