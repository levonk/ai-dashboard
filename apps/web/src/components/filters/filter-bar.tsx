import { useState } from "react";
import { Filter, X } from "lucide-react";
import { Button } from "../ui/button";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "../ui/select";

interface FilterOption {
  value: string;
  label: string;
}

interface FilterConfig {
  id: string;
  label: string;
  options: FilterOption[];
  value: string;
}

export function FilterBar() {
  const [isOpen, setIsOpen] = useState(false);
  const [filters, setFilters] = useState<FilterConfig[]>([
    {
      id: "client",
      label: "Client",
      options: [
        { value: "all", label: "All Clients" },
        { value: "claude-code", label: "Claude Code" },
        { value: "codex", label: "Codex" },
        { value: "pi", label: "Pi" },
        { value: "devin", label: "Devin" },
      ],
      value: "all",
    },
    {
      id: "model",
      label: "Model",
      options: [
        { value: "all", label: "All Models" },
        { value: "claude-3-5-sonnet", label: "Claude 3.5 Sonnet" },
        { value: "claude-3-opus", label: "Claude 3 Opus" },
        { value: "gpt-4", label: "GPT-4" },
        { value: "gpt-3.5-turbo", label: "GPT-3.5 Turbo" },
      ],
      value: "all",
    },
    {
      id: "supplier",
      label: "Supplier",
      options: [
        { value: "all", label: "All Suppliers" },
        { value: "anthropic", label: "Anthropic" },
        { value: "openai", label: "OpenAI" },
        { value: "google", label: "Google" },
        { value: "microsoft", label: "Microsoft" },
      ],
      value: "all",
    },
    {
      id: "team",
      label: "Team",
      options: [
        { value: "all", label: "All Teams" },
        { value: "engineering", label: "Engineering" },
        { value: "product", label: "Product" },
        { value: "design", label: "Design" },
      ],
      value: "all",
    },
    {
      id: "input-type",
      label: "Input Type",
      options: [
        { value: "all", label: "All Types" },
        { value: "text", label: "Text/Chat" },
        { value: "image", label: "Image" },
        { value: "audio", label: "Audio" },
      ],
      value: "all",
    },
  ]);

  const activeFilters = filters.filter(f => f.value !== "all");

  const updateFilter = (id: string, value: string) => {
    setFilters(filters.map(f => f.id === id ? { ...f, value } : f));
  };

  const clearFilter = (id: string) => {
    updateFilter(id, "all");
  };

  const clearAll = () => {
    setFilters(filters.map(f => ({ ...f, value: "all" })));
  };

  return (
    <div className="space-y-4">
      <div className="flex items-center justify-between">
        <div className="flex items-center space-x-2">
          <Button
            variant="outline"
            size="sm"
            onClick={() => setIsOpen(!isOpen)}
          >
            <Filter className="w-4 h-4 mr-2" />
            Filters
            {activeFilters.length > 0 && (
              <span className="ml-2 bg-blue-100 text-blue-800 text-xs px-2 py-0.5 rounded-full">
                {activeFilters.length}
              </span>
            )}
          </Button>
          {activeFilters.length > 0 && (
            <Button variant="ghost" size="sm" onClick={clearAll}>
              Clear all
            </Button>
          )}
        </div>
      </div>

      {isOpen && (
        <div className="bg-white border border-gray-200 rounded-lg p-4 space-y-4">
          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
            {filters.map((filter) => (
              <div key={filter.id}>
                <label className="block text-sm font-medium text-gray-700 mb-1">
                  {filter.label}
                </label>
                <Select
                  value={filter.value}
                  onValueChange={(value) => updateFilter(filter.id, value)}
                >
                  <SelectTrigger>
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    {filter.options.map((option) => (
                      <SelectItem key={option.value} value={option.value}>
                        {option.label}
                      </SelectItem>
                    ))}
                  </SelectContent>
                </Select>
              </div>
            ))}
          </div>
        </div>
      )}

      {activeFilters.length > 0 && (
        <div className="flex flex-wrap gap-2">
          {activeFilters.map((filter) => (
            <div
              key={filter.id}
              className="inline-flex items-center bg-gray-100 rounded-md px-3 py-1 text-sm"
            >
              <span className="font-medium text-gray-700">{filter.label}:</span>
              <span className="ml-1 text-gray-600">
                {filter.options.find(o => o.value === filter.value)?.label}
              </span>
              <button
                onClick={() => clearFilter(filter.id)}
                className="ml-2 text-gray-400 hover:text-gray-600"
              >
                <X className="w-3 h-3" />
              </button>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}
