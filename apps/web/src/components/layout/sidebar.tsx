import Link from "next/link";
import { BarChart3, Settings, Users, Database, Zap } from "lucide-react";

const navigation = [
  { name: "Dashboard", href: "/", icon: BarChart3 },
  { name: "Analytics", href: "/analytics", icon: Zap },
  { name: "Data Sources", href: "/data-sources", icon: Database },
  { name: "Teams", href: "/teams", icon: Users },
  { name: "Settings", href: "/settings", icon: Settings },
];

export function Sidebar() {
  return (
    <div className="w-64 bg-white border-r border-gray-200 flex flex-col">
      <div className="p-6 border-b border-gray-200">
        <h1 className="text-xl font-bold text-gray-900">AI Analytics</h1>
        <p className="text-sm text-gray-500 mt-1">Dashboard</p>
      </div>
      <nav className="flex-1 p-4 space-y-1">
        {navigation.map((item) => (
          <Link
            key={item.name}
            href={item.href}
            className="flex items-center px-3 py-2 text-sm font-medium rounded-md text-gray-700 hover:bg-gray-100 hover:text-gray-900 transition-colors"
          >
            <item.icon className="w-5 h-5 mr-3 text-gray-400" />
            {item.name}
          </Link>
        ))}
      </nav>
      <div className="p-4 border-t border-gray-200">
        <div className="text-xs text-gray-500">
          <p>Version 0.0.1</p>
          <p className="mt-1">Open Source</p>
        </div>
      </div>
    </div>
  );
}
