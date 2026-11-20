import type { Provider } from "~/types/relay";

/** 归属判定只有一处：创建人 ID 与当前身份相同即为本人。 */
export function isOwnProvider(
  provider: Pick<Provider, "owner_identity_id"> | null | undefined,
  identityId: string | null | undefined,
): boolean {
  return Boolean(identityId) && provider?.owner_identity_id === identityId;
}
