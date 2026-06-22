use axum::{Json, response::IntoResponse, extract::{State, Path}, http::StatusCode};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use analytics_rs::{AlertRule, AlertEvent, AlertStorage, StorageError, NotificationChannel, ChannelType, ChannelConfig, AlertCondition, ComparisonOperator, TimeWindow, AggregationType, AlertSeverity};

#[derive(Serialize)]
pub struct AlertRuleResponse {
    pub id: String,
    pub name: String,
    pub description: String,
    pub enabled: bool,
    pub severity: String,
    pub cooldown_period: i64,
    pub created_at: String,
    pub updated_at: String,
    pub created_by: String,
}

#[derive(Deserialize)]
pub struct CreateAlertRuleRequest {
    pub name: String,
    pub description: String,
    pub conditions: serde_json::Value,
    pub notification_channels: Vec<serde_json::Value>,
    pub severity: String,
    pub cooldown_period: i64,
    pub created_by: String,
}

#[derive(Deserialize)]
pub struct UpdateAlertRuleRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub conditions: Option<serde_json::Value>,
    pub notification_channels: Option<Vec<serde_json::Value>>,
    pub severity: Option<String>,
    pub cooldown_period: Option<i64>,
    pub enabled: Option<bool>,
}

#[derive(Serialize)]
pub struct AlertEventResponse {
    pub id: String,
    pub rule_id: String,
    pub rule_name: String,
    pub severity: String,
    pub triggered_at: String,
    pub metric_value: f64,
    pub threshold_value: f64,
    pub message: String,
    pub context: std::collections::HashMap<String, String>,
    pub resolved: bool,
    pub resolved_at: Option<String>,
    pub resolution_notes: Option<String>,
}

#[derive(Serialize)]
pub struct AlertStatisticsResponse {
    pub total_alerts: i64,
    pub active_alerts: i64,
    pub resolved_alerts: i64,
    pub alerts_by_severity: std::collections::HashMap<String, i64>,
    pub alerts_by_rule: std::collections::HashMap<String, i64>,
    pub average_resolution_time: Option<i64>,
}

#[derive(Serialize)]
pub struct ErrorResponse {
    pub error: String,
}

pub async fn create_alert_rule(
    State(storage): State<Arc<AlertStorage>>,
    Json(req): Json<CreateAlertRuleRequest>,
) -> impl IntoResponse {
    // TODO: Parse conditions and notification channels from JSON
    // For now, create a simple threshold rule as placeholder
    
    let rule = AlertRule::new(
        uuid::Uuid::new_v4().to_string(),
        req.name,
        req.description,
        AlertCondition::Threshold {
            metric: "cost".to_string(), // Placeholder
            operator: ComparisonOperator::GreaterThan,
            value: 100.0, // Placeholder
            time_window: TimeWindow {
                duration: 3600,
                aggregation: AggregationType::Sum,
            },
        },
        vec![], // Placeholder - no notification channels
        AlertSeverity::Warning, // Placeholder
        req.cooldown_period,
        req.created_by,
    );

    match storage.add_rule(rule.clone()) {
        Ok(_) => {
            let response = AlertRuleResponse {
                id: rule.id,
                name: rule.name,
                description: rule.description,
                enabled: rule.enabled,
                severity: format!("{:?}", rule.severity),
                cooldown_period: rule.cooldown_period,
                created_at: rule.created_at.to_rfc3339(),
                updated_at: rule.updated_at.to_rfc3339(),
                created_by: rule.created_by,
            };
            (StatusCode::CREATED, Json(response)).into_response()
        }
        Err(e) => {
            let error = ErrorResponse { error: e.to_string() };
            (StatusCode::BAD_REQUEST, Json(error)).into_response()
        }
    }
}

pub async fn get_alert_rule(
    State(storage): State<Arc<AlertStorage>>,
    Path(rule_id): Path<String>,
) -> impl IntoResponse {
    match storage.get_rule(&rule_id) {
        Ok(rule) => {
            let response = AlertRuleResponse {
                id: rule.id,
                name: rule.name,
                description: rule.description,
                enabled: rule.enabled,
                severity: format!("{:?}", rule.severity),
                cooldown_period: rule.cooldown_period,
                created_at: rule.created_at.to_rfc3339(),
                updated_at: rule.updated_at.to_rfc3339(),
                created_by: rule.created_by,
            };
            (StatusCode::OK, Json(response)).into_response()
        }
        Err(e) => {
            let error = ErrorResponse { error: e.to_string() };
            (StatusCode::NOT_FOUND, Json(error)).into_response()
        }
    }
}

pub async fn list_alert_rules(
    State(storage): State<Arc<AlertStorage>>,
) -> impl IntoResponse {
    match storage.list_rules() {
        Ok(rules) => {
            let responses: Vec<AlertRuleResponse> = rules
                .into_iter()
                .map(|rule| AlertRuleResponse {
                    id: rule.id,
                    name: rule.name,
                    description: rule.description,
                    enabled: rule.enabled,
                    severity: format!("{:?}", rule.severity),
                    cooldown_period: rule.cooldown_period,
                    created_at: rule.created_at.to_rfc3339(),
                    updated_at: rule.updated_at.to_rfc3339(),
                    created_by: rule.created_by,
                })
                .collect();
            (StatusCode::OK, Json(responses)).into_response()
        }
        Err(e) => {
            let error = ErrorResponse { error: e.to_string() };
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error)).into_response()
        }
    }
}

pub async fn update_alert_rule(
    State(storage): State<Arc<AlertStorage>>,
    Path(rule_id): Path<String>,
    Json(req): Json<UpdateAlertRuleRequest>,
) -> impl IntoResponse {
    match storage.get_rule(&rule_id) {
        Ok(mut rule) => {
            // Update fields if provided
            if let Some(name) = req.name {
                rule.name = name;
            }
            if let Some(description) = req.description {
                rule.description = description;
            }
            if let Some(enabled) = req.enabled {
                rule.enabled = enabled;
            }
            if let Some(cooldown_period) = req.cooldown_period {
                rule.cooldown_period = cooldown_period;
            }
            // TODO: Update conditions and notification channels if provided
            
            match storage.update_rule(rule.clone()) {
                Ok(_) => {
                    let response = AlertRuleResponse {
                        id: rule.id,
                        name: rule.name,
                        description: rule.description,
                        enabled: rule.enabled,
                        severity: format!("{:?}", rule.severity),
                        cooldown_period: rule.cooldown_period,
                        created_at: rule.created_at.to_rfc3339(),
                        updated_at: rule.updated_at.to_rfc3339(),
                        created_by: rule.created_by,
                    };
                    (StatusCode::OK, Json(response)).into_response()
                }
                Err(e) => {
                    let error = ErrorResponse { error: e.to_string() };
                    (StatusCode::BAD_REQUEST, Json(error)).into_response()
                }
            }
        }
        Err(e) => {
            let error = ErrorResponse { error: e.to_string() };
            (StatusCode::NOT_FOUND, Json(error)).into_response()
        }
    }
}

pub async fn delete_alert_rule(
    State(storage): State<Arc<AlertStorage>>,
    Path(rule_id): Path<String>,
) -> impl IntoResponse {
    match storage.delete_rule(&rule_id) {
        Ok(_) => (StatusCode::NO_CONTENT, ()).into_response(),
        Err(e) => {
            let error = ErrorResponse { error: e.to_string() };
            (StatusCode::NOT_FOUND, Json(error)).into_response()
        }
    }
}

pub async fn get_alert_events(
    State(storage): State<Arc<AlertStorage>>,
    Path(rule_id): Path<String>,
) -> impl IntoResponse {
    match storage.list_events_by_rule(&rule_id) {
        Ok(events) => {
            let responses: Vec<AlertEventResponse> = events
                .into_iter()
                .map(|event| AlertEventResponse {
                    id: event.id,
                    rule_id: event.rule_id,
                    rule_name: event.rule_name,
                    severity: format!("{:?}", event.severity),
                    triggered_at: event.triggered_at.to_rfc3339(),
                    metric_value: event.metric_value,
                    threshold_value: event.threshold_value,
                    message: event.message,
                    context: event.context,
                    resolved: event.resolved,
                    resolved_at: event.resolved_at.map(|t| t.to_rfc3339()),
                    resolution_notes: event.resolution_notes,
                })
                .collect();
            (StatusCode::OK, Json(responses)).into_response()
        }
        Err(e) => {
            let error = ErrorResponse { error: e.to_string() };
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error)).into_response()
        }
    }
}

pub async fn get_alert_statistics(
    State(storage): State<Arc<AlertStorage>>,
) -> impl IntoResponse {
    match storage.get_statistics() {
        Ok(stats) => {
            let alerts_by_severity: std::collections::HashMap<String, i64> = stats
                .alerts_by_severity
                .into_iter()
                .map(|(k, v)| (format!("{:?}", k), v))
                .collect();

            let response = AlertStatisticsResponse {
                total_alerts: stats.total_alerts,
                active_alerts: stats.active_alerts,
                resolved_alerts: stats.resolved_alerts,
                alerts_by_severity,
                alerts_by_rule: stats.alerts_by_rule,
                average_resolution_time: stats.average_resolution_time,
            };
            (StatusCode::OK, Json(response)).into_response()
        }
        Err(e) => {
            let error = ErrorResponse { error: e.to_string() };
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error)).into_response()
        }
    }
}

pub async fn resolve_alert_event(
    State(storage): State<Arc<AlertStorage>>,
    Path(event_id): Path<String>,
    Json(req): Json<serde_json::Value>,
) -> impl IntoResponse {
    let notes = req.get("notes")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    match storage.resolve_event(&event_id, notes) {
        Ok(_) => (StatusCode::NO_CONTENT, ()).into_response(),
        Err(e) => {
            let error = ErrorResponse { error: e.to_string() };
            (StatusCode::NOT_FOUND, Json(error)).into_response()
        }
    }
}