<script lang="ts">
  import { marked } from "marked";
  import DOMPurify from "dompurify";

  type Props = {
    source: string;
    inline?: boolean;
  };

  let { source, inline = false }: Props = $props();

  marked.setOptions({ gfm: true, breaks: true });

  const html = $derived.by(() => {
    if (!source) return "";
    const raw = inline
      ? (marked.parseInline(source) as string)
      : (marked.parse(source) as string);
    return DOMPurify.sanitize(raw, {
      ADD_ATTR: ["target", "rel"],
    });
  });
</script>

<div class="markdown" class:inline>
  {@html html}
</div>

<style>
  .markdown {
    font-size: 14px;
    color: #c8c8b8;
    line-height: 1.6;
    word-wrap: break-word;
    overflow-wrap: anywhere;
  }

  .markdown.inline {
    font-size: 13px;
    line-height: 1.55;
    color: #c0c0b0;
  }

  .markdown :global(p) {
    margin: 0 0 10px;
  }
  .markdown :global(p:last-child) { margin-bottom: 0; }

  .markdown :global(h1),
  .markdown :global(h2),
  .markdown :global(h3),
  .markdown :global(h4) {
    color: #e8e8d8;
    font-weight: 700;
    margin: 18px 0 8px;
    line-height: 1.3;
  }
  .markdown :global(h1) { font-size: 20px; }
  .markdown :global(h2) { font-size: 17px; }
  .markdown :global(h3) { font-size: 15px; }
  .markdown :global(h4) { font-size: 14px; color: #c8c8b8; }
  .markdown :global(h1:first-child),
  .markdown :global(h2:first-child),
  .markdown :global(h3:first-child) { margin-top: 0; }

  .markdown :global(a) {
    color: #b8e060;
    text-decoration: none;
  }
  .markdown :global(a:hover) { text-decoration: underline; }

  .markdown :global(code) {
    font-family: "SF Mono", Menlo, monospace;
    font-size: 12px;
    background: rgba(255, 255, 255, 0.06);
    padding: 1px 5px;
    border-radius: 4px;
    color: #d8d8c0;
  }

  .markdown :global(pre) {
    background: rgba(0, 0, 0, 0.35);
    border: 1px solid rgba(255, 255, 255, 0.06);
    border-radius: 6px;
    padding: 10px 12px;
    overflow-x: auto;
    margin: 10px 0;
  }
  .markdown :global(pre code) {
    background: none;
    padding: 0;
    font-size: 12px;
    color: #c0c0a8;
  }

  .markdown :global(blockquote) {
    border-left: 3px solid rgba(184, 224, 96, 0.3);
    padding: 2px 0 2px 12px;
    margin: 10px 0;
    color: #a0a090;
  }

  .markdown :global(ul),
  .markdown :global(ol) {
    margin: 6px 0 10px;
    padding-left: 22px;
  }
  .markdown :global(li) { margin-bottom: 3px; }

  .markdown :global(hr) {
    border: none;
    border-top: 1px solid rgba(255, 255, 255, 0.08);
    margin: 14px 0;
  }

  .markdown :global(table) {
    border-collapse: collapse;
    margin: 10px 0;
    font-size: 13px;
  }
  .markdown :global(th),
  .markdown :global(td) {
    border: 1px solid rgba(255, 255, 255, 0.08);
    padding: 5px 10px;
    text-align: left;
  }
  .markdown :global(th) {
    background: rgba(255, 255, 255, 0.04);
    font-weight: 700;
    color: #d8d8c8;
  }

  .markdown :global(img) {
    max-width: 100%;
    border-radius: 6px;
  }

  .markdown :global(input[type="checkbox"]) {
    margin-right: 6px;
    vertical-align: middle;
  }
</style>
