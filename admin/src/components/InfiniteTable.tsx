import { Button, Spin, Table, Typography } from "antd";
import type { TableProps } from "antd";
import { useEffect, useRef } from "react";
import type { UIEvent } from "react";

const LOAD_MORE_THRESHOLD = 240;

interface InfiniteTableProps<T extends object>
  extends Omit<TableProps<T>, "dataSource" | "pagination"> {
  items: T[];
  hasMore: boolean;
  isFetchingNextPage?: boolean;
  itemLabel: string;
  onLoadMore: () => Promise<unknown> | void;
}

export function InfiniteTable<T extends object>({
  items,
  hasMore,
  isFetchingNextPage = false,
  itemLabel,
  onLoadMore,
  ...tableProps
}: InfiniteTableProps<T>) {
  const loadMorePendingRef = useRef(false);

  useEffect(() => {
    if (!isFetchingNextPage) {
      loadMorePendingRef.current = false;
    }
  }, [isFetchingNextPage]);

  const requestLoadMore = () => {
    if (!hasMore || isFetchingNextPage || loadMorePendingRef.current) {
      return;
    }

    loadMorePendingRef.current = true;
    void Promise.resolve(onLoadMore())
      .catch(() => undefined)
      .finally(() => {
        loadMorePendingRef.current = false;
      });
  };

  const handleScroll = (event: UIEvent<HTMLDivElement>) => {
    const target = event.currentTarget;
    const distanceToBottom = target.scrollHeight - target.scrollTop - target.clientHeight;

    if (
      distanceToBottom > LOAD_MORE_THRESHOLD ||
      !hasMore ||
      isFetchingNextPage ||
      loadMorePendingRef.current
    ) {
      return;
    }

    requestLoadMore();
  };

  return (
    <div className="infinite-table-shell" onScroll={handleScroll}>
      <Table<T>
        {...tableProps}
        dataSource={items}
        pagination={false}
      />
      {isFetchingNextPage ? (
        <div className="infinite-table-load-more">
          <Spin size="small" />
          <Typography.Text type="secondary">正在加载更多{itemLabel}</Typography.Text>
        </div>
      ) : hasMore ? (
        <div className="infinite-table-load-more">
          <Button type="link" disabled={loadMorePendingRef.current} onClick={requestLoadMore}>
            加载更多
          </Button>
        </div>
      ) : items.length ? (
        <div className="infinite-table-load-more">
          <Typography.Text type="secondary">
            已加载全部 {items.length} 条{itemLabel}
          </Typography.Text>
        </div>
      ) : null}
    </div>
  );
}
