---
# Product Requirements Document (PRD)

## Introduction / Overview
- **Feature name:** AI Analytics Dashboard System
- **Summary:** A comprehensive analytics platform for AI usage across multiple dimensions: company clients, AI clients (Claude Code, Codex, Pi, Devin, etc.), teams, pipeline stages, AI model suppliers (Anthropic, OpenAI, Google, Microsoft, AWS, OpenRouter, etc.), models, and input types (text/chat, image, audio, etc.).
- **Context:**
  - This feature is for organizations and AI platform providers who need deep visibility into AI usage patterns across their entire AI infrastructure.
  - Addresses the lack of comprehensive analytics that can handle the complexity of modern AI stacks with multiple clients, providers, models, and transformation stages.
  - Supports dual licensing model: AGPL 3.0 for open-source users, commercial license for enterprise features and multi-tenancy.
  - Open-source version uses 2-service architecture (proxy + web) designed for single-tenant deployments.
  - Commercial version will use 4-service architecture (proxy + collector + analytics + web) for multi-tenant scale.

## Goals
- Provide comprehensive analytics across all AI usage dimensions (clients, teams, providers, models, pipeline stages, input types)
- Support single-tenant deployment with architecture that can scale to multi-tenant
- Deliver actionable insights for cost optimization, security monitoring, performance improvement, and compliance
- Create extensible architecture for future commercial multi-tenant capabilities
- Maintain dual licensing model (AGPL 3.0 open-source + commercial license available)

## User Stories

### Single-Tenant Users
- As a developer using multiple AI coding agents, I want to see comprehensive analytics across all my AI tools in one dashboard.
- As a small team, I want to understand our AI usage patterns across different providers and models to optimize costs.
- As a security-conscious user, I want to monitor AI requests for anomalous patterns and potential security issues.
- As a DevOps engineer, I want to deploy this easily in my existing infrastructure with minimal configuration.
- As a researcher, I want to analyze AI usage patterns across different models and providers to understand performance characteristics.

### Future Multi-Tenant Capabilities (Commercial License)
- As an organization, I want to monitor AI usage across multiple teams and departments with proper data isolation.
- As a platform provider, I want to offer analytics services to customers (requires commercial license).
- As a managed service provider, I want to provide analytics dashboards to multiple clients (requires commercial license).

## Functional Requirements

### Multi-Dimensional Data Collection
- **Company Clients**: Support multiple company/client identification and isolation
- **AI Clients**: Track usage across different AI coding agents (Claude Code, Codex, Pi, Devin, Cursor, Cline, etc.)
- **Teams**: Support team/sub-organization hierarchy within clients
- **Pipeline Stages**: Collect analytics at multiple stages:
  - Pre-optimization (original requests)
  - Post-optimization (after compression/transformation)
  - Per-stage collection (Headroom, OmniRoute, Iron-Proxy, custom stages)
- **AI Model Suppliers**: Track usage across providers (Anthropic, OpenAI, Google, Microsoft, AWS, OpenRouter, etc.)
- **Models**: Analytics at individual model level (GPT-4, Claude 3.5 Opus, Gemini Pro, etc.)
- **Input Types**: Support different input modalities (text/chat, image, audio, video, code, etc.)

### Data Collection Architecture
- **Collector Framework**: Pluggable collector system for different pipeline stages
- **Standardized Metadata**: Common metadata schema across all collectors for consistent analytics
- **Content Hashing**: Request fingerprinting for correlation across pipeline stages
- **Token Estimation**: Accurate token counting for different models and input types
- **Timing Metrics**: High-precision timing at each pipeline stage
- **Error Tracking**: Comprehensive error classification and tracking

### Analytics Processing
- **Real-time Aggregation**: Stream processing for real-time dashboards and alerts
- **Batch Processing**: Scheduled jobs for deep analytics and reporting
- **Comparative Analysis**: Compare metrics across dimensions (clients, providers, models, etc.)
- **Trend Analysis**: Time-series analysis for usage patterns and anomalies
- **Cost Calculation**: Multi-provider cost modeling with accurate pricing data
- **Compression Analytics**: Measure effectiveness of optimization stages

### Extensible Architecture (Foundation for Future Multi-Tenant)
- **Data Isolation Ready**: Architecture designed to support future tenant isolation strategies
- **Scalable Data Model**: Data model designed to support multi-dimensional filtering and aggregation
- **Configuration System**: Extensible configuration system for future tenant-specific settings
- **API Structure**: API designed to support future multi-tenant endpoints and authentication

### Dashboard and Visualization
- **Multi-Dimensional Filtering**: Filter analytics by any combination of dimensions
- **Drill-Down Capability**: From high-level metrics to individual request details
- **Custom Dashboards**: User-configurable dashboard layouts and widgets
- **Export Capabilities**: Export analytics data in multiple formats:
  - **End-User Exports**: CSV, JSON, PDF for human consumption
  - **AI Agent Exports**: ToonFormat (https://toonformat.dev/) for bulk data transfer to minimize token usage
  - **Single Record Exports**: JSON for individual record transfer in plaintext protocols
- **Responsive Design**: Works across desktop, tablet, and mobile devices
- **Multi-Format Output**: Support both HTML (for end-users) and Markdown/ToonFormat (for AI agents) from the same underlying data

### Alerting and Notifications
- **Real-Time Alerts**: Configurable alerts for usage thresholds, anomalies, cost overruns
- **Alert Channels**: Multiple notification channels (email, Slack, webhooks, SMS)
- **Alert Rules**: Flexible rule engine for alert conditions
- **Alert History**: Track alert history and resolution status
- **Anomaly Detection**: ML-based anomaly detection for unusual patterns

### Security and Compliance
- **Authentication**: Basic authentication with support for future SSO integration
- **Authorization**: Role-based access control (admin, viewer, analyst)
- **Audit Logging**: Comprehensive audit trail for all system operations
- **Data Retention**: Configurable data retention policies
- **Data Encryption**: Encryption at rest and in transit
- **Security Best Practices**: Following OWASP guidelines and security best practices

### API and Integration
- **REST API**: Comprehensive REST API for all analytics operations
- **Webhooks**: Real-time webhook notifications for events
- **SDK Support**: Official SDKs for popular languages (Python, JavaScript, Go)
- **Collector SDK**: Easy integration for custom pipeline stages
- **Data Export**: Bulk data export capabilities for external analysis

### Multi-Output Interface Requirements
- **Web Dashboard (End-User Interface)**: HTML format for human users browsing the dashboard
- **AI Agent Interface**: Support standard AI agent consumption patterns:
  - Direct service API access
  - Markdown output format
  - Both API and markdown output options
- **AI-to-Service Data Exchange**:
  - **Single Records**: Use JSON format for plaintext protocol exchanges (do not use binary formats like Protocol Buffers, Apache Thrift, Captain Proto, Apache Avro for plaintext protocols)
  - **Bulk Data Transfer**: Use ToonFormat (https://toonformat.dev/) instead of JSON to minimize token usage when transferring large datasets between AI agents and services
- **Service-to-AI Data Exchange**:
  - **Single Records**: Use JSON format for plaintext protocol responses
  - **Bulk Data Transfer**: Use ToonFormat (https://toonformat.dev/) instead of JSON to minimize token usage when providing large datasets to AI agents

### Licensing and Contribution
- **Open Source License**: AGPL 3.0 for all open-source features
- **Commercial License Available**: Commercial license available for organizations requiring multi-tenant, white-label, or proprietary use
- **Contributor Agreement**: CLA for contributors to enable dual licensing model
- **Clear Feature Separation**: Documentation clearly distinguishes open-source vs commercial features

## Non-Functional Requirements

### Performance (Open-Source)
- **Ingestion Latency**: <200ms from request to analytics availability
- **Query Performance**: <5 seconds for standard dashboard queries
- **Concurrent Users**: Support 10+ concurrent dashboard users
- **Throughput**: Support 100+ requests/second
- **Data Freshness**: Near real-time data with <30 second latency

### Performance (Commercial - Future)
- **Ingestion Latency**: <100ms from request to analytics availability
- **Query Performance**: <2 seconds for standard dashboard queries
- **Concurrent Users**: Support 1000+ concurrent dashboard users
- **Throughput**: Support 10,000+ requests/second per tenant
- **Data Freshness**: Real-time data with <5 second latency

### Scalability (Open-Source)
- **Vertical Scaling**: Components designed for vertical scaling
- **Database Performance**: Optimized queries and indexing for single-tenant workload
- **Cache Strategy**: Optional Redis for dashboard query performance
- **Resource Management**: Configurable resource limits and monitoring

### Scalability (Commercial - Future)
- **Horizontal Scaling**: All components must support horizontal scaling
- **Database Scaling**: Support read replicas and connection pooling
- **Queue Scaling**: Support Redis clustering for higher throughput
- **Performance**: Efficient query patterns and caching strategies
- **Resource Management**: Configurable resource limits and monitoring

### Reliability
- **Availability**: 99.5% uptime target for self-hosted deployments
- **Data Durability**: Proper backup and recovery procedures
- **Graceful Degradation**: System remains functional during partial failures
- **Error Handling**: Comprehensive error handling and recovery
- **Monitoring**: Health check endpoints and logging

### Security
- **Data Protection**: Proper input validation and sanitization
- **Encryption**: TLS 1.3 for data in transit, optional encryption at rest
- **Vulnerability Management**: Regular dependency updates and security scanning
- **Security Best Practices**: Following OWASP guidelines
- **Documentation**: Clear security documentation and best practices

### Maintainability
- **Modular Architecture**: Clear separation of concerns with well-defined interfaces
- **Documentation**: Comprehensive documentation for all components
- **Testing**: >80% code coverage with integration and E2E tests
- **Logging**: Structured logging with configurable levels and destinations
- **Monitoring**: Prometheus metrics export and Grafana dashboards

### Usability
- **Intuitive UI**: Clean, modern interface with minimal learning curve
- **Responsive Design**: Works seamlessly on desktop, tablet, and mobile
- **Accessibility**: WCAG 2.1 AA compliance
- **Internationalization**: Support for multiple languages and time zones
- **Onboarding**: Guided setup and onboarding process

## Technical Considerations

### Architecture Components (Open-Source)
- **Proxy Service**: Rust-based proxy that routes AI requests and collects telemetry data
  - Default mode: "analytics mode" - writes telemetry directly to database
  - Alternative mode: "emitter mode" - sends telemetry to downstream collector (commercial)
  - Routes requests to AI providers (Anthropic, OpenAI, Google, etc.)
  - Handles API key security and request/response processing
  - Uses shared analytics-rs package for on-demand processing
- **Analytics Package**: Rust library (`packages/analytics-rs`) with reusable analytics processing logic
  - Aggregation functions (count, sum, average, percentiles)
  - Cost calculation engine with provider-specific pricing
  - Multi-dimensional filtering and time-series analysis
  - Can be consumed by proxy, analytics service, or Spark jobs
- **Web Service**: Next.js TypeScript dashboard for visualization and analytics
  - On-demand analytics queries against database
  - Real-time dashboard updates via polling
  - Multi-dimensional filtering and drill-down capabilities
- **Database**: PostgreSQL for analytics data storage
  - Request/response events with dimensional attributes
  - User accounts and basic authentication
  - Alert rules and notification history
- **Cache**: Optional Redis for dashboard query performance

### Architecture Components (Commercial - Future)
- **Proxy Service**: Operates in "emitter mode" for high-scale deployments
- **Analytics Service**: Consumes analytics-rs package for real-time aggregation and ML capabilities
- **Apache Spark Jobs**: Consumes analytics-rs package for batch processing and deep analytics
- **Web Service**: Enhanced dashboard with real-time streaming and advanced features
- **Message Queue**: Redis or Kafka for high-throughput processing
- **Time-Series DB**: TimescaleDB for optimized time-series analytics

### Data Model
- **Tenants**: Tenant configuration, settings, quotas
- **Users**: User accounts, roles, permissions
- **Requests**: Request events with all dimensional attributes
- **Responses**: Response events with metrics and metadata
- **Aggregations**: Pre-computed aggregates for common queries
- **Alerts**: Alert rules, history, and notifications
- **Audit Logs**: System operation audit trail

### Integration Points
- **AI Clients**: Native integrations with popular AI coding agents
- **Pipeline Stages**: Standard integration protocol for custom stages
- **Identity Providers**: SSO integration (Okta, Auth0, Azure AD)
- **Notification Services**: Slack, email, SMS, webhook providers
- **Monitoring**: Prometheus, Grafana, DataDog integration
- **Billing**: Integration with billing systems for commercial customers

### Technology Stack (Open-Source)
- **Proxy Backend**: Rust with axum web framework, tokio async runtime
- **Analytics Library**: Rust package (`analytics-rs`) with reusable processing logic
- **Web Frontend**: Next.js with TypeScript, modern component library
- **Databases**: PostgreSQL for analytics data storage
- **Cache**: Optional Redis for dashboard query performance
- **Infrastructure**: Docker and Docker Compose for deployment
- **Monitoring**: Prometheus metrics export, health check endpoints
- **CI/CD**: GitHub Actions for CI/CD

### Technology Stack (Commercial - Future)
- **Proxy Backend**: Rust with emitter mode for high-scale deployments
- **Analytics Library**: Same `analytics-rs` package consumed by multiple services
- **Analytics Service**: Rust service consuming analytics-rs for real-time processing
- **Apache Spark**: Spark jobs consuming analytics-rs for batch processing
- **Web Frontend**: Enhanced Next.js with real-time streaming
- **Databases**: PostgreSQL, TimescaleDB for time-series data, Redis for caching
- **Message Queue**: Redis or Kafka for high-throughput processing
- **ML Framework**: Integration with ML libraries for anomaly detection

### Licensing Strategy
- **Open Source (AGPL 3.0)**:
  - Single-tenant deployment
  - Comprehensive analytics features
  - Community support via GitHub
  - Self-hosted deployment
  - Full source code access
- **Commercial License (Available)**:
  - Multi-tenant support
  - White-label capabilities
  - Priority support and SLA
  - Proprietary use without network copyleft
  - Custom integrations and features
  - See business plan for commercial offering details

## Success Metrics
- **Adoption**: 100+ open-source installations in first year, 500+ by year 2
- **Performance**: <200ms ingestion latency, <5s query performance consistently (open-source)
- **Reliability**: 99.5% uptime target for self-hosted deployments
- **Community**: 200+ GitHub stars in first year, active contributor community
- **Documentation**: Comprehensive documentation with clear setup guides
- **Integration**: Successful integrations with 5+ major AI clients
- **Commercial Interest**: Demonstrated commercial interest for multi-tenant features

## Open Questions
- What are the minimum viable features for the initial open-source release?
- Which AI clients should we prioritize for initial integrations?
- What level of multi-dimensional analytics is needed for MVP?
- How do we balance feature completeness with time to market?
- What commercial features show the strongest market demand for future development?

## Dependencies
- **Infrastructure**: Docker and Docker Compose for local development and deployment
- **AI Client APIs**: Access to AI client APIs for integration testing
- **Monitoring**: Prometheus and Grafana for system monitoring
- **Documentation Tools**: Tools for generating and maintaining documentation
- **Community Platforms**: GitHub for issues, discussions, and contributions

## Timeline / Milestones

### Phase 1: Foundation (Weeks 1-4)
- Set up project structure and licensing framework
- Implement AGPL 3.0 licensing and contributor agreement
- Create proxy service with analytics mode (default)
- Implement single-tenant data model in PostgreSQL
- Build basic Next.js dashboard with core visualizations
- Set up CI/CD pipeline

### Phase 2: Multi-Dimensional Analytics (Weeks 5-8)
- Implement multi-dimensional data collection in proxy
- Add support for major AI clients (Claude Code, Codex, Cursor)
- Create pipeline stage analytics
- Implement comparative analysis features
- Add comprehensive filtering and drill-down
- Support major AI providers (Anthropic, OpenAI, Google)

### Phase 3: Advanced Features (Weeks 9-12)
- Implement on-demand analytics queries in web service
- Add basic alerting and notification system
- Create custom dashboard builder
- Implement advanced cost analysis
- Add data export and reporting capabilities
- Support additional input types (image, audio)

### Phase 4: Integration and Polish (Weeks 13-16)
- Integrate with additional AI clients and providers
- Implement comprehensive testing suite
- Add monitoring and observability
- Create deployment automation
- Performance optimization and load testing
- Security hardening and best practices

### Phase 5: Documentation and Launch (Weeks 17-20)
- Complete documentation and setup guides
- Create tutorials and examples
- Build community resources
- Launch open-source offering
- Establish contribution guidelines
- Set up community support channels

### Phase 6: Future Commercial Planning (Weeks 21-24)
- Gather user feedback and usage patterns
- Identify most requested commercial features
- Plan proxy emitter mode implementation
- Design collector and analytics service architecture
- Prepare commercial licensing framework
- Document business requirements for commercial version

---
*Generated from PRD template*