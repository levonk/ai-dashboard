import { useState } from "react";
import { Button } from "../ui/button";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
} from "../ui/dialog";
import { ChevronRight, ChevronLeft } from "lucide-react";

interface DrillDownLevel {
  id: string;
  title: string;
  data: any[];
  columns: Array<{
    key: string;
    label: string;
  }>;
}

interface DrillDownModalProps {
  isOpen: boolean;
  onClose: () => void;
  levels: DrillDownLevel[];
  currentLevel: number;
  onLevelChange: (level: number) => void;
}

export function DrillDownModal({
  isOpen,
  onClose,
  levels,
  currentLevel,
  onLevelChange,
}: DrillDownModalProps) {
  const currentData = levels[currentLevel];

  const handleRowClick = (row: any) => {
    // In a real implementation, this would navigate to the next level
    // or show detailed information about the selected row
    console.log("Row clicked:", row);
  };

  return (
    <Dialog open={isOpen} onOpenChange={onClose}>
      <DialogContent className="max-w-4xl max-h-[80vh] overflow-auto">
        <DialogHeader>
          <DialogTitle className="flex items-center justify-between">
            <span>{currentData.title}</span>
            <div className="flex items-center space-x-2">
              {currentLevel > 0 && (
                <Button
                  variant="ghost"
                  size="sm"
                  onClick={() => onLevelChange(currentLevel - 1)}
                >
                  <ChevronLeft className="w-4 h-4 mr-1" />
                  Back
                </Button>
              )}
              {currentLevel < levels.length - 1 && (
                <Button
                  variant="ghost"
                  size="sm"
                  onClick={() => onLevelChange(currentLevel + 1)}
                >
                  Next
                  <ChevronRight className="w-4 h-4 ml-1" />
                </Button>
              )}
            </div>
          </DialogTitle>
        </DialogHeader>

        <div className="mt-4">
          <table className="w-full">
            <thead>
              <tr className="border-b border-gray-200">
                {currentData.columns.map((column) => (
                  <th
                    key={column.key}
                    className="text-left py-2 px-4 font-medium text-gray-700"
                  >
                    {column.label}
                  </th>
                ))}
              </tr>
            </thead>
            <tbody>
              {currentData.data.map((row, index) => (
                <tr
                  key={index}
                  className="border-b border-gray-100 hover:bg-gray-50 cursor-pointer"
                  onClick={() => handleRowClick(row)}
                >
                  {currentData.columns.map((column) => (
                    <td key={column.key} className="py-2 px-4 text-sm text-gray-600">
                      {row[column.key]}
                    </td>
                  ))}
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      </DialogContent>
    </Dialog>
  );
}
