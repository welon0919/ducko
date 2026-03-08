---
title: "📦 Understanding Page Bundles"
date: "2026-03-08"
template: "post.html"
---

As your website grows, managing images and assets in a single global `static/` folder can become messy. **Page Bundles**
allow you to keep a page's Markdown file and its specific assets together in one folder.

### 1. Flat Files vs. Page Bundles

In a traditional setup, you have a single `.md` file. In a Page Bundle setup, you create a directory named after your
post and place an `index.md` inside it.

**Traditional (Flat):**

```text
content/
└── hello-world.md
static/
└── images/
    └── sunset.jpg
```

Page Bundle:

```Plaintext
content/
└── hello-world/
├── index.md      <-- The content
└── sunset.jpg    <-- Asset specific to this post
```

### 2. Why use Page Bundles?

Organization: Everything related to a post lives in one place. If you delete the post folder, the images go with it. In
fact, this documentation is a page bundle!

Relative Paths: You can reference images using simple relative paths in Markdown.

Portability: It's easier to move sections of your site around when the assets are bundled with the text.

### 3. Referencing Bundle Assets

When using Page Bundles, our Rust SSG automatically detects non-Markdown files within the folder and copies them to
the same output directory as the HTML.

This means in your index.md, you can reference an image like this:

```Markdown
![A beautiful sunset](sunset.jpg)
```

Instead of this:

```Markdown
![A beautiful sunset](/static/images/posts/hello-world/sunset.jpg)
```

### 4. Anatomy of a Bundle

A Page Bundle can contain many types of files:

`index.md`: The heart of the bundle. This is the file that gets rendered into HTML.

`Images`: `.jpg`, `.png`, `.svg`, etc.

`Data`: `.json`, `.csv`, or `.yaml ` files that you might want to fetch via custom template logic.

Code: Snippets or text files you want to link to.

### 5. Rules to Remember

The Entry Point: The main content file must be named index.md.

Output Path: If your bundle is at `content/posts/my-travel/`, the generated page will be at
`public/posts/my-travel/index.html`.

No Sub-Pages: Files in a Page Bundle (a folder with `index.md`) are treated as assets. If you want a folder to contain
multiple separate pages, you should use standard flat files or nested folders without an index.md at the top level.

Tip: Page Bundles are highly recommended for tutorials and image-heavy blog posts!