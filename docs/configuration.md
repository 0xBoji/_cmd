# Configuration

`view` currently uses environment variables rather than a user config file.

## Environment Variables

| Variable | Surface | Purpose |
| :--- | :--- | :--- |
| `VIEW_DEMO` | `cli`, `desktop` | Enables synthetic demo data when set to `1`, `true`, `yes`, `on`, or `demo`. |
| `VIEW_HISTORY_FILE` | `desktop` | Overrides the persisted shell history file path used for command retention. |
| `VIEW_WEB_PORT` | `desktop` | Overrides the background web API port. |
| `VIEW_DESKTOP_DEBUG_LOG` | `desktop` | Appends desktop debug messages to the given file path. |
| `VIEW_DESKTOP_SCREENSHOT_TO` | `desktop` | Saves a desktop screenshot to the given file path during screenshot automation. |

## Planned Config

A file-backed configuration layer may be added later if the desktop or web
surfaces need persistent user preferences. Until then, avoid documenting Kaku or
WezTerm-style Lua config as `view` behavior.
