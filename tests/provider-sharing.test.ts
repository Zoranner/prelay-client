import { expect, test } from "bun:test";
import { readFileSync } from "node:fs";

import {
  quotaFromRule,
  ruleFromQuota,
} from "../app/components/providers/sharing/rule";

const drawer = readFileSync(
  new URL("../app/components/providers/sharing/Drawer.vue", import.meta.url),
  "utf8",
);
const quotaTypes = readFileSync(
  new URL("../app/types/quota.ts", import.meta.url),
  "utf8",
);
const userTransfer = readFileSync(
  new URL(
    "../app/components/providers/sharing/UserTransfer.vue",
    import.meta.url,
  ),
  "utf8",
);
const sharingComposable = readFileSync(
  new URL("../app/composables/providerSharing.ts", import.meta.url),
  "utf8",
);
const providerList = readFileSync(
  new URL("../app/components/providers/List.vue", import.meta.url),
  "utf8",
);

test("供应商列表在名称旁标记共享供应商", () => {
  expect(providerList).toContain("isSharedProvider");
  expect(providerList).toContain("provider-share");
  expect(providerList.indexOf("provider-name-text")).toBeLessThan(
    providerList.indexOf("provider-share"),
  );
});

test("共享抽屉用三态范围和用户措辞，标题与按钮不重复上下文", () => {
  expect(drawer).toContain('title="共享设置"');
  expect(drawer).toContain('label: "不共享"');
  expect(drawer).toContain('label: "指定用户"');
  expect(drawer).toContain('label: "所有用户"');
  expect(drawer).not.toContain("保存共享设置");
  expect(drawer).not.toContain("身份");
});

test("共享配置提交完整范围和用户列表", () => {
  expect(quotaTypes).toContain("visibility: ProviderVisibility");
  expect(quotaTypes).toContain("users: ProviderShareUser[]");
  expect(quotaTypes).not.toContain("can_manage");
  expect(drawer).toContain("visibility: visibility.value");
});

test("共享保存成功后关闭抽屉，再刷新并通知", () => {
  const closeAt = sharingComposable.indexOf("close();");
  const reloadAt = sharingComposable.indexOf("await options.loadProviders();");
  const successAt = sharingComposable.indexOf(
    'notifications.success("共享设置已保存")',
  );
  expect(closeAt).toBeGreaterThan(-1);
  expect(closeAt).toBeLessThan(reloadAt);
  expect(reloadAt).toBeLessThan(successAt);
  expect(sharingComposable).toContain("const saving = ref(false)");
});

test("选择共享用户使用可切换的头像胶囊", () => {
  expect(drawer).toContain("<UserTransfer");
  expect(drawer).toContain(":readonly=\"visibility === 'all'\"");
  expect(drawer).toContain(
    ":selected=\"visibility === 'all' ? selectableUsers : users\"",
  );
  expect(drawer).not.toContain('placeholder="选择用户"');
  expect(drawer).toContain('localeCompare(right.display_name, "zh-CN"');
  expect(drawer).toContain(".sort(compareUsers)");
  expect(userTransfer).not.toContain("Checkbox");
  expect(userTransfer).toContain("user-chip");
  expect(userTransfer).toContain("user-chip--selected:hover");
  expect(userTransfer).toContain("user-picker--readonly");
  expect(userTransfer).toContain('size="default"');
  expect(userTransfer).toContain(":aria-pressed=");
  expect(userTransfer).toContain('emit("add"');
  expect(userTransfer).toContain('emit("remove"');
});

test("共享额度是供应商级的一套规则，不按用户分别保存", () => {
  const ruleEditor = readFileSync(
    new URL(
      "../app/components/providers/sharing/RuleEditor.vue",
      import.meta.url,
    ),
    "utf8",
  );
  const userInput = quotaTypes.slice(
    quotaTypes.indexOf("export interface ProviderShareUserInput"),
    quotaTypes.indexOf("export interface ProviderShareUser {"),
  );
  expect(userInput).not.toContain("quota");
  expect(quotaTypes).toContain("quota: ShareQuotaRule | null");
  expect(drawer.match(/<RuleEditor/g)).toHaveLength(1);
  expect(drawer).toContain('visibility.value === "selected"');
  expect(drawer).toContain(
    "users.value.map((user) => ({ identity_id: user.identity_id }))",
  );
  expect(drawer).not.toContain("share.quota");
  expect(drawer.indexOf('class="sharing-users"')).toBeLessThan(
    drawer.indexOf('class="sharing-field__label">共享额度'),
  );
  expect(drawer).toContain('class="sharing-field__label">共享额度');
  expect(ruleEditor).toContain("width: 100%");
  expect(ruleEditor).toContain("grid-template-columns:");
});

test("关闭有未保存修改的共享设置需要确认", () => {
  expect(drawer).toContain("const dirty = computed");
  expect(drawer).toContain("放弃共享设置");
});

test("共享额度回填后保存不改变 token 数", () => {
  const small = ruleFromQuota({ amount: 1000, accumulate: "none" });
  expect(small.amount).toBe("0.00001");
  expect(quotaFromRule(small)).toEqual({ amount: 1000, accumulate: "none" });

  const odd = ruleFromQuota({ amount: 12_345_678, accumulate: "week" });
  expect(odd.amount).toBe("0.12345678");
  expect(quotaFromRule(odd)).toEqual({
    amount: 12_345_678,
    accumulate: "week",
  });
});
