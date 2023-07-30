# Bridge

> 🛑 This branch is no longer maintained, as I've switched to using a bevy plugin. Check out the main branch!

> ⚠️ Hypixel has historically banned accounts running bot software as Azalea is not a regular client. Use at your own risk!

## Deploying
*Deploying on railway button here*

### Requirements
- A Minecraft account
- A Discord bot:
  - Permissions needed:
    - `Manage Webhooks`
    - `View Channels`
    - `Add Reactions`
    - `Send Mesaages`
  - Scopes needed:
    - `bot`
    - `applications.commands`

## To do
- [x] Add all guild events
- [x] Add discord slash commands
- [x] Slash command replies
- [x] Slash command autocomplete
- [x] Sanitise discord nicks and content
- [x] Online/offline messages
- [x] Console Logging (`RUST_LOG`)
- [x] Console Logging (`stdout`)
- [x] Discord Logging
- [x] Clean up error handling
- [ ] Chat message feedback (emojis)
- [ ] Command retries for `You are sending commands too fast!`
- [x] New threads for each Minecraft event