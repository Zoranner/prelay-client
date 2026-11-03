import { Avatar as DiceBearAvatar, Style } from "@dicebear/core";
import cutouts from "@dicebear/styles/cutouts.json";

const cutoutsStyle = new Style(cutouts);
const avatarSources = new Map<string, string>();

export function identityAvatarSrc(identityId: string) {
  const cached = avatarSources.get(identityId);
  if (cached) {
    return cached;
  }
  const source = new DiceBearAvatar(cutoutsStyle, {
    seed: identityId,
  }).toDataUri();
  avatarSources.set(identityId, source);
  return source;
}
