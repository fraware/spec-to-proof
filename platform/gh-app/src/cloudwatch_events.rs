use std::collections::HashMap;
use std::time::{Duration, Instant};
use serde::{Deserialize, Serialize};
use anyhow::Result;
use tracing::{info, warn, error};
use aws_sdk_events::Client as EventsClient;
use aws_sdk_events::types::{Rule, Target, EcsParameters, TaskDefinitionOverride};
use aws_sdk_ecs::Client as EcsClient;
use aws_sdk_lambda::Client as LambdaClient;
use aws_sdk_lambda::types::{InvocationType, LogType};
use tokio::sync::RwLock;

// CloudWatch Events configuration
#[derive(Debug, Clone)]
pub struct CloudWatchEventsConfig {
    pub cost_report_schedule: String, // cron expression
    pub budget_alert_schedule: String,
    pub drift_backlog_schedule: String,
    pub events_bus_name: String,
    pub lambda_function_name: String,
    pub ecs_cluster_name: String,
    pub ecs_task_definition: String,
}

impl Default for CloudWatchEventsConfig {
    fn default() -> Self {
        Self {
            cost_report_schedule: "cron(0 8 * * ? *)".to_string(), // Daily at 8 AM UTC
            budget_alert_schedule: "cron(0 */6 * * ? *)".to_string(), // Every 6 hours
            drift_backlog_schedule: "cron(0 */2 * * ? *)".to_string(), // Every 2 hours
            events_bus_name: "default".to_string(),
            lambda_function_name: "spec-to-proof-cost-governance".to_string(),
            ecs_cluster_name: "spec-to-proof-cluster".to_string(),
            ecs_task_definition: "spec-to-proof-task".to_string(),
        }
    }
}

// Event types
#[derive(Debug, Serialize, Deserialize)]
pub enum GovernanceEvent {
    DailyCostReport {
        date: String,
        services: Vec<String>,
    },
    BudgetAlert {
        current_cost: f64,
        threshold: f64,
        service: String,
    },
    DriftBacklogAlert {
        backlog_count: u32,
        threshold: u32,
        oldest_drift_hours: u32,
    },
    TokenBucketReset {
        tenant_id: String,
        reason: String,
    },
    LLMCallLimit {
        tenant_id: String,
        current_usage: u32,
        limit: u32,
    },
}

// CloudWatch Events manager
pub struct CloudWatchEventsManager {
    config: CloudWatchEventsConfig,
    events_client: EventsClient,
    lambda_client: LambdaClient,
    ecs_client: EcsClient,
    rule_cache: RwLock<HashMap<String, String>>, // rule_name -> rule_arn
}

impl CloudWatchEventsManager {
    pub fn new(
        config: CloudWatchEventsConfig,
        events_client: EventsClient,
        lambda_client: LambdaClient,
        ecs_client: EcsClient,
    ) -> Self {
        Self {
            config,
            events_client,
            lambda_client,
            ecs_client,
            rule_cache: RwLock::new(HashMap::new()),
        }
    }

    pub async fn setup_cost_governance_rules(&self) -> Result<()> {
        info!("Setting up CloudWatch Events rules for cost governance");

        // Create daily cost report rule
        self.create_cost_report_rule().await?;

        // Create budget alert rule
        self.create_budget_alert_rule().await?;

        // Create drift backlog alert rule
        self.create_drift_backlog_rule().await?;

        info!("CloudWatch Events rules setup completed");
        Ok(())
    }

    async fn create_cost_report_rule(&self) -> Result<()> {
        let rule_name = "spec-to-proof-daily-cost-report";
        let rule_description = "Triggers daily cost report generation";

        let rule = Rule::builder()
            .name(rule_name)
            .description(rule_description)
            .schedule_expression(&self.config.cost_report_schedule)
            .state("ENABLED")
            .build()?;

        let response = self.events_client
            .put_rule()
            .name(rule_name)
            .description(rule_description)
            .schedule_expression(&self.config.cost_report_schedule)
            .state("ENABLED")
            .send()
            .await?;

        let rule_arn = response.rule_arn().unwrap();
        
        // Add Lambda target
        let target = Target::builder()
            .id("cost-report-lambda")
            .arn(format!("arn:aws:lambda:us-east-1:123456789012:function:{}", self.config.lambda_function_name))
            .build()?;

        self.events_client
            .put_targets()
            .rule(rule_name)
            .targets(target)
            .send()
            .await?;

        // Cache the rule ARN
        {
            let mut cache = self.rule_cache.write().await;
            cache.insert(rule_name.to_string(), rule_arn.to_string());
        }

        info!("Created cost report rule: {}", rule_arn);
        Ok(())
    }

    async fn create_budget_alert_rule(&self) -> Result<()> {
        let rule_name = "spec-to-proof-budget-alert";
        let rule_description = "Triggers budget alerts when thresholds are exceeded";

        let response = self.events_client
            .put_rule()
            .name(rule_name)
            .description(rule_description)
            .schedule_expression(&self.config.budget_alert_schedule)
            .state("ENABLED")
            .send()
            .await?;

        let rule_arn = response.rule_arn().unwrap();

        // Add Lambda target
        let target = Target::builder()
            .id("budget-alert-lambda")
            .arn(format!("arn:aws:lambda:us-east-1:123456789012:function:{}", self.config.lambda_function_name))
            .build()?;

        self.events_client
            .put_targets()
            .rule(rule_name)
            .targets(target)
            .send()
            .await?;

        // Cache the rule ARN
        {
            let mut cache = self.rule_cache.write().await;
            cache.insert(rule_name.to_string(), rule_arn.to_string());
        }

        info!("Created budget alert rule: {}", rule_arn);
        Ok(())
    }

    async fn create_drift_backlog_rule(&self) -> Result<()> {
        let rule_name = "spec-to-proof-drift-backlog-alert";
        let rule_description = "Triggers alerts when drift backlog exceeds threshold";

        let response = self.events_client
            .put_rule()
            .name(rule_name)
            .description(rule_description)
            .schedule_expression(&self.config.drift_backlog_schedule)
            .state("ENABLED")
            .send()
            .await?;

        let rule_arn = response.rule_arn().unwrap();

        // Add Lambda target
        let target = Target::builder()
            .id("drift-backlog-lambda")
            .arn(format!("arn:aws:lambda:us-east-1:123456789012:function:{}", self.config.lambda_function_name))
            .build()?;

        self.events_client
            .put_targets()
            .rule(rule_name)
            .targets(target)
            .send()
            .await?;

        // Cache the rule ARN
        {
            let mut cache = self.rule_cache.write().await;
            cache.insert(rule_name.to_string(), rule_arn.to_string());
        }

        info!("Created drift backlog rule: {}", rule_arn);
        Ok(())
    }

    pub async fn trigger_cost_report(&self) -> Result<()> {
        let event = GovernanceEvent::DailyCostReport {
            date: chrono::Utc::now().format("%Y-%m-%d").to_string(),
            services: vec![
                "Amazon SageMaker".to_string(),
                "Amazon S3".to_string(),
                "Amazon DynamoDB".to_string(),
                "Amazon CloudWatch".to_string(),
                "AWS Lambda".to_string(),
            ],
        };

        self.invoke_lambda_function(&event).await?;
        Ok(())
    }

    pub async fn trigger_budget_alert(&self, current_cost: f64, threshold: f64, service: &str) -> Result<()> {
        let event = GovernanceEvent::BudgetAlert {
            current_cost,
            threshold,
            service: service.to_string(),
        };

        self.invoke_lambda_function(&event).await?;
        Ok(())
    }

    pub async fn trigger_drift_backlog_alert(&self, backlog_count: u32, threshold: u32, oldest_drift_hours: u32) -> Result<()> {
        let event = GovernanceEvent::DriftBacklogAlert {
            backlog_count,
            threshold,
            oldest_drift_hours,
        };

        self.invoke_lambda_function(&event).await?;
        Ok(())
    }

    async fn invoke_lambda_function(&self, event: &GovernanceEvent) -> Result<()> {
        let payload = serde_json::to_string(event)?;

        let response = self.lambda_client
            .invoke()
            .function_name(&self.config.lambda_function_name)
            .invocation_type(InvocationType::Event)
            .log_type(LogType::Tail)
            .payload(aws_sdk_lambda::types::Blob::new(payload))
            .send()
            .await?;

        info!("Lambda function invoked: {:?}", response.status_code);
        Ok(())
    }

    pub async fn get_rule_status(&self, rule_name: &str) -> Result<String> {
        let response = self.events_client
            .describe_rule()
            .name(rule_name)
            .send()
            .await?;

        Ok(response.state().unwrap_or("UNKNOWN").to_string())
    }

    pub async fn disable_rule(&self, rule_name: &str) -> Result<()> {
        self.events_client
            .disable_rule()
            .name(rule_name)
            .send()
            .await?;

        info!("Disabled rule: {}", rule_name);
        Ok(())
    }

    pub async fn enable_rule(&self, rule_name: &str) -> Result<()> {
        self.events_client
            .enable_rule()
            .name(rule_name)
            .send()
            .await?;

        info!("Enabled rule: {}", rule_name);
        Ok(())
    }

    pub async fn list_rules(&self) -> Result<Vec<String>> {
        let response = self.events_client
            .list_rules()
            .name_prefix("spec-to-proof")
            .send()
            .await?;

        let rules: Vec<String> = response.rules()
            .unwrap_or_default()
            .iter()
            .filter_map(|rule| rule.name().map(|s| s.to_string()))
            .collect();

        Ok(rules)
    }
}

// Lambda function handler for cost governance events
pub async fn handle_governance_event(event: GovernanceEvent) -> Result<()> {
    match event {
        GovernanceEvent::DailyCostReport { date, services } => {
            info!("Processing daily cost report for date: {}", date);
            // TODO: Implement actual cost report generation
            todo!("Implement cost report generation");
        }
        GovernanceEvent::BudgetAlert { current_cost, threshold, service } => {
            warn!("Budget alert triggered: cost={:.2}, threshold={:.2}, service={}", 
                  current_cost, threshold, service);
            // TODO: Implement budget alert handling
            todo!("Implement budget alert handling");
        }
        GovernanceEvent::DriftBacklogAlert { backlog_count, threshold, oldest_drift_hours } => {
            warn!("Drift backlog alert: count={}, threshold={}, oldest={}h", 
                  backlog_count, threshold, oldest_drift_hours);
            // TODO: Implement drift backlog alert handling
            todo!("Implement drift backlog alert handling");
        }
        GovernanceEvent::TokenBucketReset { tenant_id, reason } => {
            info!("Token bucket reset for tenant {}: {}", tenant_id, reason);
            // TODO: Implement token bucket reset
            todo!("Implement token bucket reset");
        }
        GovernanceEvent::LLMCallLimit { tenant_id, current_usage, limit } => {
            warn!("LLM call limit reached for tenant {}: {}/{}", tenant_id, current_usage, limit);
            // TODO: Implement LLM call limit handling
            todo!("Implement LLM call limit handling");
        }
    }
}

// Unit test for CloudWatch Events rule creation
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_rule_creation() {
        // TODO: Implement with mock AWS clients
        todo!("Implement CloudWatch Events test");
    }

    #[tokio::test]
    async fn test_event_handling() {
        let event = GovernanceEvent::DailyCostReport {
            date: "2024-01-01".to_string(),
            services: vec!["Amazon SageMaker".to_string()],
        };

        // TODO: Implement actual event handling test
        todo!("Implement event handling test");
    }
} 