---
title: Markdown Utility
type: entity
created: 2026-08-01
updated: 2026-08-01
tags: [client, utility, markdown, sanitization]
sources: [markdown.js]
---

# Markdown Utility

The `markdown.js` file provides markdown rendering with custom extensions, HTML escaping, and sanitization. It uses the `marked` library for core markdown parsing and `DOMPurify` for sanitization.

## Class: BaseMarkdownWrapper

A base class for text pre/post processing wrappers applied before and after markdown parsing.

### Method: `preprocess(text)` — returns `text` unchanged.

### Method: `postprocess(text)` — returns `text` unchanged.

## Class: SjisWrapper extends BaseMarkdownWrapper

Wraps `[sjis]...[/sjis]` tags. During preprocessing, replaces the content with a placeholder `%%%SJIS<index>`. During postprocessing, replaces the placeholder with `<div class="sjis">content</div>`.

### Internal State

- `buf` — an array that stores extracted SJIS content indexed by placeholder numbers.

## Class: TildeWrapper extends BaseMarkdownWrapper

Preserves escaped tildes (`\~`) that would otherwise be stripped by the markdown parser.

- Preprocess: replaces `\~` with `%%%T`.
- Postprocess: restores `%%%T` to `\~`.

## Class: TagPermalinkFixWrapper extends BaseMarkdownWrapper

Prevents lines starting with `#tag` from being treated as markdown headings.

- Preprocess: replaces `^#` (line-start hash followed by alphanumeric/underscore/hyphen) with `%%%#`.
- Postprocess: restores `%%%#` to `#`.

## Class: EntityPermalinkWrapper extends BaseMarkdownWrapper

Converts shorthand references to markdown links:

- `@1234` becomes `[@1234](/post/1234)` — post permalink.
- `+username` becomes `[+username](/user/username)` — user permalink.
- `#tagname` becomes `[#tagname](/posts/query=tagname)` — tag search permalink.

The regex matches these patterns when preceded by start of line, opening parenthesis, whitespace, angle brackets, or closing brackets.

## Class: SearchPermalinkWrapper extends BaseMarkdownWrapper

Postprocesses `[search]query[/search]` tags into clickable search links:
```html
<a href="/posts/query=<query>"><code><query></code></a>
```

## Class: SpoilersWrapper extends BaseMarkdownWrapper

Postprocesses `[spoiler]content[/spoiler]` tags into `<span class="spoiler">content</span>`.

## Class: SmallWrapper extends BaseMarkdownWrapper

Postprocesses `[small]content[/small]` tags into `<small>content</small>`.

## Class: StrikeThroughWrapper extends BaseMarkdownWrapper

Postprocesses `~~text~~` or `~text~` into `<del>text</del>`. Also restores escaped tildes (`\~` to `~`).

## Class: FaviconWrapper extends BaseMarkdownWrapper

Preprocesses `[icon]domain[/icon]` into a favicon link:
```html
<a href="domain"><img src="https://www.google.com/s2/favicons?domain=domain"> domain</a>
```

## Function: `escapeHtml(unsafe)`

Escapes HTML special characters: `&`, `<`, `"`, `'` to their corresponding HTML entities (`&amp;`, `&lt;`, `&quot;`, `&apos;`).

## Function: `createRenderer()`

Creates a custom `marked.Renderer` instance with a custom image renderer. The image renderer parses the href for optional `= WIDTH x HEIGHT` dimensions after the URL and adds `width` and `height` attributes to the `<img>` tag. All values are HTML-escaped.

## Function: `formatMarkdown(text)`

Renders full markdown text into sanitized HTML.

Pipeline:
1. `escapeHtml(text)`.
2. Apply `preprocess()` from each wrapper in order: SjisWrapper, TildeWrapper, TagPermalinkFixWrapper, EntityPermalinkWrapper, SearchPermalinkWrapper, SpoilersWrapper, SmallWrapper, StrikeThroughWrapper, FaviconWrapper.
3. `marked.parse(text, options)` with: custom renderer, `breaks: true`, `smartypants: true`, `headerIds: false`.
4. Apply `postprocess()` from each wrapper in reverse order.
5. `DOMPurify.sanitize(text)`.

## Function: `formatInlineMarkdown(text)`

Renders inline markdown (no block-level elements). Same pipeline as `formatMarkdown()` but uses `marked.parseInline()` and excludes SjisWrapper and TagPermalinkFixWrapper from the wrappers list.

## Exports

```js
{
    formatMarkdown,
    formatInlineMarkdown,
    escapeHtml,
}
```
