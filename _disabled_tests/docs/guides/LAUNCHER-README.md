# MIDI Software Center Desktop Launcher

## Quick Start

**Double-click the desktop icon** `MIDI-Software-Center.desktop` on your Desktop to launch the application.

## What Happens When You Launch

The launcher script automatically:

1. ✅ **Kills old processes** - Stops any lingering Tauri, Vite, or pnpm processes
2. ✅ **Cleans caches** - Removes stale Vite build artifacts
3. ✅ **Checks database** - Ensures PostgreSQL & Meilisearch containers are running
4. ✅ **Launches fresh** - Starts new Tauri dev instance with clean state
5. ✅ **Logs output** - Saves logs to `~/midi-center.log`

## Manual Launch

If the desktop launcher doesn't work, run manually:

```bash
cd /home/dojevou/projects/midi-software-center
./launch-midi-center.sh
```

## Quitting the Application

**Two ways to exit cleanly:**

### 1. GUI Menu (Recommended)
- Click **File → Quit** (or press `Ctrl+Q`)
- This cleanly shuts down:
  - Frontend (Vite)
  - Backend (Tauri/Rust)
  - All database connections
  - MIDI hardware connections

### 2. Close Window
- Click the window X button
- Or press `Alt+F4`

## Process Management

### Check Running Processes
```bash
ps aux | grep -E "(vite|tauri|midi-software-center)"
```

### Manual Cleanup (if needed)
```bash
pkill -9 -f "midi-software-center"
pkill -9 -f "vite.*5173"
pkill -9 -f "pnpm.*tauri"
```

### Database Control
```bash
# Stop database (keeps data)
cd /home/dojevou/projects/midi-software-center
docker-compose stop

# Start database
docker-compose up -d

# Check status
docker-compose ps
```

## Architecture

**Full Stack Components:**

| Component | Port | Process | Auto-Started |
|-----------|------|---------|--------------|
| Vite Dev Server | 5173 | Node.js | ✅ Yes |
| Tauri Backend | - | Rust binary | ✅ Yes |
| PostgreSQL | 5433 | Docker | ✅ Yes (checked) |
| Meilisearch | 7700 | Docker | ✅ Yes (checked) |

**Clean Shutdown Sequence:**
1. User clicks **File → Quit**
2. Frontend calls `shutdown_application()` command
3. Backend logs shutdown event
4. Tauri closes window and exits
5. Vite dev server terminates
6. All connections close cleanly

**Database persists** - PostgreSQL and Meilisearch containers keep running (use `docker-compose stop` to stop them).

## Troubleshooting

### Desktop Icon Not Working
```bash
# Make executable
chmod +x /home/dojevou/Desktop/MIDI-Software-Center.desktop
chmod +x /home/dojevou/projects/midi-software-center/launch-midi-center.sh

# Trust icon (GNOME)
gio set /home/dojevou/Desktop/MIDI-Software-Center.desktop metadata::trusted true
```

### Ports Already in Use
```bash
# Kill process on port 5173
lsof -ti:5173 | xargs kill -9

# Or use launcher (automatically cleans)
./launch-midi-center.sh
```

### Database Not Starting
```bash
cd /home/dojevou/projects/midi-software-center
docker-compose up -d
docker-compose logs -f postgres
```

### View Logs
```bash
# Application logs
tail -f ~/midi-center.log

# Tauri logs
tail -f /home/dojevou/projects/midi-software-center/app/logs/midi-app.log.*

# Database logs
docker-compose logs -f postgres
```

## Files

- `/home/dojevou/Desktop/MIDI-Software-Center.desktop` - Desktop launcher icon
- `/home/dojevou/projects/midi-software-center/launch-midi-center.sh` - Launch script
- `~/midi-center.log` - Application output log
- `app/logs/midi-app.log.*` - Structured Rust logs

## Support

For issues, check:
1. `~/midi-center.log` for launch errors
2. `docker-compose ps` for database status
3. `ps aux | grep midi` for hanging processes
