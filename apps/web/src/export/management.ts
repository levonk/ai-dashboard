/**
 * Export Job Management and Monitoring
 * 
 * Orchestrates export job execution, monitoring, and lifecycle management
 */

import { ExportJob, ExportJobQueue, exportJobQueue } from './jobs';
import { csvExporter } from './formats/csv';
import { jsonExporter } from './formats/json';
import { pdfExporter } from './formats/pdf';
import { exportFilterProcessor } from './filters';
import { QueryResultRow } from '../api/analytics/types';

export interface ExportExecutionContext {
  jobId: string;
  data: QueryResultRow[];
  config: any;
  onProgress?: (progress: number) => void;
}

export interface ExportManagerConfig {
  maxConcurrentJobs: number;
  jobTimeout: number; // milliseconds
  cleanupInterval: number; // milliseconds
  maxJobAge: number; // milliseconds
}

/**
 * Export job manager
 */
export class ExportManager {
  private queue: ExportJobQueue;
  private config: ExportManagerConfig;
  private activeJobs: Map<string, AbortController> = new Map();
  private cleanupTimer: NodeJS.Timeout | null = null;

  constructor(
    queue: ExportJobQueue = exportJobQueue,
    config: Partial<ExportManagerConfig> = {}
  ) {
    this.queue = queue;
    this.config = {
      maxConcurrentJobs: config.maxConcurrentJobs || 1,
      jobTimeout: config.jobTimeout || 30 * 60 * 1000, // 30 minutes
      cleanupInterval: config.cleanupInterval || 5 * 60 * 1000, // 5 minutes
      maxJobAge: config.maxJobAge || 24 * 60 * 60 * 1000 // 24 hours
    };

    this.startCleanupScheduler();
  }

  /**
   * Create and queue a new export job
   */
  async createExportJob(
    data: QueryResultRow[],
    format: 'csv' | 'json' | 'pdf',
    filters: Record<string, any>,
    userId?: string
  ): Promise<ExportJob> {
    const job = await this.queue.createJob(
      { format, filters },
      userId
    );

    // Store data reference (in production, this would be in a database or object storage)
    (job as any)._data = data;

    return job;
  }

  /**
   * Execute an export job
   */
  async executeJob(jobId: string): Promise<ExportJob> {
    const job = this.queue.getJob(jobId);
    if (!job) {
      throw new Error(`Job not found: ${jobId}`);
    }

    if (job.status !== 'pending') {
      throw new Error(`Job is not in pending state: ${job.status}`);
    }

    // Mark as processing
    this.queue.updateJobStatus(jobId, 'processing');

    // Create abort controller for timeout
    const abortController = new AbortController();
    this.activeJobs.set(jobId, abortController);

    // Set timeout
    const timeout = setTimeout(() => {
      abortController.abort();
      this.queue.failJob(jobId, {
        message: 'Job timed out',
        code: 'TIMEOUT'
      });
    }, this.config.jobTimeout);

    try {
      // Get data
      const data = (job as any)._data || [];
      
      // Apply filters if provided
      let filteredData = data;
      if (job.filters && Object.keys(job.filters).length > 0) {
        const filterConfig = {
          filters: this.convertToExportFilters(job.filters),
          sorts: [],
          columns: exportFilterProcessor.createDefaultColumns(data)
        };
        filteredData = exportFilterProcessor.applyConfig(data, filterConfig);
      }

      // Update total records
      this.queue.updateJobProgress(jobId, 0, filteredData.length);

      // Execute export based on format
      let result;
      switch (job.format) {
        case 'csv':
          result = await csvExporter.export(filteredData);
          break;
        case 'json':
          result = await jsonExporter.export(filteredData);
          break;
        case 'pdf':
          result = await pdfExporter.export(filteredData);
          break;
        default:
          throw new Error(`Unsupported format: ${job.format}`);
      }

      // Update progress
      this.queue.updateJobProgress(jobId, filteredData.length, filteredData.length);

      // Complete job
      const completedJob = this.queue.completeJob(jobId, {
        filename: result.filename,
        size: result.size,
        downloadUrl: `/api/export/download/${jobId}`,
        expiresAt: new Date(Date.now() + this.config.maxJobAge).toISOString()
      });

      clearTimeout(timeout);
      return completedJob;

    } catch (error) {
      clearTimeout(timeout);
      
      const errorMessage = error instanceof Error ? error.message : 'Unknown error';
      this.queue.failJob(jobId, {
        message: errorMessage,
        code: 'EXECUTION_ERROR',
        details: error
      });

      throw error;
    } finally {
      this.activeJobs.delete(jobId);
    }
  }

  /**
   * Cancel a running job
   */
  cancelJob(jobId: string): boolean {
    const abortController = this.activeJobs.get(jobId);
    if (abortController) {
      abortController.abort();
      this.activeJobs.delete(jobId);
    }

    const cancelled = this.queue.cancelJob(jobId);
    return cancelled !== undefined;
  }

  /**
   * Get job status with detailed information
   */
  getJobStatus(jobId: string): {
    job: ExportJob | undefined;
    isActive: boolean;
    canCancel: boolean;
  } {
    const job = this.queue.getJob(jobId);
    const isActive = this.activeJobs.has(jobId);
    const canCancel = job && (job.status === 'pending' || job.status === 'processing');

    return {
      job,
      isActive,
      canCancel
    };
  }

  /**
   * Get all jobs for a user with status
   */
  getUserJobsWithStatus(userId: string): Array<{
    job: ExportJob;
    isActive: boolean;
    canCancel: boolean;
  }> {
    const jobs = this.queue.getUserJobs(userId);

    return jobs.map(job => ({
      job,
      isActive: this.activeJobs.has(job.id),
      canCancel: job.status === 'pending' || job.status === 'processing'
    }));
  }

  /**
   * Get queue statistics
   */
  getQueueStats() {
    return {
      ...this.queue.getStats(),
      activeJobs: this.activeJobs.size,
      maxConcurrentJobs: this.config.maxConcurrentJobs
    };
  }

  /**
   * Process next pending job
   */
  async processNextJob(): Promise<ExportJob | null> {
    // Check if we can start a new job
    if (this.activeJobs.size >= this.config.maxConcurrentJobs) {
      return null;
    }

    const nextJob = this.queue.getNextPendingJob();
    if (!nextJob) {
      return null;
    }

    try {
      return await this.executeJob(nextJob.id);
    } catch (error) {
      console.error(`Failed to execute job ${nextJob.id}:`, error);
      return null;
    }
  }

  /**
   * Start automatic job processing
   */
  startAutoProcessing(interval: number = 1000): void {
    setInterval(() => {
      this.processNextJob().catch(error => {
        console.error('Auto-processing error:', error);
      });
    }, interval);
  }

  /**
   * Start cleanup scheduler
   */
  private startCleanupScheduler(): void {
    this.cleanupTimer = setInterval(() => {
      const cleaned = this.queue.cleanupOldJobs(this.config.maxJobAge);
      if (cleaned > 0) {
        console.log(`Cleaned up ${cleaned} old export jobs`);
      }
    }, this.config.cleanupInterval);
  }

  /**
   * Stop cleanup scheduler
   */
  stopCleanupScheduler(): void {
    if (this.cleanupTimer) {
      clearInterval(this.cleanupTimer);
      this.cleanupTimer = null;
    }
  }

  /**
   * Shutdown manager
   */
  shutdown(): void {
    // Cancel all active jobs
    for (const [jobId, controller] of this.activeJobs.entries()) {
      controller.abort();
      this.queue.cancelJob(jobId);
    }
    this.activeJobs.clear();

    // Stop cleanup scheduler
    this.stopCleanupScheduler();
  }

  /**
   * Convert simple filter object to ExportFilter array
   */
  private convertToExportFilters(filters: Record<string, any>): any[] {
    return Object.entries(filters).map(([field, value]) => ({
      field,
      operator: 'eq',
      value
    }));
  }
}

// Global export manager instance
export const exportManager = new ExportManager();