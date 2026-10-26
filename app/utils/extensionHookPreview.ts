import type { ExtensionCatalogPackage, HookReviewItem } from "~/stores/relay";

export interface HookInstallPreview {
  name: string;
  version: string;
  commitSha: string;
  files: string[];
  review: HookReviewItem[];
}

export function hookPreviewMatchesExtension(
  preview: HookInstallPreview,
  extension: ExtensionCatalogPackage,
) {
  return (
    preview.name === extension.name &&
    preview.version === extension.version &&
    preview.commitSha === extension.commitSha
  );
}
