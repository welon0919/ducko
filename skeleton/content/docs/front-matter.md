---
title: "🎨 Front Matter"
date: "2026-03-06"
template: "post.html"
---

This system uses **Front Matter** (YAML format) to control settings for each individual page.

### What is Front Matter?

At the top of every Markdown file, you can wrap a block of settings between three dashes `---`:

```yaml
---
title: "My Article Title"
date: "2026-03-08"
template: "post.html"  <-- Specifies which HTML template to use
---
```

### Available Fields

title: The page title (Required).

date: The publication date, used for sorting posts.

template: The filename of the HTML template in the `templates/` folder to use. Defaults to post.html if omitted.

### Custom Templates

You can create new HTML files in the `templates/` folder (e.g., gallery.html) and assign them in your Markdown:

```yaml
template: "gallery.html"
```
