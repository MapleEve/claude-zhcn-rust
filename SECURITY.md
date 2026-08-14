# Security Policy

## Supported versions

The repository is currently a `0.1.0` scaffold and does not yet provide a
supported end-user release. Security fixes are made only on the default branch
until the first release policy is published.

## Reporting a vulnerability

Please use GitHub's **Private vulnerability reporting** for this repository.
Do not open a public issue for a suspected vulnerability and do not include
credentials, private keys, personal data, or proprietary application packages
in a report.

Include the affected commit, platform, minimal reproduction, expected impact,
and whether the issue can modify an installed Claude product. You should
receive an initial acknowledgement within seven days. A disclosure timeline
will be agreed after triage.

## Trust boundaries

- Updater artifacts must be signed. A release workflow without the configured
  updater public key and private signing secret fails closed.
- Official Claude packages are never stored in this repository or downloaded
  by CI.
- Windows localization changes application files and therefore invalidates the
  original vendor signature/package trust. A future usable release must show
  this consequence before writing and must pass real Windows launch and UI
  acceptance.
- Unknown versions, ambiguous installations, npm CLI installations, and
  Squirrel Desktop installations are read-only detection states.

## License status

No license has been selected yet. Publication of this source does not grant
permission to copy, modify, or redistribute it. External code contributions
are not accepted until a license and contribution policy are published.
