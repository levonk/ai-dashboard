use analytics_rs::{Processor, Aggregator, CostCalculator, FilterEngine, TelemetryEvent, AnalyticsQuery, AnalyticsResult};
use anyhow::Result;

pub struct AnalyticsProcessor {
    processor: Processor,
    aggregator: Aggregator,
    cost_calculator: CostCalculator,
    filter_engine: FilterEngine,
}

impl AnalyticsProcessor {
    pub fn new() -> Self {
        Self {
            processor: Processor,
            aggregator: Aggregator::new(),
            cost_calculator: CostCalculator::new(),
            filter_engine: FilterEngine,
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
}

impl Default for AnalyticsProcessor {
    fn default() -> Self {
        Self::new()
    }
}