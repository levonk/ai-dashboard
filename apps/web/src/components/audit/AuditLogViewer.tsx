'use client';

import { useState } from 'react';
import { AuditEvent, AuditEventType, AuditSeverity } from '@/types/audit';

interface AuditLogViewerProps {
  events: AuditEvent[];
  onRefresh: () => void;
  onExport: () => void;
}

export function AuditLogViewer({ events, onRefresh, onExport }: AuditLogViewerProps) {
  const [filter, setFilter] = useState({
    eventType: 'all',
    severity: 'all',
    userId: '',
    dateFrom: '',
    dateTo: '',
  });
  const [expandedRows, setExpandedRows] = useState<Set<string>>(new Set());

  const filteredEvents = events.filter((event) => {
    if (filter.eventType !== 'all' && event.event_type !== filter.eventType) return false;
    if (filter.severity !== 'all' && event.severity !== filter.severity) return false;
    if (filter.userId && event.user_id !== filter.userId) return false;
    if (filter.dateFrom && new Date(event.timestamp) < new Date(filter.dateFrom)) return false;
    if (filter.dateTo && new Date(event.timestamp) > new Date(filter.to)) return false;
    return true;
  });

  const getSeverityColor = (severity: AuditSeverity) => {
    switch (severity) {
      case AuditSeverity.Critical:
        return 'bg-red-100 text-red-800';
      case AuditSeverity.Error:
        return 'bg-orange-100 text-orange-800';
      case AuditSeverity.Warning:
        return 'bg-yellow-100 text-yellow-800';
      case AuditSeverity.Info:
        return 'bg-blue-100 text-blue-800';
      default:
        return 'bg-gray-100 text-gray-800';
    }
  };

  const getEventTypeLabel = (eventType: AuditEventType) => {
    switch (eventType) {
      case AuditEventType.UserLogin:
        return 'User Login';
      case AuditEventType.UserLogout:
        return 'User Logout';
      case AuditEventType.LoginFailed:
        return 'Login Failed';
      case AuditEventType.UserCreated:
        return 'User Created';
      case AuditEventType.UserUpdated:
        return 'User Updated';
      case AuditEventType.UserDeleted:
        return 'User Deleted';
      case AuditEventType.DataRead:
        return 'Data Read';
      case AuditEventType.DataExported:
        return 'Data Exported';
      case AuditEventType.DataDeleted:
        return 'Data Deleted';
      case AuditEventType.ConfigUpdated:
        return 'Config Updated';
      case AuditEventType.SystemError:
        return 'System Error';
      case AuditEventType.ApiAccess:
        return 'API Access';
      case AuditEventType.ApiError:
        return 'API Error';
      default:
        return eventType.toString();
    }
  };

  const toggleRow = (id: string) => {
    const newExpanded = new Set(expandedRows);
    if (newExpanded.has(id)) {
      newExpanded.delete(id);
    } else {
      newExpanded.add(id);
    }
    setExpandedRows(newExpanded);
  };

  return (
    <div className="audit-log-viewer">
      <div className="flex justify-between items-center mb-6">
        <h2 className="text-2xl font-bold">Audit Log Viewer</h2>
        <div className="space-x-2">
          <button
            onClick={onRefresh}
            className="px-4 py-2 bg-blue-600 text-white rounded hover:bg-blue-700"
          >
            Refresh
          </button>
          <button
            onClick={onExport}
            className="px-4 py-2 bg-green-600 text-white rounded hover:bg-green-700"
          >
            Export
          </button>
        </div>
      </div>

      <div className="bg-white shadow rounded-lg p-4 mb-4">
        <div className="grid grid-cols-1 md:grid-cols-4 gap-4">
          <div>
            <label className="block text-sm font-medium text-gray-700 mb-1">Event Type</label>
            <select
              value={filter.eventType}
              onChange={(e) => setFilter({ ...filter, eventType: e.target.value })}
              className="block w-full rounded-md border-gray-300 shadow-sm focus:border-blue-500 focus:ring-blue-500"
            >
              <option value="all">All Events</option>
              <option value={AuditEventType.UserLogin}>User Login</option>
              <option value={AuditEventType.UserLogout}>User Logout</option>
              <option value={AuditEventType.UserCreated}>User Created</option>
              <option value={AuditEventType.UserUpdated}>User Updated</option>
              <option value={AuditEventType.UserDeleted}>User Deleted</option>
              <option value={AuditEventType.DataRead}>Data Read</option>
              <option value={AuditEventType.DataExported}>Data Exported</option>
              <option value={AuditEventType.SystemError}>System Error</option>
            </select>
          </div>
          <div>
            <label className="block text-sm font-medium text-gray-700 mb-1">Severity</label>
            <select
              value={filter.severity}
              onChange={(e) => setFilter({ ...filter, severity: e.target.value })}
              className="block w-full rounded-md border-gray-300 shadow-sm focus:border-blue-500 focus:ring-blue-500"
            >
              <option value="all">All Severities</option>
              <option value={AuditSeverity.Info}>Info</option>
              <option value={AuditSeverity.Warning}>Warning</option>
              <option value={AuditSeverity.Error}>Error</option>
              <option value={AuditSeverity.Critical}>Critical</option>
            </select>
          </div>
          <div>
            <label className="block text-sm font-medium text-gray-700 mb-1">User ID</label>
            <input
              type="text"
              value={filter.userId}
              onChange={(e) => setFilter({ ...filter, userId: e.target.value })}
              placeholder="Filter by user ID"
              className="block w-full rounded-md border-gray-300 shadow-sm focus:border-blue-500 focus:ring-blue-500"
            />
          </div>
          <div>
            <label className="block text-sm font-medium text-gray-700 mb-1">Date Range</label>
            <div className="flex space-x-2">
              <input
                type="date"
                value={filter.dateFrom}
                onChange={(e) => setFilter({ ...filter, dateFrom: e.target.value })}
                className="block w-full rounded-md border-gray-300 shadow-sm focus:border-blue-500 focus:ring-blue-500"
              />
              <input
                type="date"
                value={filter.dateTo}
                onChange={(e) => setFilter({ ...filter, dateTo: e.target.value })}
                className="block w-full rounded-md border-gray-300 shadow-sm focus:border-blue-500 focus:ring-blue-500"
              />
            </div>
          </div>
        </div>
      </div>

      <div className="bg-white shadow rounded-lg overflow-hidden">
        <table className="min-w-full divide-y divide-gray-200">
          <thead className="bg-gray-50">
            <tr>
              <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                Timestamp
              </th>
              <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                Event Type
              </th>
              <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                Severity
              </th>
              <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                User
              </th>
              <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                Resource
              </th>
              <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                Action
              </th>
              <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                Status
              </th>
              <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                Details
              </th>
            </tr>
          </thead>
          <tbody className="bg-white divide-y divide-gray-200">
            {filteredEvents.map((event) => (
              <>
                <tr key={event.id} className="hover:bg-gray-50 cursor-pointer" onClick={() => toggleRow(event.id)}>
                  <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-500">
                    {new Date(event.timestamp).toLocaleString()}
                  </td>
                  <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
                    {getEventTypeLabel(event.event_type)}
                  </td>
                  <td className="px-6 py-4 whitespace-nowrap">
                    <span className={`inline-flex rounded-full px-2 text-xs font-semibold leading-5 ${getSeverityColor(event.severity)}`}>
                      {event.severity}
                    </span>
                  </td>
                  <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-500">
                    {event.user_id || 'System'}
                  </td>
                  <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-500">
                    {event.resource}
                  </td>
                  <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-500">
                    {event.action}
                  </td>
                  <td className="px-6 py-4 whitespace-nowrap">
                    <span className={`inline-flex rounded-full px-2 text-xs font-semibold leading-5 ${
                      event.success ? 'bg-green-100 text-green-800' : 'bg-red-100 text-red-800'
                    }`}>
                      {event.success ? 'Success' : 'Failed'}
                    </span>
                  </td>
                  <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-500">
                    {expandedRows.has(event.id) ? '▼' : '▶'}
                  </td>
                </tr>
                {expandedRows.has(event.id) && (
                  <tr key={`${event.id}-details`} className="bg-gray-50">
                    <td colSpan={9} className="px-6 py-4">
                      <div className="space-y-2">
                        <div>
                          <span className="font-medium">Session ID:</span> {event.session_id || 'N/A'}
                        </div>
                        <div>
                          <span className="font-medium">IP Address:</span> {event.ip_address || 'N/A'}
                        </div>
                        <div>
                          <span className="font-medium">User Agent:</span> {event.user_agent || 'N/A'}
                        </div>
                        {event.error_message && (
                          <div>
                            <span className="font-medium text-red-600">Error:</span> {event.error_message}
                          </div>
                        )}
                        <div>
                          <span className="font-medium">Details:</span>
                          <pre className="mt-1 p-2 bg-gray-100 rounded text-xs overflow-auto">
                            {JSON.stringify(event.details, null, 2)}
                          </pre>
                        </div>
                      </div>
                    </td>
                  </tr>
                )}
              </>
            ))}
          </tbody>
        </table>
      </div>

      {filteredEvents.length === 0 && (
        <div className="text-center py-8 text-gray-500">
          No audit logs found matching the current filters.
        </div>
      )}
    </div>
  );
}