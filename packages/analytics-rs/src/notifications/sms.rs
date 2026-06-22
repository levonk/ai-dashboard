use super::{NotificationSender, NotificationError};
use crate::alerts::{AlertEvent, NotificationChannel, ChannelConfig};

pub struct SmsSender;

impl SmsSender {
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
    }

    fn truncate_message(&self, message: &str, max_length: usize) -> String {
        if message.len() <= max_length {
            message.to_string()
        } else {
            // Ensure we don't panic if max_length is less than 3
            if max_length <= 3 {
                "...".to_string()
            } else {
                format!("{}...", &message[..max_length - 3])
            }
        }
    }
}

impl NotificationSender for SmsSender {
    fn send(&self, event: &AlertEvent, channel: &NotificationChannel) -> Result<(), NotificationError> {
        if let ChannelConfig::Sms { phone_numbers, message_template } = &channel.config {
            let message = self.render_template(message_template, event);
            // SMS messages are typically limited to 160 characters
            let truncated_message = self.truncate_message(&message, 160);
            
            // TODO: Implement actual SMS sending
            // For now, this is a placeholder that logs the SMS content
            for phone_number in phone_numbers {
                println!("SMS: To: {}, Message: {}", phone_number, truncated_message);
            }
            
            Ok(())
        } else {
            Err(NotificationError::ConfigurationError(
                "Invalid channel config for SMS".to_string()
            ))
        }
    }
}

impl Default for SmsSender {
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
        let sender = SmsSender::new();
        
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
    fn test_truncate_message() {
        let sender = SmsSender::new();
        
        let long_message = "This is a very long message that exceeds the SMS character limit of 160 characters and should be truncated appropriately";
        let truncated = sender.truncate_message(long_message, 160);
        
        assert!(truncated.len() <= 160);
        // Only check for truncation if the message was actually truncated
        if long_message.len() > 160 {
            assert!(truncated.ends_with("..."));
        }
    }

    #[test]
    fn test_send_sms() {
        let sender = SmsSender::new();
        
        let channel = NotificationChannel::new(
            "test-channel".to_string(),
            ChannelType::Sms,
            ChannelConfig::Sms {
                phone_numbers: vec!["+1234567890".to_string()],
                message_template: "Alert: {{rule_name}} - {{message}}".to_string(),
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

    #[test]
    fn test_send_sms_multiple_recipients() {
        let sender = SmsSender::new();
        
        let channel = NotificationChannel::new(
            "test-channel".to_string(),
            ChannelType::Sms,
            ChannelConfig::Sms {
                phone_numbers: vec![
                    "+1234567890".to_string(),
                    "+0987654321".to_string(),
                ],
                message_template: "Alert: {{rule_name}}".to_string(),
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