# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.3.3](https://github.com/kdheepak/pixel-peeker/compare/v0.3.2...v0.3.3) - 2025-08-08

### Other

- Use RELEASE_PLZ_TOKEN
- Update workflow cd.yml
- Update workflow
- Update FUNDING.yml

## [0.3.2](https://github.com/kdheepak/pixel-peeker/compare/v0.3.1...v0.3.2) - 2025-08-08

### Other

- Update cd.yml to publish on tags only
- Remove println
- Update README.md

## [0.3.1](https://github.com/kdheepak/pixel-peeker/compare/v0.3.0...v0.3.1) - 2025-08-07

### Other

- Update CD with working rust toolchain

## [0.3.0](https://github.com/kdheepak/pixel-peeker/releases/tag/v0.3.0) - 2025-08-07

### Added
- Save settings with more fallbacks
- Rename to pixel peeker
- format colors in css format
- *(settings)* add persistent user settings with window and history state
- *(preview-renderer)* improve crosshair visibility with outlined lines and dot
- *(ui)* add zoom slider and improve grid centering and crosshair visibility
- *(ui)* add shadow and border to preview container
- *(ui)* add zoom slider for preview canvas
- *(ui)* add dynamic container background color based on frozen state
- *(color)* [**breaking**] add oklch color format using palette crate and remove cmyk
- *(pixel-picker)* capture and preview only region around cursor for color selection
- *(ui)* add copy buttons and display for HSV, HSL, and CMYK color formats
- *(app)* add capture_current_color to refreeze and capture color at position
- *(ui)* freeze color selection when history color is clicked
- *(main)* increase window height to 500 pixels
- *(ui)* [**breaking**] replace egui/eframe with iced and tokio for new UI framework
- *(ui)* add crosshair to center cell in pixel grid
- *(ui)* add color history with swatches and copy-to-clipboard feature
- *(main)* add device_state field to PixelPickerApp and update usage

### Fixed
- *(capture-region)* adjust preview region clamping and pixel offset calculation
- *(ui)* update app title spacing in main view
- *(main)* update application name to "Pixel Picker"

### Other
- Release v0.3.0
- add rust-toolchain.toml with stable channel
- *(ci)* add release published trigger to cd workflow
- *(app)* remove capture throttling logic and related fields
- release v0.2.1 ([#8](https://github.com/kdheepak/pixel-peeker/pull/8))
- *(main)* simplify app state and modularize color picking logic
- release v0.2.0 ([#7](https://github.com/kdheepak/pixel-peeker/pull/7))
- *(readme)* update project name to pixel-picker
- Update README.md
- *(main)* update window settings and remove unused tokio dependency
- *(deps)* update iced and tokio to latest versions and refactor for new iced API
- *(main)* inline color and preview helpers for monitor capture
- *(ci)* add libgbm-dev to GitHub Actions dependencies
- *(ci)* add mesa EGL and GL dev packages to workflow dependencies
- *(ci)* update dependencies installation step in workflow
- *(ci)* add GitHub Actions workflow for CI
- *(ci)* add libpipewire-0.3-dev install step to release workflow
- release v0.1.1 ([#2](https://github.com/kdheepak/pixel-peeker/pull/2))
- *(readme)* add install instructions and build command
- *(main)* remove emoji from status labels
- *(deps)* bump softprops/action-gh-release from 1 to 2 ([#1](https://github.com/kdheepak/pixel-peeker/pull/1))
- *(github-actions)* add PR title and breaking change check workflow
- *(ci)* add release-plz workflow for automated releases
- *(github)* add FUNDING.yml for GitHub Sponsors
- *(dependabot)* add dependabot config for github-actions and cargo
- *(ci)* add GitHub Actions CD workflow for release automation
- add initial implementation of pixel picker

## [0.2.1](https://github.com/kdheepak/pixel-peeker/compare/v0.2.0...v0.2.1) - 2025-08-07

### Added

- *(pixel-picker)* capture and preview only region around cursor for color selection

### Fixed

- *(ui)* update app title spacing in main view

### Other

- *(main)* simplify app state and modularize color picking logic

## [0.2.0](https://github.com/kdheepak/pixel-peeker/compare/v0.1.1...v0.2.0) - 2025-08-07

### Added

- *(ui)* add copy buttons and display for HSV, HSL, and CMYK color formats
- *(app)* add capture_current_color to refreeze and capture color at position
- *(ui)* freeze color selection when history color is clicked
- *(main)* increase window height to 500 pixels
- *(ui)* [**breaking**] replace egui/eframe with iced and tokio for new UI framework
- *(ui)* add crosshair to center cell in pixel grid
- *(ui)* add color history with swatches and copy-to-clipboard feature
- *(main)* add device_state field to PixelPickerApp and update usage

### Fixed

- *(main)* update application name to "Pixel Picker"

### Other

- *(readme)* update project name to pixel-picker
- Update README.md
- *(main)* update window settings and remove unused tokio dependency
- *(deps)* update iced and tokio to latest versions and refactor for new iced API
- *(main)* inline color and preview helpers for monitor capture

## [0.1.1](https://github.com/kdheepak/pixel-peeker/compare/v0.1.0...v0.1.1) - 2025-08-06

### Other

- *(readme)* add install instructions and build command
- *(main)* remove emoji from status labels
- *(deps)* bump softprops/action-gh-release from 1 to 2 ([#1](https://github.com/kdheepak/pixel-peeker/pull/1))
- *(github-actions)* add PR title and breaking change check workflow
- *(ci)* add release-plz workflow for automated releases
- *(github)* add FUNDING.yml for GitHub Sponsors
- *(dependabot)* add dependabot config for github-actions and cargo
- *(ci)* add GitHub Actions CD workflow for release automation
