# Security policy

## Reporting a vulnerability

Report privately through GitHub: [Report a vulnerability](https://github.com/aprxty3/project-baca/security/advisories/new). Do not open a public issue, pull request, or discussion for a security problem.

Include the affected component and version (or commit), steps to reproduce, the impact you observed, and any proof of concept. Reports in Indonesian or English are both fine.

## What to expect

| Step | Target |
|---|---|
| Acknowledgement | 3 working days |
| Triage and severity (CVSS) | 7 days |
| Fix for a critical issue | 7 days after triage |
| Fix for a high issue | 30 days after triage |
| Fix for anything else | the next release |

The advisory is published with credit, if you want it, once a fixed release is out.

## Scope

In scope: the API (`crates/server`, `crates/infra`), the web app (`crates/web`), the ingestion worker (`python_worker`), the Tauri app (`apps/tauri`), the release images on GHCR, and the GitHub workflows of this repository.

Out of scope: third-party services Rotaria runs on (Cloudflare, GitHub, the AI and mail providers), volumetric denial of service, social engineering, and findings that need a compromised device or account.

## Supported versions

Only the latest release and the `develop` branch receive fixes while the version is `0.y.z`.
