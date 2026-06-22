/**
 * Export Domain Routes
 * 
 * Route handlers for data export operations
 */

import { NextRequest, NextResponse } from 'next/server';
import { exportManager } from '../../export/management';
import { exportJobQueue } from '../../export/jobs';
import { dataSanitizer } from '../../export/privacy';
import { QueryResultRow } from '../analytics/types';

/**
 * Execute data export
 */
export async function execute(request: NextRequest): Promise<NextResponse> {
  try {
    const body = await request.json();
    const { format, filters, data, privacy } = body;

    // Validate request
    if (!format || !['csv', 'json', 'pdf'].includes(format)) {
      return NextResponse.json(
        { error: 'Invalid format. Must be csv, json, or pdf' },
        { status: 400 }
      );
    }

    if (!data || !Array.isArray(data)) {
      return NextResponse.json(
        { error: 'Data is required and must be an array' },
        { status: 400 }
      );
    }

    // Apply privacy sanitization if requested
    let processedData = data as QueryResultRow[];
    if (privacy?.enabled) {
      processedData = dataSanitizer.sanitize(processedData, privacy);
    }

    // Create export job
    const job = await exportManager.createExportJob(
      processedData,
      format,
      filters || {}
    );

    // Execute job asynchronously
    exportManager.executeJob(job.id).catch(error => {
      console.error(`Export job ${job.id} failed:`, error);
    });

    return NextResponse.json({
      exportId: job.id,
      status: job.status,
      format: job.format,
      createdAt: job.createdAt,
      estimatedCompletion: new Date(Date.now() + 30000).toISOString()
    });
  } catch (error) {
    return NextResponse.json(
      { error: 'Export execution failed', message: error instanceof Error ? error.message : 'Unknown error' },
      { status: 500 }
    );
  }
}

/**
 * Get available export formats
 */
export async function getFormats(request: NextRequest): Promise<NextResponse> {
  return NextResponse.json({
    formats: [
      {
        name: 'json',
        description: 'JSON format with schema validation',
        mimeType: 'application/json',
        extension: '.json',
        features: ['schema_validation', 'metadata', 'pretty_print']
      },
      {
        name: 'csv',
        description: 'CSV format with Excel compatibility',
        mimeType: 'text/csv',
        extension: '.csv',
        features: ['excel_compatible', 'custom_delimiter', 'header_options']
      },
      {
        name: 'pdf',
        description: 'PDF report with summary statistics',
        mimeType: 'application/pdf',
        extension: '.pdf',
        features: ['summary_statistics', 'data_tables', 'formatted_report']
      }
    ]
  });
}

/**
 * Get export job status
 */
export async function getStatus(request: NextRequest): Promise<NextResponse> {
  try {
    const { pathname } = new URL(request.url);
    const id = pathname.split('/').pop();
    
    if (!id) {
      return NextResponse.json(
        { error: 'Export ID is required' },
        { status: 400 }
      );
    }

    const jobStatus = exportManager.getJobStatus(id);
    
    if (!jobStatus.job) {
      return NextResponse.json(
        { error: 'Export job not found' },
        { status: 404 }
      );
    }

    return NextResponse.json({
      exportId: jobStatus.job.id,
      status: jobStatus.job.status,
      progress: jobStatus.job.progress,
      totalRecords: jobStatus.job.totalRecords,
      processedRecords: jobStatus.job.processedRecords,
      createdAt: jobStatus.job.createdAt,
      startedAt: jobStatus.job.startedAt,
      completedAt: jobStatus.job.completedAt,
      result: jobStatus.job.result,
      error: jobStatus.job.error,
      isActive: jobStatus.isActive,
      canCancel: jobStatus.canCancel
    });
  } catch (error) {
    return NextResponse.json(
      { error: 'Failed to get export status', message: error instanceof Error ? error.message : 'Unknown error' },
      { status: 500 }
    );
  }
}

/**
 * Cancel export job
 */
export async function cancel(request: NextRequest): Promise<NextResponse> {
  try {
    const { pathname } = new URL(request.url);
    const id = pathname.split('/').pop();
    
    if (!id) {
      return NextResponse.json(
        { error: 'Export ID is required' },
        { status: 400 }
      );
    }

    const cancelled = exportManager.cancelJob(id);
    
    if (!cancelled) {
      return NextResponse.json(
        { error: 'Unable to cancel job - job may not exist or cannot be cancelled' },
        { status: 400 }
      );
    }

    return NextResponse.json({
      exportId: id,
      status: 'cancelled',
      message: 'Export job cancelled successfully'
    });
  } catch (error) {
    return NextResponse.json(
      { error: 'Failed to cancel export', message: error instanceof Error ? error.message : 'Unknown error' },
      { status: 500 }
    );
  }
}

/**
 * Download exported data
 */
export async function download(request: NextRequest): Promise<NextResponse> {
  try {
    const { pathname } = new URL(request.url);
    const id = pathname.split('/').pop();
    
    if (!id) {
      return NextResponse.json(
        { error: 'Export ID is required' },
        { status: 400 }
      );
    }

    const job = exportJobQueue.getJob(id);
    
    if (!job) {
      return NextResponse.json(
        { error: 'Export job not found' },
        { status: 404 }
      );
    }

    if (job.status !== 'completed') {
      return NextResponse.json(
        { error: 'Export is not ready for download', status: job.status },
        { status: 400 }
      );
    }

    if (!job.result) {
      return NextResponse.json(
        { error: 'Export result not available' },
        { status: 500 }
      );
    }

    // In production, this would serve the actual file
    // For now, return a placeholder response
    return NextResponse.json({
      message: 'File download would be served here',
      filename: job.result.filename,
      size: job.result.size,
      downloadUrl: job.result.downloadUrl,
      expiresAt: job.result.expiresAt
    });
  } catch (error) {
    return NextResponse.json(
      { error: 'Download failed', message: error instanceof Error ? error.message : 'Unknown error' },
      { status: 500 }
    );
  }
}

/**
 * Get user's export jobs
 */
export async function getUserJobs(request: NextRequest): Promise<NextResponse> {
  try {
    const { searchParams } = new URL(request.url);
    const userId = searchParams.get('userId');
    
    if (!userId) {
      return NextResponse.json(
        { error: 'User ID is required' },
        { status: 400 }
      );
    }

    const jobs = exportManager.getUserJobsWithStatus(userId);

    return NextResponse.json({
      jobs: jobs.map(j => ({
        ...j.job,
        isActive: j.isActive,
        canCancel: j.canCancel
      })),
      total: jobs.length
    });
  } catch (error) {
    return NextResponse.json(
      { error: 'Failed to get user jobs', message: error instanceof Error ? error.message : 'Unknown error' },
      { status: 500 }
    );
  }
}

/**
 * Get queue statistics
 */
export async function getStats(request: NextRequest): Promise<NextResponse> {
  try {
    const stats = exportManager.getQueueStats();

    return NextResponse.json(stats);
  } catch (error) {
    return NextResponse.json(
      { error: 'Failed to get queue stats', message: error instanceof Error ? error.message : 'Unknown error' },
      { status: 500 }
    );
  }
}
