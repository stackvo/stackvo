---
title: Configuration
description: Stackvo configuration guide - Step-by-step configuration for all platforms
---

# Configuration

Stackvo's configuration system has a flexible and multi-layered structure. This section detailedly explains how you can achieve full control at every level, from global system settings to project-based customizations, from custom webserver configurations to runtime settings. With three different configuration levels, maximum flexibility and customization possibilities are offered.

---

## Configuration Levels

Stackvo offers configuration at 3 different levels. Each level is designed for a different purpose and works together to provide maximum flexibility:

<div class="grid cards" markdown>

-   :material-cog:{ .lg .middle } __Global__

    ---

    Managed via `.env` file and affects the entire system

    [:octicons-arrow-right-24: Global Configuration](global.md)

-   :material-file-cog:{ .lg .middle } __Project__

    ---

    Project-specific settings are defined with `stackvo.json` file

    [:octicons-arrow-right-24: Project Configuration](project.md)

-   :material-file-edit:{ .lg .middle } __Custom__

    ---

    Custom webserver and runtime settings in `.stackvo/` directory

    [:octicons-arrow-right-24: Custom Configuration](custom.md)
</div>
