import type { QueryClient, QueryKey } from "@tanstack/react-query";

export const INFINITE_TABLE_QUERY_OPTIONS = {
  staleTime: 5 * 60_000,
  gcTime: 10 * 60_000,
  refetchOnMount: false,
  refetchOnWindowFocus: false
} as const;

interface InfiniteQueryCacheData {
  pages: unknown[];
  pageParams: unknown[];
  [key: string]: unknown;
}

export function trimInfiniteQueryCache(
  queryClient: QueryClient,
  queryKey: QueryKey,
  keepPageCount = 1
) {
  queryClient.setQueriesData<unknown>({ queryKey, exact: false }, (data: unknown) => {
    if (!isInfiniteQueryCacheData(data) || data.pages.length <= keepPageCount) {
      return data;
    }

    return {
      ...data,
      pages: data.pages.slice(0, keepPageCount),
      pageParams: data.pageParams.slice(0, keepPageCount)
    };
  });
}

function isInfiniteQueryCacheData(data: unknown): data is InfiniteQueryCacheData {
  if (!data || typeof data !== "object") {
    return false;
  }

  const candidate = data as Partial<InfiniteQueryCacheData>;

  return Array.isArray(candidate.pages) && Array.isArray(candidate.pageParams);
}
