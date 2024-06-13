# Knowling

### A desktop notes application designed for Personal Knowledge Management (PKM)

![kn.logo.384.light.png](doc/img/kn.logo.300.light.png)

Knowling aims to provide users with an intuitive platform for gathering and organizing knowledge from various research
sources. By leveraging AI, Knowling assists users in categorizing their notes and highlighting connections between them,
thereby enhancing the overall management of their personal knowledge store.

## Features

- **Fast Performance**: Knowling is developed using Rust and JavaScript, ensuring a responsive and efficient user
  experience.
- **WSIWIG Markdown Editor**: A What-You-See-Is-What-You-Get (WSIWIG) Markdown editor for seamless and straightforward
  note-taking.
- **Simple, Uncluttered UI**: The user interface is designed to be minimalistic and distraction-free, allowing users to
  focus on their content.
- **Export/Import Notes**: Easily export and import notes to manage your knowledge base across different devices and
  formats.
- **AI Integration**: AI is integrated to empower users by automatically categorizing notes and identifying meaningful
  connections between them.
- **Open Source**: Knowling is open source and licensed under the Apache 2.0 license, encouraging community
  contributions
  and
  transparency.

## Current Development Status

Knowling is currently in the early stages of development, with a minimal feature set. We are actively working on
expanding the application's capabilities and enhancing its functionality. We welcome you to check out the open feature
requests and encourage you to open new ones if you have any suggestions or ideas.

- [Open Issues](https://github.com/samkeen/knowling/issues)
- [Project view](https://github.com/users/samkeen/projects/2)

We hope you find Knowling valuable for managing your personal knowledge. If you have any feedback or encounter any
issues, please don't hesitate to reach out or contribute to the project.

**Why the name Knowling**: Knowling is a play on the words "Knowledge" and "Knolling", a process of arranging objects to
create clean and organized
spaces. This reflects our goal of helping users keep their knowledge organized and easily accessible.

---

---

## Developing Knowling

Knowling is built atop [Tauri 1.x](https://tauri.app/)

### Project setup

```bash
npm install
npm run tauri dev

```

## Development

### Build

https://tauri.app/v1/api/cli/

```bash
npm run tauri build
```

Development follows the common practices of [developing a Tauri application](https://tauri.app/v1/guides/).

### Debugging in RustRover

https://tauri.app/v1/guides/debugging/rustrover/