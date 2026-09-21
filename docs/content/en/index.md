---
template: home.html
title: StackVo
hide:
  - navigation
  - toc
  - footer
hero:
  eyebrow: Local development, in a window
  title: A local development environment that is a desktop app
  text: >-
    One Docker container per project, with its own runtime version, its own
    database and its own domain — https://shop.loc, trusted certificate
    included. PHP, Node, Python, Go, Ruby or Rust. StackVo runs on your host,
    not inside a container, so it opens even when Docker is down.
  meta: macOS · Windows · Linux · free, MIT-licensed
  tabs_label: Screenshots
  shots:
    - { file: web/dashboard.webp, label: Dashboard, alt: "The dashboard: CPU, memory, disk, network and the running projects" }
    - { file: web/projects.webp, label: Projects, alt: "The projects page: every project and the state of its container" }
    - { file: web/project-detail.webp, label: Project page, alt: "A project's page: 45 panes, one subject each" }
    - { file: web/mail.webp, label: Mail, alt: "The mail inbox: everything the app sent, read inside the window" }
    - { file: web/logs.webp, label: Logs, alt: "Every project's logs in one place, live" }
  secondary:
    label: Get started
    href: getting-started/
stack:
  title: A complete local stack
  text: >-
    Runtimes, web servers, databases and tools, installed as instances and
    picked per project. Laravel, Symfony, WordPress and Next.js get presets;
    anything else is a folder with a runtime.
  link: The full list
  href: reference/supported-stack/
  more: >-
    Also search, monitoring, queues and admin UIs — and more than 80 PHP
    extensions.
  groups:
    - title: Runtimes
      icon: material/code-tags
      list:
        - { name: PHP 5.6–8.5, icon: simple/php }
        - { name: Node.js, icon: simple/nodedotjs }
        - { name: Python, icon: simple/python }
        - { name: Go, icon: simple/go }
        - { name: Ruby, icon: simple/ruby }
        - { name: Rust, icon: simple/rust }
    - title: Web servers
      icon: material/server
      list:
        - { name: nginx, icon: simple/nginx }
        - { name: Apache, icon: simple/apache }
        - { name: Caddy, icon: simple/caddy }
        - { name: FrankenPHP }
        - { name: Swoole }
        - { name: RoadRunner }
    - title: Databases
      icon: material/database
      list:
        - { name: MySQL }
        - { name: MariaDB, icon: simple/mariadb }
        - { name: PostgreSQL, icon: simple/postgresql }
        - { name: MongoDB, icon: simple/mongodb }
        - { name: ClickHouse, icon: simple/clickhouse }
        - { name: Cassandra, icon: simple/apachecassandra }
        - { name: MS SQL Server }
    - title: Cache and messaging
      icon: material/memory
      list:
        - { name: Redis, icon: simple/redis }
        - { name: Valkey }
        - { name: Memcached }
        - { name: Dragonfly }
        - { name: RabbitMQ, icon: simple/rabbitmq }
        - { name: Kafka, icon: simple/apachekafka }
    - title: Tools
      icon: material/tools
      list:
        - { name: Mailpit, icon: material/email-outline }
        - { name: phpMyAdmin, icon: simple/phpmyadmin }
        - { name: Adminer, icon: simple/adminer }
        - { name: pgAdmin, icon: material/database-search }
        - { name: Grafana, icon: simple/grafana }
        - { name: MinIO, icon: simple/minio }
features:
  title: What the others do not
  text: >-
    Every tool in this category gives you PHP, a database and a domain. These
    are the things you get here and nowhere else.
  list:
    - icon: material/source-branch
      title: A full environment per git branch
      text: Each worktree gets its own hostname and its own database. Preview environments, locally and free.
    - icon: material/speedometer
      title: Why was this request slow?
      text: Profiler, query log and your dump() calls on one axis; replay a recorded request with one click.
    - icon: material/package-variant-closed
      title: One container per project
      text: PHP 5.6–8.5, Node, Python, Go, Ruby, Rust — what runs is what a Dockerfile says, not what your brew history says.
    - icon: material/robot-outline
      title: MCP for AI assistants, with a leash
      text: 38 tools. Writes sit behind an explicit flag, a project scope and a time limit.
    - icon: material/rocket-launch-outline
      title: Builds the production image
      text: The same manifest renders the image you ship, and exports a devcontainer for the cloud.
    - icon: material/chart-timeline-variant
      title: Measures what Docker costs you
      text: '"shop has held 4.2 GB·hours and used 38 minutes of CPU today." The only tool here that says so.'
download:
  title: Download
  text: >-
    Ten installers per release, every platform and both CPU families. Not code-signed by decision;
    every release publishes SHA256 checksums and the updater verifies a
    minisign signature.
  hero_label: 'Download for {os}'
  hero_fallback: Download
  version_fallback: Latest release
  yours: Your system
  others: Other formats
  integrity: SHA256 checksums beside every file, minisign-signed updates
  all: All downloads on GitHub
  help: If the OS says the build is not signed
  list:
    - key: mac
      icon: material/apple
      title: macOS
      note: macOS 10.15 or later
      formats:
        - label: Apple Silicon (.dmg)
          file: 'StackVo_{v}_aarch64.dmg'
        - label: Intel (.dmg)
          file: 'StackVo_{v}_x64.dmg'
    - key: win
      icon: material/microsoft-windows
      title: Windows
      note: Windows 10 or later
      formats:
        - label: x64 installer (.exe)
          file: 'StackVo_{v}_x64-setup.exe'
        - label: ARM64 installer (.exe)
          file: 'StackVo_{v}_arm64-setup.exe'
    - key: linux
      icon: material/linux
      title: Linux
      note: x86_64 and aarch64
      formats:
        - label: .deb, x86_64
          file: 'StackVo_{v}_amd64.deb'
        - label: .rpm, x86_64
          file: 'StackVo-{v}-1.x86_64.rpm'
        - label: AppImage, x86_64
          file: 'StackVo_{v}_amd64.AppImage'
        - label: .deb, aarch64
          file: 'StackVo_{v}_arm64.deb'
        - label: .rpm, aarch64
          file: 'StackVo-{v}-1.aarch64.rpm'
        - label: AppImage, aarch64
          file: 'StackVo_{v}_aarch64.AppImage'
steps:
  title: Three steps to a running project
  text: No terminal, no config file, no Docker knowledge. The window asks, you answer.
  alt: The new-project drawer
  caption: The new-project drawer — name, domain, runtime and services on one panel.
  cta: Download and try it
  list:
    - title: Pick a folder for the workspace
      text: The first launch asks exactly one question. An empty folder is enough; the app writes the rest.
    - title: Name the project, choose the stack
      text: Laravel, WordPress, Symfony, plain PHP, Node… then the runtime version, the web server and the services.
    - title: Press Create
      text: The build streams Docker's own output. When it finishes, the domain opens in your browser with no certificate warning.
compare:
  title: How it compares
  text: >-
    What each tool chose, not which is better. Compiled from each project's
    own documentation.
  words:
    'yes': 'Yes'
    'no': 'No'
    partial: partial
  columns: [StackVo, Herd, ServBay, Laragon, DDEV, FlyEnv]
  rows:
    - [Approach, Docker + desktop, Native binaries, Native binaries, Native binaries, Docker + CLI, Native binaries]
    - [Platforms, mac · Win · Linux, mac · Win, mac · Win, Win, mac · Win · Linux, mac · Win · Linux]
    - [Project isolation, Container, Site, Site, Site, Container, Site]
    - [Per-branch env with its own DB, 'Yes', 'No', 'No', 'No', partial, 'No']
    - ['Request-level "why slow"', 'Yes', partial (Pro), 'No', 'No', 'No', 'No']
    - [Replay a recorded request, 'Yes', 'No', 'No', 'No', 'No', 'No']
    - [In-app mail inbox, 'Yes', Pro, Pro, 'Yes', web UI, 'Yes']
    - [Named DB snapshots, Yes + scheduled, 'No', scheduled, scheduled, 'Yes', 'No']
    - [MCP for AI assistants, 38 tools scoped, 'No', 'Yes', 'No', 'No', 'Yes']
    - [Builds the production image, 'Yes', 'No', 'No', 'No', 'No', 'No']
    - [Import sources, '7', '1', a few, 'No', a few, a few]
    - [Price, 'Free, MIT', Free + Pro $99/yr, Free + Pro, Free, 'Free, Apache-2', Free + Pro $10]
  more: The full table, and what Docker costs you in return
  href: reference/comparison/
pricing:
  eyebrow: Pricing
  price: $0
  per: forever
  title: Everything, for everyone
  text: >-
    There is no Pro tier and there is not going to be one. Nothing is gated
    behind an account, a licence key or a nag screen.
  list:
    - Every feature, on every platform
    - No account, no licence key, no telemetry
    - MIT-licensed — the code stays available whatever happens
    - Updates verified with a minisign signature
  cta: Download
  aside_title: What that means, plainly
  facts:
    - icon: material/account-outline
      title: Maintained by one person
      text: Issues are read and most get answered. "Most" is the honest word; there is no response-time commitment.
    - icon: material/cash-off
      title: Not funded
      text: No company, no foundation, no sponsorship behind it today. Several tools in this category have one; this one does not.
    - icon: material/file-document-outline
      title: No support contract
      text: The MIT licence means the code stays available whatever happens. It does not mean somebody will be there to fix it.
  aside_link: Sponsoring is what changes it
faq:
  title: Commonly asked questions
  text: Short answers. The longer ones are in the documentation and in the README.
  more: Something else?
  more_label: Ask in Discussions
  more_href: https://github.com/stackvo/stackvo/discussions
  list:
    - q: Is it really free?
      a: >-
        Yes. Free, MIT-licensed, every feature on every platform. No paid tier, no account, no telemetry.
    - q: Is it only for Laravel?
      a: >-
        No. Laravel, Symfony and WordPress get presets, but a project is any folder with a runtime: PHP, Node, Python, Go, Ruby or Rust.
    - q: Is Docker required?
      a: >-
        Yes: Docker Desktop, Docker Engine, or Podman, Colima or OrbStack. If it is not running, the app opens, says so and offers to start it.
    - q: Why isn't it code-signed?
      a: >-
        Certificates are recurring costs with an identity attached. Instead every release publishes SHA256 checksums and the updater verifies a minisign signature.
    - q: Can I run two versions of the same service at once?
      a: >-
        Yes. Services are instances: MySQL 8.0 and 8.4 side by side, each project on the one it asked for.
    - q: What does Docker cost me compared with a native tool?
      a: >-
        An install that includes Docker, an image build on a project's first start, and the VM's idle memory. In return, what runs is what a Dockerfile says.
    - q: What is the state of Windows support?
      a: >-
        Windows is in the CI matrix and the suite passes there; the hosts write through UAC and the named pipe against Docker Desktop are still unverified on a real machine.
    - q: Where does my data go?
      a: >-
        Nowhere. No telemetry. The network calls are the ones you press, plus one help document fetched from the repository when you open a help panel.
more:
  title: And then
  text: The rest of what the window does, each with its own page.
  list:
    - title: Mail, dumps and logs in the window
      href: guide/everyday-use/
      text: Sent mail is captured, dump() output is collected, every project's logs are live in one place.
    - title: Named database snapshots
      href: help/page-project-detail/
      text: Take one before a migration and restore it by name. Scheduled snapshots too.
    - title: Public tunnels, eight providers
      href: help/project-tunnel/
      text: Cloudflare, ngrok, Tailscale, zrok, Pinggy, localtunnel, localhost.run, LocalXpose.
    - title: Imports from seven tools
      href: help/settings-workspace-import/
      text: XAMPP, Laragon, MAMP, Valet, Sail, Herd, DDEV — it brings your projects over.
    - title: Doctor
      href: help/settings-diagnostics/
      text: Says what is broken and how to fix it, line by line, with a button beside most findings.
    - title: A terminal and a TUI too
      href: guide/cli/
      text: Everything the window does has a stackvo command; the CLI is there for scripts and CI.
sponsor:
  eyebrow: Sponsorship
  title: Become a sponsor
  secondary: Or star the repository
  text: >-
    StackVo is free, MIT-licensed and maintained by one person, with no
    company behind it and no paid tier. If it saves you time, sponsoring is
    what keeps it maintained — and what turns "most issues get answered"
    into "issues get answered".
  label: Sponsor on GitHub
  href: https://github.com/sponsors/stackvo
---

StackVo is a desktop application that manages the local development
environment on your machine. Start with [Getting started](getting-started/index.md).
