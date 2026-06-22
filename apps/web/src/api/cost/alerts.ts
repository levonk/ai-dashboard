/**
 * Cost Alerting and Threshold Monitoring
 * 
 * Manages cost alerts and threshold-based monitoring
 */

import {
  CostAlertRequest,
  CostAlertResponse,
  CostAlertStatus,
  CostAlertTrigger,
  CostAlertCondition
} from './types';

export class CostAlertManager {
  private alerts: Map<string, CostAlertRequest['alert']> = new Map();
  private alertStatuses: Map<string, CostAlertStatus> = new Map();

  /**
   * Create or update a cost alert
   */
  async createAlert(request: CostAlertRequest): Promise<CostAlertResponse> {
    const alertId = this.generateAlertId(request.alert);

    try {
      // Validate the alert configuration
      const validation = this.validateAlert(request.alert);
      if (!validation.valid) {
        return {
          alertId,
          status: 'created',
          alert: request.alert,
          validation: {
            valid: false,
            errors: validation.errors
          }
        };
      }

      // Store the alert
      this.alerts.set(alertId, request.alert);
      
      // Initialize alert status
      this.alertStatuses.set(alertId, {
        alertId,
        name: request.alert.name,
        status: 'active',
        triggerHistory: [],
        currentState: {
          triggerCount: 0
        }
      });

      return {
        alertId,
        status: 'created',
        alert: request.alert,
        validation: {
          valid: true
        }
      };
    } catch (error) {
      throw this.handleError(error, alertId);
    }
  }

  /**
   * Get alert status
   */
  async getAlertStatus(alertId: string): Promise<CostAlertStatus | null> {
    return this.alertStatuses.get(alertId) || null;
  }

  /**
   * List all alerts
   */
  async listAlerts(): Promise<CostAlertStatus[]> {
    return Array.from(this.alertStatuses.values());
  }

  /**
   * Update an existing alert
   */
  async updateAlert(alertId: string, request: CostAlertRequest): Promise<CostAlertResponse> {
    if (!this.alerts.has(alertId)) {
      throw new Error(`Alert ${alertId} not found`);
    }

    try {
      // Validate the alert configuration
      const validation = this.validateAlert(request.alert);
      if (!validation.valid) {
        return {
          alertId,
          status: 'updated',
          alert: request.alert,
          validation: {
            valid: false,
            errors: validation.errors
          }
        };
      }

      // Update the alert
      this.alerts.set(alertId, request.alert);
      
      // Update alert status name
      const status = this.alertStatuses.get(alertId);
      if (status) {
        status.name = request.alert.name;
      }

      return {
        alertId,
        status: 'updated',
        alert: request.alert,
        validation: {
          valid: true
        }
      };
    } catch (error) {
      throw this.handleError(error, alertId);
    }
  }

  /**
   * Delete an alert
   */
  async deleteAlert(alertId: string): Promise<boolean> {
    this.alerts.delete(alertId);
    this.alertStatuses.delete(alertId);
    return true;
  }

  /**
   * Pause an alert
   */
  async pauseAlert(alertId: string): Promise<boolean> {
    const status = this.alertStatuses.get(alertId);
    if (status) {
      status.status = 'paused';
      return true;
    }
    return false;
  }

  /**
   * Resume an alert
   */
  async resumeAlert(alertId: string): Promise<boolean> {
    const status = this.alertStatuses.get(alertId);
    if (status) {
      status.status = 'active';
      return true;
    }
    return false;
  }

  /**
   * Check alert conditions against current cost data
   */
  async checkAlerts(currentCostData: {
    totalCost: number;
    costByDimension: Record<string, number>;
    timestamp: string;
  }): Promise<CostAlertTrigger[]> {
    const triggers: CostAlertTrigger[] = [];

    for (const [alertId, alert] of this.alerts.entries()) {
      const status = this.alertStatuses.get(alertId);
      if (!status || status.status !== 'active') continue;

      // Check each condition
      for (const condition of alert.conditions) {
        const trigger = await this.checkCondition(alertId, alert, condition, currentCostData);
        if (trigger) {
          triggers.push(trigger);
          
          // Update alert status
          status.currentState.lastTriggered = trigger.timestamp;
          status.currentState.triggerCount++;
          status.triggerHistory.push(trigger);
          
          // Send notification if configured
          await this.sendNotification(alert, trigger);
        }
      }
    }

    return triggers;
  }

  /**
   * Check a single alert condition
   */
  private async checkCondition(
    alertId: string,
    alert: CostAlertRequest['alert'],
    condition: CostAlertCondition,
    currentCostData: { totalCost: number; costByDimension: Record<string, number>; timestamp: string }
  ): Promise<CostAlertTrigger | null> {
    let currentValue: number;
    let threshold: number;

    switch (condition.type) {
      case 'total_cost':
        currentValue = currentCostData.totalCost;
        threshold = condition.value;
        break;
      
      case 'cost_increase':
        // Compare with previous period (simplified)
        currentValue = currentCostData.totalCost;
        threshold = condition.value;
        break;
      
      case 'anomaly':
        // Check for anomaly (simplified - would use actual anomaly detection)
        currentValue = currentCostData.totalCost;
        threshold = condition.value;
        break;
      
      case 'budget_exceeded':
        currentValue = currentCostData.totalCost;
        threshold = condition.value;
        break;
      
      default:
        return null;
    }

    // Evaluate condition
    const triggered = this.evaluateCondition(currentValue, condition.operator, threshold);
    
    if (triggered) {
      const severity = this.calculateSeverity(condition, currentValue, threshold);
      
      return {
        triggerId: this.generateTriggerId(),
        timestamp: currentCostData.timestamp,
        condition,
        value: currentValue,
        threshold,
        severity,
        acknowledged: false
      };
    }

    return null;
  }

  /**
   * Evaluate condition operator
   */
  private evaluateCondition(value: number, operator: string, threshold: number): boolean {
    switch (operator) {
      case 'gt':
        return value > threshold;
      case 'gte':
        return value >= threshold;
      case 'lt':
        return value < threshold;
      case 'lte':
        return value <= threshold;
      case 'eq':
        return value === threshold;
      default:
        return false;
    }
  }

  /**
   * Calculate trigger severity
   */
  private calculateSeverity(condition: CostAlertCondition, value: number, threshold: number): 'low' | 'medium' | 'high' {
    const percentage = Math.abs((value - threshold) / threshold) * 100;
    
    if (percentage < 10) return 'low';
    if (percentage < 25) return 'medium';
    return 'high';
  }

  /**
   * Send notification for triggered alert
   */
  private async sendNotification(alert: CostAlertRequest['alert'], trigger: CostAlertTrigger): Promise<void> {
    // TODO: Implement actual notification sending
    // This would integrate with email, Slack, webhooks, etc.
    
    console.log(`[Cost Alert] Notification triggered for alert: ${alert.name}`, {
      severity: trigger.severity,
      value: trigger.value,
      threshold: trigger.threshold,
      channels: alert.notifications.channels
    });
  }

  /**
   * Validate alert configuration
   */
  private validateAlert(alert: CostAlertRequest['alert']): { valid: boolean; errors?: string[] } {
    const errors: string[] = [];

    // Validate name
    if (!alert.name || alert.name.trim().length === 0) {
      errors.push('Alert name is required');
    }

    // Validate conditions
    if (!alert.conditions || alert.conditions.length === 0) {
      errors.push('At least one condition is required');
    } else {
      alert.conditions.forEach((condition, index) => {
        if (!condition.type) {
          errors.push(`Condition ${index + 1}: type is required`);
        }
        if (!condition.operator) {
          errors.push(`Condition ${index + 1}: operator is required`);
        }
        if (condition.value === undefined || condition.value === null) {
          errors.push(`Condition ${index + 1}: value is required`);
        }
      });
    }

    // Validate thresholds
    if (!alert.thresholds) {
      errors.push('Thresholds are required');
    } else {
      const hasThreshold = 
        alert.thresholds.costThreshold !== undefined ||
        alert.thresholds.percentageChange !== undefined ||
        alert.thresholds.anomalySeverity !== undefined;
      
      if (!hasThreshold) {
        errors.push('At least one threshold must be specified');
      }
    }

    // Validate notifications
    if (!alert.notifications || !alert.notifications.channels || alert.notifications.channels.length === 0) {
      errors.push('At least one notification channel is required');
    }

    // Validate webhook URL if webhook channel is specified
    if (alert.notifications?.channels.includes('webhook') && !alert.notifications?.webhookUrl) {
      errors.push('Webhook URL is required when webhook channel is specified');
    }

    return {
      valid: errors.length === 0,
      errors: errors.length > 0 ? errors : undefined
    };
  }

  /**
   * Handle errors
   */
  private handleError(error: any, alertId: string): Error {
    console.error(`Cost alert error (${alertId}):`, error);
    return new Error(`Cost alert operation failed: ${error.message}`);
  }

  /**
   * Generate alert ID
   */
  private generateAlertId(alert: CostAlertRequest['alert']): string {
    const hash = this.hashAlert(alert);
    return `alert_${Date.now()}_${hash.substring(0, 8)}`;
  }

  /**
   * Generate trigger ID
   */
  private generateTriggerId(): string {
    return `trigger_${Date.now()}_${Math.random().toString(36).substring(2, 9)}`;
  }

  /**
   * Hash alert for identification
   */
  private hashAlert(alert: CostAlertRequest['alert']): string {
    const str = JSON.stringify(alert);
    let hash = 0;
    for (let i = 0; i < str.length; i++) {
      const char = str.charCodeAt(i);
      hash = ((hash << 5) - hash) + char;
      hash = hash & hash;
    }
    return Math.abs(hash).toString(16);
  }
}