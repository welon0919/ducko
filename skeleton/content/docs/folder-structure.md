---
title: "📂 Project Structure"
date: "2026-03-07"
template: "post.html"
---

Understanding your project's folder layout is crucial for development.

### Core Directories

Your project contains the following structure:

```text
my-project/
├── config.toml       # Global configuration (site title, menu)
├── content/          # Your content pages (Markdown)
│   └── posts/        # You can organize content in subfolders
├── static/           # Static assets (CSS, Images, JS)
│   └── style.css
├── templates/        # HTML Templates (Tera)
└── public/           # [Auto-Generated] The final build output
```

## Detailed Explanation

`content/`: The source of all your pages. The folder structure here mirrors your website's URL structure. For example,
`content/docs/intro.md` becomes `/docs/intro/`.

`static/`: Files here are copied directly to `public/` without modification. This is the place for images and
stylesheets.

`templates/`: Defines the look and feel of your site. You can modify `base.html` to change the layout or navigation bar.