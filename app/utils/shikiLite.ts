type CodeToHtmlOptions = {
  lang?: string;
};

type CodeToTokensOptions = {
  lang?: string;
  theme?: string;
};

function escapeHtml(value: string) {
  return value.replace(
    /[&<>"']/g,
    (character) =>
      ({
        "&": "&amp;",
        "<": "&lt;",
        ">": "&gt;",
        '"': "&quot;",
        "'": "&#39;",
      })[character] ?? character,
  );
}

export async function codeToHtml(
  code: string,
  options: CodeToHtmlOptions = {},
) {
  const language = options.lang?.trim() || "text";
  return `<pre class="shiki" data-language="${escapeHtml(language)}"><code>${escapeHtml(code)}</code></pre>`;
}

export async function codeToTokensBase(
  code: string,
  _options: CodeToTokensOptions = {},
) {
  return code.split("\n").map((line, offset) => [{ content: line, offset }]);
}
