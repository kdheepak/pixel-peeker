# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.2.0](https://github.com/kdheepak/pixel-picker/compare/v0.1.1...v0.2.0) - 2025-08-07

### Added

- *(ui)* freeze color selection when history color is clicked
- *(main)* increase window height to 500 pixels
- *(ui)* [**breaking**] replace egui/eframe with iced and tokio for new UI framework
- *(ui)* add crosshair to center cell in pixel grid
- *(ui)* add color history with swatches and copy-to-clipboard feature
- *(main)* add device_state field to PixelPickerApp and update usage

### Other

- "chore(ci): remove install dependencies step from workflows"
- Update README.md
- *(ci)* remove install dependencies step from workflows
- *(main)* update window settings and remove unused tokio dependency
- *(deps)* update iced and tokio to latest versions and refactor for new iced API
- *(main)* inline color and preview helpers for monitor capture

## [0.1.1](https://github.com/kdheepak/pixel-picker/compare/v0.1.0...v0.1.1) - 2025-08-06

### Other

- *(readme)* add install instructions and build command
- *(main)* remove emoji from status labels
- *(deps)* bump softprops/action-gh-release from 1 to 2 ([#1](https://github.com/kdheepak/pixel-picker/pull/1))
- *(github-actions)* add PR title and breaking change check workflow
- *(ci)* add release-plz workflow for automated releases
- *(github)* add FUNDING.yml for GitHub Sponsors
- *(dependabot)* add dependabot config for github-actions and cargo
- *(ci)* add GitHub Actions CD workflow for release automation
