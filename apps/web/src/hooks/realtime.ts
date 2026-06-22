import { useState, useEffect, useCallback } from "react";

interface RealtimeDataOptions {
  enabled?: boolean;
  interval?: number;
  onData?: (data: any) => void;
  onError?: (error: Error) => void;
}

export function useRealtimeData<T>(
  fetchFn: () => Promise<T>,
  options: RealtimeDataOptions = {}
) {
  const [data, setData] = useState<T | null>(null);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<Error | null>(null);
  const [lastUpdate, setLastUpdate] = useState<Date | null>(null);

  const {
    enabled = true,
    interval = 5000, // 5 seconds default
    onData,
    onError,
  } = options;

  const fetchData = useCallback(async () => {
    if (!enabled) return;

    setIsLoading(true);
    setError(null);

    try {
      const result = await fetchFn();
      setData(result);
      setLastUpdate(new Date());
      onData?.(result);
    } catch (err) {
      const error = err instanceof Error ? err : new Error("Failed to fetch data");
      setError(error);
      onError?.(error);
    } finally {
      setIsLoading(false);
    }
  }, [fetchFn, enabled, onData, onError]);

  useEffect(() => {
    if (!enabled) return;

    // Initial fetch
    fetchData();

    // Set up interval
    const intervalId = setInterval(fetchData, interval);

    return () => clearInterval(intervalId);
  }, [fetchData, interval, enabled]);

  const refresh = useCallback(() => {
    fetchData();
  }, [fetchData]);

  return {
    data,
    isLoading,
    error,
    lastUpdate,
    refresh,
  };
}

export function useRealtimeWebSocket<T>(
  url: string,
  options: {
    enabled?: boolean;
    onMessage?: (data: T) => void;
    onError?: (error: Error) => void;
    onConnect?: () => void;
    onDisconnect?: () => void;
  } = {}
) {
  const [data, setData] = useState<T | null>(null);
  const [isConnected, setIsConnected] = useState(false);
  const [error, setError] = useState<Error | null>(null);

  const {
    enabled = true,
    onMessage,
    onError,
    onConnect,
    onDisconnect,
  } = options;

  useEffect(() => {
    if (!enabled) return;

    let ws: WebSocket | null = null;
    let reconnectTimeout: NodeJS.Timeout | null = null;

    const connect = () => {
      try {
        ws = new WebSocket(url);

        ws.onopen = () => {
          setIsConnected(true);
          setError(null);
          onConnect?.();
        };

        ws.onmessage = (event) => {
          try {
            const parsedData = JSON.parse(event.data) as T;
            setData(parsedData);
            onMessage?.(parsedData);
          } catch (err) {
            const error = err instanceof Error ? err : new Error("Failed to parse message");
            setError(error);
            onError?.(error);
          }
        };

        ws.onerror = () => {
          const error = new Error("WebSocket error");
          setError(error);
          onError?.(error);
        };

        ws.onclose = () => {
          setIsConnected(false);
          onDisconnect?.();

          // Attempt to reconnect after 5 seconds
          reconnectTimeout = setTimeout(connect, 5000);
        };
      } catch (err) {
        const error = err instanceof Error ? err : new Error("Failed to create WebSocket");
        setError(error);
        onError?.(error);
      }
    };

    connect();

    return () => {
      if (reconnectTimeout) {
        clearTimeout(reconnectTimeout);
      }
      if (ws) {
        ws.close();
      }
    };
  }, [url, enabled, onMessage, onError, onConnect, onDisconnect]);

  return {
    data,
    isConnected,
    error,
  };
}
