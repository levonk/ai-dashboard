'use client';

import React, { useState, useEffect } from 'react';
import { Card } from '../ui/card';
import { Button } from '../ui/button';

interface AlertEvent {
  id: string;
  rule_id: string;
  rule_name: string;
  severity: string;
  triggered_at: string;
  metric_value: number;
  threshold_value: number;
  message: string;
  context: Record<string, string>;
  resolved: boolean;
  resolved_at: string | null;
  resolution_notes: string | null;
}

interface AlertEventsListProps {
  ruleId?: string;
}

export function AlertEventsList({ ruleId }: AlertEventsListProps) {
  const [events, setEvents] = useState<AlertEvent[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    fetchAlertEvents();
  }, [ruleId]);

  const fetchAlertEvents = async () => {
    try {
      setLoading(true);
      const url = ruleId 
        ? `/api/alerts/rules/${ruleId}/events`
        : '/api/alerts/events';
      const response = await fetch(url);
      if (!response.ok) {
        throw new Error('Failed to fetch alert events');
      }
      const data = await response.json();
      setEvents(data);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'An error occurred');
    } finally {
      setLoading(false);
    }
  };

  const resolveEvent = async (eventId: string) => {
    const notes = prompt('Enter resolution notes (optional):');
    try {
      const response = await fetch(`/api/alerts/events/${eventId}/resolve`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ notes: notes || null }),
      });
      if (!response.ok) {
        throw new Error('Failed to resolve event');
      }
      await fetchAlertEvents();
    } catch (err) {
      setError(err instanceof Error ? err.message : 'An error occurred');
    }
  };

  const getSeverityColor = (severity: string) => {
    switch (severity.toLowerCase()) {
      case 'critical':
        return 'border-l-red-500';
      case 'error':
        return 'border-l-orange-500';
      case 'warning':
        return 'border-l-yellow-500';
      case 'info':
        return 'border-l-blue-500';
      default:
        return 'border-l-gray-500';
    }
  };

  if (loading) {
    return <div className="p-4">Loading alert events...</div>;
  }

  if (error) {
    return <div className="p-4 text-red-600">Error: {error}</div>;
  }

  return (
    <div className="space-y-4">
      <div className="flex justify-between items-center">
        <h2 className="text-2xl font-bold">Alert Events</h2>
        <Button onClick={fetchAlertEvents}>Refresh</Button>
      </div>

      {events.length === 0 ? (
        <Card className="p-8 text-center text-gray-500">
          No alert events found.
        </Card>
      ) : (
        <div className="space-y-3">
          {events.map((event) => (
            <Card key={event.id} className={`p-4 border-l-4 ${getSeverityColor(event.severity)}`}>
              <div className="flex items-start justify-between">
                <div className="flex-1">
                  <div className="flex items-center gap-3">
                    <h3 className="font-semibold">{event.rule_name}</h3>
                    <span className={`px-2 py-1 rounded text-xs font-medium ${
                      event.resolved ? 'bg-green-100 text-green-800' : 'bg-red-100 text-red-800'
                    }`}>
                      {event.resolved ? 'Resolved' : 'Active'}
                    </span>
                  </div>
                  <p className="text-sm text-gray-600 mt-1">{event.message}</p>
                  <div className="text-xs text-gray-500 mt-2">
                    Value: {event.metric_value} • Threshold: {event.threshold_value}
                  </div>
                  <div className="text-xs text-gray-500">
                    Triggered: {new Date(event.triggered_at).toLocaleString()}
                  </div>
                  {event.resolved && event.resolved_at && (
                    <div className="text-xs text-gray-500">
                      Resolved: {new Date(event.resolved_at).toLocaleString()}
                    </div>
                  )}
                  {event.resolution_notes && (
                    <div className="text-xs text-gray-600 mt-1 italic">
                      Notes: {event.resolution_notes}
                    </div>
                  )}
                </div>
                {!event.resolved && (
                  <Button
                    size="sm"
                    onClick={() => resolveEvent(event.id)}
                  >
                    Resolve
                  </Button>
                )}
              </div>
            </Card>
          ))}
        </div>
      )}
    </div>
  );
}