import { DashboardShell } from "../components/layout/dashboard-shell";
import { FilterBar } from "../components/filters/filter-bar";
import { MetricCard } from "../components/charts/metric-card";
import { BarChart3, DollarSign, Cpu, Users } from "lucide-react";

export default function Page() {
  return (
    <DashboardShell>
      <div className="space-y-6">
        <div>
          <h1 className="text-2xl font-bold text-gray-900">Dashboard Overview</h1>
          <p className="text-gray-500 mt-1">AI usage analytics and insights</p>
        </div>

        <FilterBar />

        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
          <MetricCard
            title="Total Requests"
            value="0"
            subtitle="Last 30 days"
            icon={BarChart3}
          />
          <MetricCard
            title="Total Cost"
            value="$0.00"
            subtitle="Last 30 days"
            icon={DollarSign}
          />
          <MetricCard
            title="Active Models"
            value="0"
            subtitle="Currently in use"
            icon={Cpu}
          />
          <MetricCard
            title="Active Teams"
            value="0"
            subtitle="With activity"
            icon={Users}
          />
        </div>

        <div className="bg-white rounded-lg border border-gray-200 p-6">
          <h2 className="text-lg font-semibold text-gray-900 mb-4">Getting Started</h2>
          <p className="text-gray-600">
            Configure your data sources and start tracking AI usage. The dashboard will populate with analytics data once requests are being processed.
          </p>
        </div>
      </div>
    </DashboardShell>
  );
}
