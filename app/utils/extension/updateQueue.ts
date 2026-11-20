export const EXTENSION_UPDATE_CONCURRENCY = 3;

type UpdateQueueOptions<T> = {
  concurrency?: number;
  onStart?: (item: T) => void;
};

export type UpdateQueueResult<T> = {
  item: T;
  error?: unknown;
};

export async function runUpdateQueue<T>(
  items: readonly T[],
  run: (item: T) => Promise<void>,
  options: UpdateQueueOptions<T> = {},
): Promise<UpdateQueueResult<T>[]> {
  if (!items.length) return [];

  const concurrency = Math.min(
    Math.max(1, options.concurrency ?? EXTENSION_UPDATE_CONCURRENCY),
    items.length,
  );
  const results = new Array<UpdateQueueResult<T>>(items.length);
  let nextIndex = 0;

  async function worker() {
    while (nextIndex < items.length) {
      const index = nextIndex;
      nextIndex += 1;
      const item = items[index]!;
      options.onStart?.(item);
      try {
        await run(item);
        results[index] = { item };
      } catch (error) {
        results[index] = { item, error };
      }
    }
  }

  await Promise.all(Array.from({ length: concurrency }, () => worker()));
  return results;
}
