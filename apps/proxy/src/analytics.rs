use analytics_rs::{Processor, Aggregator, CostCalculator, FilterEngine, TelemetryEvent, AnalyticsQuery, AnalyticsResult};
use anyhow::Result;
use crate::collection::{
    TokenCollector, CostInfo, TokenThroughput, TokenGenerationRate,
    TokenCachingMetrics, TokenStreamingMetrics, TokenProviderCorrelation,
    TokenMetricsAggregator, StreamingTokenData, StreamingChunk
};
use std::time::Duration;
use std::collections::HashMap;

pub struct AnalyticsProcessor {
    processor: Processor,
    aggregator: Aggregator,
    cost_calculator: CostCalculator,
    filter_engine: FilterEngine,
    token_collector: TokenCollector,
}

impl AnalyticsProcessor {
    pub fn new() -> Self {
        Self {
            processor: Processor,
            aggregator: Aggregator::new(),
            cost_calculator: CostCalculator::new(),
            filter_engine: FilterEngine,
            token_collector: TokenCollector::new(),
        }
    }

    pub fn process_telemetry(&self, events: &[TelemetryEvent], query: &AnalyticsQuery) -> Result<AnalyticsResult> {
        // Filter events based on query
        let filtered_events = FilterEngine::apply(events, &query.filters)?;
        
        // Process the filtered events
        let result = Processor::process_events(&filtered_events, query)?;
        
        Ok(result)
    }

    pub fn calculate_costs(&self, events: &[TelemetryEvent]) -> Result<f64> {
        self.cost_calculator.calculate_total_cost(events)
    }

    pub fn aggregate_metrics(&self, events: &[TelemetryEvent], query: &AnalyticsQuery) -> Result<serde_json::Value> {
        let filtered_events = FilterEngine::apply(events, &query.filters)?;
        self.aggregator.aggregate(&filtered_events, query)
    }

    /// Process token metrics for a request
    pub fn process_token_metrics(
        &self,
        request_id: String,
        provider: String,
        model: String,
        request_data: &serde_json::Value,
        response_data: &serde_json::Value,
        duration: Duration,
    ) -> Result<TokenProviderCorrelation> {
        // Extract token counts
        let mut token_count = self.token_collector.extract_request_tokens(request_data, &model)?;
        self.token_collector.update_with_response(&mut token_count, response_data, &model)?;

        // Calculate costs
        let input_cost = self.token_collector.calculate_cost(&provider, &model, token_count.input_tokens, 0)?;
        let output_cost = self.token_collector.calculate_cost(&provider, &model, 0, token_count.output_tokens)?;
        let total_cost = input_cost + output_cost;

        let cost_info = CostInfo::new(
            input_cost,
            output_cost,
            self.token_collector.calculate_cost(&provider, &model, 1000, 0)?,
            self.token_collector.calculate_cost(&provider, &model, 0, 1000)?,
        );

        // Calculate throughput
        let throughput = TokenThroughput::calculate(request_id.clone(), &token_count, duration);

        // Create correlation
        let correlation = TokenProviderCorrelation::create(
            request_id,
            provider,
            model,
            token_count,
            cost_info,
            duration.as_millis() as u64,
            throughput,
            true,
            None,
        );

        Ok(correlation)
    }

    /// Process streaming token metrics
    pub fn process_streaming_metrics(
        &self,
        request_id: String,
        streaming_data: &[StreamingTokenData],
    ) -> TokenStreamingMetrics {
        let chunks: Vec<StreamingChunk> = streaming_data
            .iter()
            .map(|data| StreamingChunk {
                elapsed_ms: data.elapsed_ms,
                token_count: data.token_count,
            })
            .collect();

        TokenStreamingMetrics::calculate(request_id, &chunks)
    }

    /// Process generation rate analysis
    pub fn process_generation_rate(
        &self,
        request_id: String,
        streaming_data: &[StreamingTokenData],
    ) -> TokenGenerationRate {
        TokenGenerationRate::analyze(request_id, streaming_data)
    }

    /// Aggregate token metrics across multiple requests
    pub fn aggregate_token_metrics(
        &self,
        correlations: &[TokenProviderCorrelation],
    ) -> HashMap<String, crate::collection::ProviderAggregation> {
        TokenMetricsAggregator::aggregate_by_provider(correlations)
    }

    /// Calculate caching efficiency
    pub fn calculate_caching_efficiency(
        &self,
        request_id: String,
        cache_hits: u32,
        cache_misses: u32,
        tokens_saved: u32,
        model: &str,
    ) -> TokenCachingMetrics {
        let cost_per_1k_tokens = self
            .token_collector
            .calculate_cost("anthropic", model, 1000, 0)
            .unwrap_or(0.0);
        let cost_per_token = cost_per_1k_tokens / 1000.0;

        TokenCachingMetrics::calculate(request_id, cache_hits, cache_misses, tokens_saved, cost_per_token)
    }
}

impl Default for AnalyticsProcessor {
    fn default() -> Self {
        Self::new()
    }
}