---
title: "⚙️ Configuration Guide"
date: "2026-03-05"
template: "post.html"
---

Global settings are managed in the `config.yaml` file located at the root of your project.

### Site Information

You can update the basic metadata for your website here:

```yaml
title: "My Blog"
description: "A blog about everything"
base_url: "[https://example.com](https://example.com)"
language_code: "en"
```

### Navigation Menu

The navigation bar is dynamically generated. You can add, remove, or reorder menu links in the menu section:

```yaml
menu:
  - name: "Home"
    url: "/"
  - name: "Google"
    url: "[https://google.com](https://google.com)"
  - name: "About Me"
    url: "/docs/configuration/"
```

### Author Information

You can also define your personal details to be displayed in the footer or post metadata

```yaml
author:
  name: "Your Name"
  email: "you@example.com"
```

When you save config.yaml, the Live Reload system will detect the changes and automatically update the navigation and
site-wide settings across all pages.