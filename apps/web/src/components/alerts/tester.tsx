'use client';

import React, { useState } from 'react';
import { Card } from '../ui/card';
import { Button } from '../ui/button';
import { Input } from '../ui/input';
import { Textarea } from '../ui/textarea';
import { Label } from '../ui/label';

interface AlertTestResult {
  triggered: boolean;
  metric_value: number;
  threshold_value: number;
  message: string;
  context: Record<string, string>;
}

export function AlertTester() {
  const [metricName, setMetricName] = useState('cost');
  const [metricValue, setMetricValue] = useState('150');
  const [thresholdValue, setThresholdValue] = useState('100');
  const [operator, setOperator] = useState('greater_than');
  const [testResult, setTestResult] = useState<AlertTestResult | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const runTest = async () => {
    try {
      setLoading(true);
      setError(null);
      setTestResult(null);

      // Simulate alert evaluation (in real implementation, this would call the API)
      const metricValueNum = parseFloat(metricValue);
      const thresholdValueNum = parseFloat(thresholdValue);

      if (isNaN(metricValueNum) || isNaN(thresholdValueNum)) {
        throw new Error('Invalid metric or threshold value');
      }

      let triggered = false;
      switch (operator) {
        case 'greater_than':
          triggered = metricValueNum > thresholdValueNum;
          break;
        case 'greater_than_or_equal':
          triggered = metricValueNum >= thresholdValueNum;
          break;
        case 'less_than':
          triggered = metricValueNum < thresholdValueNum;
          break;
        case 'less_than_or_equal':
          triggered = metricValueNum <= thresholdValueNum;
          break;
        case 'equal':
          triggered = Math.abs(metricValueNum - thresholdValueNum) < 0.0001;
          break;
        case 'not_equal':
          triggered = Math.abs(metricValueNum - thresholdValueNum) >= 0.0001;
          break;
      }

      const result: AlertTestResult = {
        triggered,
        metric_value: metricValueNum,
        threshold_value: thresholdValueNum,
        message: triggered
          ? `Alert would trigger: ${metricName} ${operator.replace('_', ' ')} ${thresholdValueNum} (current: ${metricValueNum})`
          : `Alert would not trigger: ${metricName} within normal range (current: ${metricValueNum}, threshold: ${thresholdValueNum})`,
        context: {
          metric: metricName,
          operator,
          threshold: thresholdValueNum.toString(),
          current_value: metricValueNum.toString(),
        },
      };

      setTestResult(result);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'An error occurred');
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="space-y-4">
      <h2 className="text-2xl font-bold">Alert Rule Tester</h2>
      <Card className="p-6">
        <div className="space-y-4">
          <div>
            <Label htmlFor="metric-name">Metric Name</Label>
            <Input
              id="metric-name"
              value={metricName}
              onChange={(e) => setMetricName(e.target.value)}
              placeholder="e.g., cost, requests, latency"
            />
          </div>

          <div>
            <Label htmlFor="metric-value">Current Metric Value</Label>
            <Input
              id="metric-value"
              type="number"
              step="any"
              value={metricValue}
              onChange={(e) => setMetricValue(e.target.value)}
              placeholder="e.g., 150"
            />
          </div>

          <div>
            <Label htmlFor="threshold-value">Threshold Value</Label>
            <Input
              id="threshold-value"
              type="number"
              step="any"
              value={thresholdValue}
              onChange={(e) => setThresholdValue(e.target.value)}
              placeholder="e.g., 100"
            />
          </div>

          <div>
            <Label htmlFor="operator">Comparison Operator</Label>
            <select
              id="operator"
              value={operator}
              onChange={(e) => setOperator(e.target.value)}
              className="w-full p-2 border rounded"
            >
              <option value="greater_than">Greater Than (&gt;)</option>
              <option value="greater_than_or_equal">Greater Than or Equal (&gt;=)</option>
              <option value="less_than">Less Than (&lt;)</option>
              <option value="less_than_or_equal">Less Than or Equal (&lt;=)</option>
              <option value="equal">Equal (=)</option>
              <option value="not_equal">Not Equal (!=)</option>
            </select>
          </div>

          <Button onClick={runTest} disabled={loading} className="w-full">
            {loading ? 'Testing...' : 'Run Test'}
          </Button>

          {error && (
            <div className="p-3 bg-red-50 border border-red-200 rounded text-red-600">
              {error}
            </div>
          )}

          {testResult && (
            <div className={`p-4 border rounded ${
              testResult.triggered
                ? 'bg-red-50 border-red-200'
                : 'bg-green-50 border-green-200'
            }`}>
              <div className="font-semibold mb-2">
                {testResult.triggered ? '⚠️ Alert Would Trigger' : '✅ Alert Would Not Trigger'}
              </div>
              <div className="text-sm">{testResult.message}</div>
              <div className="mt-3 text-xs text-gray-600">
                <div>Metric: {testResult.context.metric}</div>
                <div>Current Value: {testResult.context.current_value}</div>
                <div>Threshold: {testResult.context.threshold}</div>
                <div>Operator: {testResult.context.operator}</div>
              </div>
            </div>
          )}
        </div>
      </Card>
    </div>
  );
}