import { useState } from "react";
import { Button } from "../ui/button";
import { Input } from "../ui/input";
import { Dialog, DialogContent, DialogHeader, DialogTitle } from "../ui/dialog";
import { Save, Trash2, Plus } from "lucide-react";

interface DashboardConfig {
  id: string;
  name: string;
  layout: {
    metrics: string[];
    charts: string[];
    filters: string[];
  };
}

interface DashboardConfigPanelProps {
  isOpen: boolean;
  onClose: () => void;
  configs: DashboardConfig[];
  onSave: (config: DashboardConfig) => void;
  onLoad: (configId: string) => void;
  onDelete: (configId: string) => void;
}

export function DashboardConfigPanel({
  isOpen,
  onClose,
  configs,
  onSave,
  onLoad,
  onDelete,
}: DashboardConfigPanelProps) {
  const [newConfigName, setNewConfigName] = useState("");

  const handleSave = () => {
    if (!newConfigName.trim()) return;

    const newConfig: DashboardConfig = {
      id: Date.now().toString(),
      name: newConfigName,
      layout: {
        metrics: ["requests", "cost", "models", "teams"],
        charts: ["time-series", "bar-chart"],
        filters: ["client", "model", "supplier", "team", "input-type"],
      },
    };

    onSave(newConfig);
    setNewConfigName("");
  };

  return (
    <Dialog open={isOpen} onOpenChange={onClose}>
      <DialogContent className="max-w-2xl">
        <DialogHeader>
          <DialogTitle>Dashboard Configurations</DialogTitle>
        </DialogHeader>

        <div className="space-y-4 mt-4">
          <div className="flex space-x-2">
            <Input
              placeholder="New configuration name..."
              value={newConfigName}
              onChange={(e) => setNewConfigName(e.target.value)}
              onKeyPress={(e) => e.key === "Enter" && handleSave()}
            />
            <Button onClick={handleSave}>
              <Plus className="w-4 h-4 mr-2" />
              Save
            </Button>
          </div>

          <div className="space-y-2">
            {configs.map((config) => (
              <div
                key={config.id}
                className="flex items-center justify-between p-3 border border-gray-200 rounded-lg"
              >
                <div>
                  <div className="font-medium text-gray-900">{config.name}</div>
                  <div className="text-sm text-gray-500">
                    {config.layout.metrics.length} metrics, {config.layout.charts.length} charts
                  </div>
                </div>
                <div className="flex space-x-2">
                  <Button
                    variant="outline"
                    size="sm"
                    onClick={() => onLoad(config.id)}
                  >
                    Load
                  </Button>
                  <Button
                    variant="ghost"
                    size="sm"
                    onClick={() => onDelete(config.id)}
                  >
                    <Trash2 className="w-4 h-4" />
                  </Button>
                </div>
              </div>
            ))}
          </div>
        </div>
      </DialogContent>
    </Dialog>
  );
}
