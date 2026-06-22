use crate::models::{TelemetryEvent, AnalyticsQuery, AnalyticsResult, QueryMetadata};
use crate::{Aggregator, FilterEngine, CostCalculator, TimeSeriesAnalyzer};
use anyhow::Result;
use std::time::Instant;

pub struct Processor;

impl Processor {
    pub fn process_events(events: &[TelemetryEvent], query: &AnalyticsQuery) -> Result<AnalyticsResult> {
        let start_time = Instant::now();
        
        // Step 1: Apply time range filter
        let time_filtered = FilterEngine::apply_time_range_filter(
            events,
            query.time_range.start,
            query.time_range.end
        )?;
        
        // Step 2: Apply additional filters
        let filtered = FilterEngine::apply(&time_filtered, &query.filters)?;
        
        // Step 3: Perform aggregation
        let aggregator = Aggregator::new();
        let data = aggregator.aggregate(&filtered, query)?;
        
        let query_duration = start_time.elapsed();
        
        Ok(AnalyticsResult {
            data,
            metadata: QueryMetadata {
                query_duration_ms: query_duration.as_millis() as u64,
                events_processed: filtered.len() as u64,
                time_range: query.time_range.clone(),
            },
        })
    }
    
    pub fn process_with_grouping(events: &[TelemetryEvent], query: &AnalyticsQuery) -> Result<AnalyticsResult> {
        let start_time = Instant::now();
        
        // Apply filters
        let time_filtered = FilterEngine::apply_time_range_filter(
            events,
            query.time_range.start,
            query.time_range.end
        )?;
        
        let filtered = FilterEngine::apply(&time_filtered, &query.filters)?;
        
        // Group by specified fields
        let grouped_data = Self::group_events(&filtered, &query.group_by)?;
        
        // Apply aggregation to each group
        let aggregator = Aggregator::new();
        let mut results = Vec::new();
        for (group_key, group_events) in grouped_data {
            let mut group_query = query.clone();
            group_query.group_by = vec![]; // Remove grouping for individual aggregation
            
            let aggregated = aggregator.aggregate(&group_events, &group_query)?;
            
            results.push(serde_json::json!({
                "group": group_key,
                "count": group_events.len(),
                "data": aggregated
            }));
        }
        
        let query_duration = start_time.elapsed();
        
        Ok(AnalyticsResult {
            data: serde_json::json!(results),
            metadata: QueryMetadata {
                query_duration_ms: query_duration.as_millis() as u64,
                events_processed: filtered.len() as u64,
                time_range: query.time_range.clone(),
            },
        })
    }
    
    pub fn process_cost_analysis(events: &[TelemetryEvent], query: &AnalyticsQuery) -> Result<AnalyticsResult> {
        let start_time = Instant::now();
        
        // Apply filters
        let time_filtered = FilterEngine::apply_time_range_filter(
            events,
            query.time_range.start,
            query.time_range.end
        )?;
        
        let filtered = FilterEngine::apply(&time_filtered, &query.filters)?;
        
        // Calculate cost breakdowns
        let total_cost = CostCalculator::calculate_total_cost(&filtered)?;
        let cost_by_provider = CostCalculator::calculate_cost_by_provider(&filtered)?;
        let cost_by_model = CostCalculator::calculate_cost_by_model(&filtered)?;
        
        let query_duration = start_time.elapsed();
        
        Ok(AnalyticsResult {
            data: serde_json::json!({
                "total_cost_usd": total_cost,
                "cost_by_provider": cost_by_provider,
                "cost_by_model": cost_by_model,
                "average_cost_per_event": if filtered.is_empty() { 0.0 } else { total_cost / filtered.len() as f64 }
            }),
            metadata: QueryMetadata {
                query_duration_ms: query_duration.as_millis() as u64,
                events_processed: filtered.len() as u64,
                time_range: query.time_range.clone(),
            },
        })
    }
    
    pub fn process_time_series_analysis(events: &[TelemetryEvent], query: &AnalyticsQuery) -> Result<AnalyticsResult> {
        let start_time = Instant::now();
        
        // Apply filters
        let time_filtered = FilterEngine::apply_time_range_filter(
            events,
            query.time_range.start,
            query.time_range.end
        )?;
        
        let filtered = FilterEngine::apply(&time_filtered, &query.filters)?;
        
        // Perform time series analysis
        let time_series = TimeSeriesAnalyzer::analyze_time_series(&filtered, &query.time_range)?;
        let trends = TimeSeriesAnalyzer::calculate_trends(&filtered)?;
        let anomalies = TimeSeriesAnalyzer::detect_anomalies(&filtered)?;
        
        let query_duration = start_time.elapsed();
        
        Ok(AnalyticsResult {
            data: serde_json::json!({
                "time_series": time_series,
                "trends": trends,
                "anomalies": anomalies
            }),
            metadata: QueryMetadata {
                query_duration_ms: query_duration.as_millis() as u64,
                events_processed: filtered.len() as u64,
                time_range: query.time_range.clone(),
            },
        })
    }
    
    fn group_events(events: &[TelemetryEvent], group_by: &[String]) -> Result<Vec<(String, Vec<TelemetryEvent>)>> {
        if group_by.is_empty() {
            return Ok(vec![("all".to_string(), events.to_vec())]);
        }
        
        let mut groups: std::collections::HashMap<String, Vec<TelemetryEvent>> = std::collections::HashMap::new();
        
        for event in events {
            let group_key = group_by.iter()
                .map(|field| Self::get_group_key_value(event, field))
                .collect::<Vec<_>>()
                .join("/");
            
            groups.entry(group_key).or_insert_with(Vec::new).push(event.clone());
        }
        
        Ok(groups.into_iter().collect())
    }
    
    fn get_group_key_value(event: &TelemetryEvent, field: &str) -> String {
        match field {
            "ai_client" => event.ai_client.clone(),
            "ai_provider" => event.ai_provider.clone(),
            "model" => event.model.clone(),
            "input_type" => event.input_type.clone(),
            _ => "unknown".to_string(),
        }
    }
    
    pub fn validate_query(query: &AnalyticsQuery) -> Result<()> {
        // Validate time range
        if query.time_range.start > query.time_range.end {
            return Err(anyhow::anyhow!("Time range start must be before end"));
        }
        
        // Validate group_by fields
        let valid_fields = ["ai_client", "ai_provider", "model", "input_type"];
        for field in &query.group_by {
            if !valid_fields.contains(&field.as_str()) {
                return Err(anyhow::anyhow!("Invalid group_by field: {}", field));
            }
        }
        
        Ok(())
    }
    
    pub fn get_query_statistics(events: &[TelemetryEvent], query: &AnalyticsQuery) -> Result<serde_json::Value> {
        let time_filtered = FilterEngine::apply_time_range_filter(
            events,
            query.time_range.start,
            query.time_range.end
        )?;
        
        let filtered = FilterEngine::apply(&time_filtered, &query.filters)?;
        
        let unique_clients: std::collections::HashSet<&String> = filtered.iter().map(|e| &e.ai_client).collect();
        let unique_providers: std::collections::HashSet<&String> = filtered.iter().map(|e| &e.ai_provider).collect();
        let unique_models: std::collections::HashSet<&String> = filtered.iter().map(|e| &e.model).collect();
        
        Ok(serde_json::json!({
            "total_events_in_range": time_filtered.len(),
            "events_after_filters": filtered.len(),
            "unique_clients": unique_clients.len(),
            "unique_providers": unique_providers.len(),
            "unique_models": unique_models.len(),
            "filter_count": query.filters.len(),
            "group_by_fields": query.group_by
        }))
    }
}