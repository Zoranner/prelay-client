import { expect, test } from "bun:test";

import { runUpdateQueue } from "../app/utils/extension/updateQueue";

const tick = () => new Promise((resolve) => setTimeout(resolve, 0));

test("批量更新同时最多运行三个，其余任务等待空位", async () => {
  const started: number[] = [];
  const releases = new Map<number, () => void>();
  let running = 0;
  let peak = 0;

  const queue = runUpdateQueue(
    [1, 2, 3, 4, 5, 6],
    async (item) => {
      running += 1;
      peak = Math.max(peak, running);
      await new Promise<void>((resolve) => releases.set(item, resolve));
      running -= 1;
    },
    {
      concurrency: 3,
      onStart: (item) => started.push(item),
    },
  );

  expect(started).toEqual([1, 2, 3]);
  expect(peak).toBe(3);

  releases.get(1)?.();
  await tick();
  expect(started).toEqual([1, 2, 3, 4]);
  expect(peak).toBe(3);

  for (const item of [2, 3, 4, 5, 6]) {
    releases.get(item)?.();
    await tick();
  }

  await queue;
  expect(started).toEqual([1, 2, 3, 4, 5, 6]);
  expect(peak).toBe(3);
});

test("单个任务失败后队列继续处理等待项", async () => {
  const finished: number[] = [];
  const results = await runUpdateQueue(
    [1, 2, 3, 4],
    async (item) => {
      if (item === 2) throw new Error("download failed");
      finished.push(item);
    },
    { concurrency: 2 },
  );

  expect(finished).toEqual([1, 3, 4]);
  expect(results).toHaveLength(4);
  expect(results[1]?.error).toBeInstanceOf(Error);
});
