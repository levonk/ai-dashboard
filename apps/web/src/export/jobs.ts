/**
 * Export Job System
 * 
 * Manages export job queue, execution, and lifecycle
 */

export interface ExportJob {
  id: string;
  status: 'pending' | 'processing' | 'completed' | 'failed' | 'cancelled';
  format: 'csv' | 'json' | 'pdf';
  filters: Record<string, any>;
  userId?: string;
  createdAt: string;
  startedAt?: string;
  completedAt?: string;
  progress: number;
  totalRecords: number;
  processedRecords: number;
  result?: {
    filename: string;
    size: number;
    downloadUrl: string;
    expiresAt: string;
  };
  error?: {
    message: string;
    code: string;
    details?: any;
  };
  delivery?: {
    method: 'download' | 'email' | 's3';
    destination?: string;
    status: 'pending' | 'delivered' | 'failed';
  };
}

export interface ExportJobConfig {
  format: 'csv' | 'json' | 'pdf';
  filters: Record<string, any>;
  delivery?: {
    method: 'download' | 'email' | 's3';
    destination?: string;
  };
  priority?: 'low' | 'normal' | 'high';
  expiresIn?: number; // hours
}

/**
 * Export job queue manager
 */
export class ExportJobQueue {
  private jobs: Map<string, ExportJob> = new Map();
  private processingJob: string | null = null;
  private maxConcurrentJobs = 1; // Single-threaded for now
  private jobTimeout = 30 * 60 * 1000; // 30 minutes

  /**
   * Create a new export job
   */
  async createJob(config: ExportJobConfig, userId?: string): Promise<ExportJob> {
    const jobId = this.generateJobId();
    const job: ExportJob = {
      id: jobId,
      status: 'pending',
      format: config.format,
      filters: config.filters,
      userId,
      createdAt: new Date().toISOString(),
      progress: 0,
      totalRecords: 0,
      processedRecords: 0,
      delivery: config.delivery ? {
        method: config.delivery.method,
        destination: config.delivery.destination,
        status: 'pending'
      } : undefined
    };

    this.jobs.set(jobId, job);
    return job;
  }

  /**
   * Get job by ID
   */
  getJob(jobId: string): ExportJob | undefined {
    return this.jobs.get(jobId);
  }

  /**
   * Get all jobs for a user
   */
  getUserJobs(userId: string): ExportJob[] {
    return Array.from(this.jobs.values())
      .filter(job => job.userId === userId)
      .sort((a, b) => new Date(b.createdAt).getTime() - new Date(a.createdAt).getTime());
  }

  /**
   * Get next pending job
   */
  getNextPendingJob(): ExportJob | undefined {
    const pendingJobs = Array.from(this.jobs.values())
      .filter(job => job.status === 'pending')
      .sort((a, b) => new Date(a.createdAt).getTime() - new Date(b.createdAt).getTime());

    return pendingJobs[0];
  }

  /**
   * Update job status
   */
  updateJobStatus(
    jobId: string,
    status: ExportJob['status'],
    updates?: Partial<ExportJob>
  ): ExportJob | undefined {
    const job = this.jobs.get(jobId);
    if (!job) return undefined;

    job.status = status;
    
    if (status === 'processing' && !job.startedAt) {
      job.startedAt = new Date().toISOString();
    }
    
    if (status === 'completed' || status === 'failed') {
      job.completedAt = new Date().toISOString();
    }

    if (updates) {
      Object.assign(job, updates);
    }

    this.jobs.set(jobId, job);
    return job;
  }

  /**
   * Update job progress
   */
  updateJobProgress(
    jobId: string,
    processedRecords: number,
    totalRecords: number
  ): ExportJob | undefined {
    const job = this.jobs.get(jobId);
    if (!job) return undefined;

    job.processedRecords = processedRecords;
    job.totalRecords = totalRecords;
    job.progress = totalRecords > 0 ? (processedRecords / totalRecords) * 100 : 0;

    this.jobs.set(jobId, job);
    return job;
  }

  /**
   * Mark job as completed with result
   */
  completeJob(
    jobId: string,
    result: ExportJob['result']
  ): ExportJob | undefined {
    return this.updateJobStatus(jobId, 'completed', { 
      result,
      progress: 100 
    });
  }

  /**
   * Mark job as failed with error
   */
  failJob(
    jobId: string,
    error: ExportJob['error']
  ): ExportJob | undefined {
    return this.updateJobStatus(jobId, 'failed', { error });
  }

  /**
   * Cancel a job
   */
  cancelJob(jobId: string): ExportJob | undefined {
    const job = this.jobs.get(jobId);
    if (!job || job.status === 'completed' || job.status === 'failed') {
      return undefined;
    }

    return this.updateJobStatus(jobId, 'cancelled');
  }

  /**
   * Clean up old jobs
   */
  cleanupOldJobs(maxAge: number = 24 * 60 * 60 * 1000): number {
    const now = Date.now();
    let cleaned = 0;

    for (const [jobId, job] of this.jobs.entries()) {
      const jobAge = now - new Date(job.createdAt).getTime();
      
      // Remove completed/failed/cancelled jobs older than maxAge
      if ((job.status === 'completed' || job.status === 'failed' || job.status === 'cancelled') && 
          jobAge > maxAge) {
        this.jobs.delete(jobId);
        cleaned++;
      }
    }

    return cleaned;
  }

  /**
   * Get queue statistics
   */
  getStats() {
    const jobs = Array.from(this.jobs.values());
    
    return {
      total: jobs.length,
      pending: jobs.filter(j => j.status === 'pending').length,
      processing: jobs.filter(j => j.status === 'processing').length,
      completed: jobs.filter(j => j.status === 'completed').length,
      failed: jobs.filter(j => j.status === 'failed').length,
      cancelled: jobs.filter(j => j.status === 'cancelled').length
    };
  }

  /**
   * Generate unique job ID
   */
  private generateJobId(): string {
    return `export-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`;
  }
}

// Global job queue instance
export const exportJobQueue = new ExportJobQueue();