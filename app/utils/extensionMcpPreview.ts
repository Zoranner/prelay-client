import type {
  ExtensionCatalogPackage,
  ExtensionMcpPreview,
} from "~/stores/relay";

export function mcpPreviewMatchesExtension(
  preview: ExtensionMcpPreview,
  extension: ExtensionCatalogPackage,
) {
  return (
    preview.name === extension.name &&
    preview.version === extension.version &&
    preview.commitSha === extension.commitSha
  );
}
