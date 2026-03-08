---
title: "🎨 Templates and Tera Guide"
date: "2026-03-08"
template: "post.html"
---

This system uses the ****Tera**** template engine (inspired by Jinja2 and Django) to allow you to separate your content
from your website's design.

### 1. Template Directory Structure

All template files must be stored in the `templates/` directory at the root of your project:

* ****`base.html`****: The master skeleton (contains the `<head>`, navigation, and footer).
* ****`index.html`****: Used specifically for the home page or post listings.
* ****`post.html`****: The default layout for all individual articles.

### 2. Template Inheritance

To avoid repeating HTML on every page, we use a mechanism called ****Inheritance****.

#### The Parent Template (`base.html`)

Define "placeholders" using `{% block name %}` tags that child pages will fill in.

```html  

<html>
<head>
    <title>{% block title %}{{ site.title }}{% endblock %}</title>
</head>
<body>
<header></header>

<main>
    {% block content %}{% endblock %}
</main>
</body>
</html>
```

#### **The Child Template (post.html)**

Use `{% extends "base.html" %}` at the very top of the file.

```HTML

{% extends "base.html" %}

{% block content %}
<h1>{{ meta.title }}</h1>
<div class="content">
    {{ content | safe }}
</div>
{% endblock %}
```

### **3. Available Variables**

When a page is rendered, the following objects are injected into the template from the Rust backend:

| Variable | Description                                                           |
|:---------|:----------------------------------------------------------------------|
| site     | Global settings from config.yaml (e.g. `site.title` `site.menu`).     |
| meta     | Front Matter data from the current Markdown file (e.g. `meta.title`). |
| content  | The HTML body generated from Markdown.                                |
| posts    | A list of all site articles. Each item has `.meta` and `.url`.        |

### **4. Template Selection Logic**

The system follows a specific priority when choosing which template to apply:

1. **Front Matter Override**: If your Markdown has template: "custom.html", that file is used.
2. **Automatic Detection**:

    * If the file is index.md located at the **root** of your content folder, `index.html` is applied.

3. **Fallback Default**: All other files default to `post.html`.

### **5. Common Patterns**

#### **Listing Posts (Inside index.html)**

To generate a list of links to your articles:

```HTML

<ul>
    {% for post in posts %}
    <li>
        <a href="{{ post.url }}">{{ post.meta.title }}</a> - {{ post.meta.date }}
    </li>
    {% endfor %}
</ul>
```

#### **Disabling HTML Escaping**

Markdown generates HTML tags like `<h1>` or `<p>`. To prevent Tera from displaying them as plain text, always use the
safe filter:

`{{ content | safe }}`
