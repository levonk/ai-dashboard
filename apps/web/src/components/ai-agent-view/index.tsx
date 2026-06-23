/**
 * AI Agent View Component
 * 
 * Dashboard component optimized for AI agent consumption with markdown output
 * and format selection capabilities
 */

'use client';

import React, { useState, useEffect } from 'react';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '../ui/card';
import { Button } from '../ui/button';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '../ui/select';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '../ui/tabs';
import { Badge } from '../ui/badge';
import { Copy, Download, RefreshCw } from 'lucide-react';

interface AIAgentViewProps {
  data: any[];
  onRefresh?: () => void;
  isLoading?: boolean;
}

type OutputFormat = 'markdown' | 'json' | 'toon' | 'json-single';

export function AIAgentView({ data, onRefresh, isLoading }: AIAgentViewProps) {
  const [format, setFormat] = useState<OutputFormat>('markdown');
  const [output, setOutput] = useState('');
  const [copied, setCopied] = useState(false);

  useEffect(() => {
    generateOutput();
  }, [data, format]);

  const generateOutput = () => {
    switch (format) {
      case 'markdown':
        setOutput(convertToMarkdown(data));
        break;
      case 'json':
        setOutput(JSON.stringify(data, null, 2));
        break;
      case 'toon':
        setOutput(convertToToonFormat(data));
        break;
      case 'json-single':
        setOutput(convertToJSONSingle(data));
        break;
    }
  };

  const convertToMarkdown = (records: any[]): string => {
    if (records.length === 0) return '# No Data Available\n\nNo data to display.';

    let markdown = '# Analytics Data\n\n';
    markdown += `**Record Count:** ${records.length}\n`;
    markdown += `**Generated:** ${new Date().toISOString()}\n\n`;

    // Add summary statistics
    markdown += '## Summary\n\n';
    
    if (records[0].metrics) {
      markdown += '### Metrics\n';
      for (const metric of Object.keys(records[0].metrics)) {
        const values = records.map(r => r.metrics?.[metric]).filter(v => typeof v === 'number');
        if (values.length > 0) {
          const sum = values.reduce((a, b) => a + b, 0);
          const avg = sum / values.length;
          markdown += `- **${metric}:** ${avg.toFixed(2)} (avg), ${sum.toFixed(2)} (total)\n`;
        }
      }
      markdown += '\n';
    }

    // Add data table
    markdown += '## Data\n\n';
    const headers = extractHeaders(records[0]);
    markdown += '| ' + headers.join(' | ') + ' |\n';
    markdown += '| ' + headers.map(() => '---').join(' | ') + ' |\n';

    for (const record of records.slice(0, 50)) {
      const values = headers.map(header => {
        const value = getNestedValue(record, header);
        return formatCellValue(value);
      });
      markdown += '| ' + values.join(' | ') + ' |\n';
    }

    if (records.length > 50) {
      markdown += `\n*... and ${records.length - 50} more records*\n`;
    }

    return markdown;
  };

  const convertToToonFormat = (records: any[]): string => {
    if (records.length === 0) return 'data[]';

    const fields = extractHeaders(records);
    let toon = `data[${records.length}]{${fields.join(',')}:\n`;

    for (const record of records) {
      const values = fields.map(field => {
        const value = getNestedValue(record, field);
        return formatCellValue(value);
      });
      toon += '  ' + values.join(',') + '\n';
    }

    return toon;
  };

  const convertToJSONSingle = (records: any[]): string => {
    return records.map(record => JSON.stringify(record)).join('\n');
  };

  const extractHeaders = (record: any): string[] => {
    const headers: string[] = ['timestamp'];
    
    if (record.dimensions) {
      Object.keys(record.dimensions).forEach(key => {
        headers.push(`dimensions.${key}`);
      });
    }

    if (record.metrics) {
      Object.keys(record.metrics).forEach(key => {
        headers.push(`metrics.${key}`);
      });
    }

    return headers;
  };

  const getNestedValue = (obj: any, path: string): any => {
    const keys = path.split('.');
    let current = obj;
    
    for (const key of keys) {
      if (current === null || current === undefined) return null;
      current = current[key];
    }
    
    return current;
  };

  const formatCellValue = (value: any): string => {
    if (value === null || value === undefined) return 'N/A';
    if (typeof value === 'object') return JSON.stringify(value);
    return String(value);
  };

  const handleCopy = () => {
    navigator.clipboard.writeText(output);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  const handleDownload = () => {
    const extension = format === 'markdown' ? 'md' : format === 'toon' ? 'toon' : 'json';
    const blob = new Blob([output], { type: 'text/plain' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = `analytics-export.${extension}`;
    document.body.appendChild(a);
    a.click();
    document.body.removeChild(a);
    URL.revokeObjectURL(url);
  };

  const getFormatInfo = (fmt: OutputFormat) => {
    switch (fmt) {
      case 'markdown':
        return { description: 'Human-readable with tables', efficiency: 'Medium' };
      case 'json':
        return { description: 'Standard JSON format', efficiency: 'Low' };
      case 'toon':
        return { description: 'Token-efficient for AI', efficiency: 'High' };
      case 'json-single':
        return { description: 'JSONL for streaming', efficiency: 'Medium' };
    }
  };

  const currentFormatInfo = getFormatInfo(format);

  return (
    <Card className="w-full">
      <CardHeader>
        <div className="flex justify-between items-start">
          <div>
            <CardTitle>AI Agent View</CardTitle>
            <CardDescription>
              Optimized output format for AI agent consumption
            </CardDescription>
          </div>
          <div className="flex gap-2">
            <Badge variant="outline">{currentFormatInfo.efficiency} Token Efficiency</Badge>
            <Badge variant="outline">{data.length} Records</Badge>
          </div>
        </div>
      </CardHeader>
      <CardContent>
        <div className="space-y-4">
          {/* Format Selection */}
          <div className="flex items-center gap-4">
            <label className="text-sm font-medium">Output Format:</label>
            <Select value={format} onValueChange={(value) => setFormat(value as OutputFormat)}>
              <SelectTrigger className="w-[200px]">
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="markdown">Markdown</SelectItem>
                <SelectItem value="json">JSON</SelectItem>
                <SelectItem value="toon">ToonFormat</SelectItem>
                <SelectItem value="json-single">JSON Single</SelectItem>
              </SelectContent>
            </Select>
            <div className="text-sm text-muted-foreground">
              {currentFormatInfo.description}
            </div>
          </div>

          {/* Action Buttons */}
          <div className="flex gap-2">
            <Button
              variant="outline"
              size="sm"
              onClick={handleCopy}
              disabled={!output}
            >
              <Copy className="h-4 w-4 mr-2" />
              {copied ? 'Copied!' : 'Copy'}
            </Button>
            <Button
              variant="outline"
              size="sm"
              onClick={handleDownload}
              disabled={!output}
            >
              <Download className="h-4 w-4 mr-2" />
              Download
            </Button>
            {onRefresh && (
              <Button
                variant="outline"
                size="sm"
                onClick={onRefresh}
                disabled={isLoading}
              >
                <RefreshCw className={`h-4 w-4 mr-2 ${isLoading ? 'animate-spin' : ''}`} />
                Refresh
              </Button>
            )}
          </div>

          {/* Output Display */}
          <div className="relative">
            <Tabs defaultValue="output" className="w-full">
              <TabsList className="grid w-full grid-cols-2">
                <TabsTrigger value="output">Output</TabsTrigger>
                <TabsTrigger value="preview">Preview</TabsTrigger>
              </TabsList>
              <TabsContent value="output" className="mt-4">
                <div className="bg-slate-950 text-slate-50 p-4 rounded-md font-mono text-sm overflow-auto max-h-[600px] whitespace-pre-wrap">
                  {output || 'No data to display'}
                </div>
              </TabsContent>
              <TabsContent value="preview" className="mt-4">
                <div className="prose prose-sm max-w-none bg-white p-4 rounded-md border overflow-auto max-h-[600px]">
                  {format === 'markdown' ? (
                    <div dangerouslySetInnerHTML={{ __html: renderMarkdown(output) }} />
                  ) : (
                    <pre className="whitespace-pre-wrap">{output || 'No data to display'}</pre>
                  )}
                </div>
              </TabsContent>
            </Tabs>
          </div>

          {/* Format Information */}
          <div className="text-xs text-muted-foreground">
            <p><strong>Format Information:</strong></p>
            <ul className="list-disc list-inside mt-1 space-y-1">
              <li><strong>Markdown:</strong> Best for human readability and documentation</li>
              <li><strong>JSON:</strong> Standard format for programmatic access</li>
              <li><strong>ToonFormat:</strong> Token-efficient format optimized for LLM consumption</li>
              <li><strong>JSON Single:</strong> JSONL format for streaming and single-record exchanges</li>
            </ul>
          </div>
        </div>
      </CardContent>
    </Card>
  );
}

// Simple markdown renderer (in production, use a proper markdown library)
function renderMarkdown(markdown: string): string {
  // This is a very basic markdown renderer for demonstration
  // In production, use a library like react-markdown
  return markdown
    .replace(/^### (.*$)/gim, '<h3>$1</h3>')
    .replace(/^## (.*$)/gim, '<h2>$1</h2>')
    .replace(/^# (.*$)/gim, '<h1>$1</h1>')
    .replace(/\*\*(.*)\*\*/gim, '<strong>$1</strong>')
    .replace(/\*(.*)\*/gim, '<em>$1</em>')
    .replace(/\n/gim, '<br>');
}
