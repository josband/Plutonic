# Requirements Overview

Small file to document high level requirements for Plutonic to get a minimal product up and
running. This can change and evolve or become more pluggable but this is the core of Plutonic.
I can also mark these items completed as I work on it.

## High Level Requirements

- [ ] Configurable via TOML file and/or cmd line args
- [ ] Pluggable Strategies
- [ ] Control Panel displaying metrics
- [ ] Ability to add/remove companies of interest at runtime
- [ ] Hot swappable strategies from control panel
- [ ] Configurable risk metrics
- [ ] Analytics module
- [ ] Ability to shut down and start up bot

## Potential Useful Libraries

- `axum` or `warp` for web server to control/interact with bot
- `libloading` for dynamic library loading
- `sqlx` for DB interfacing
