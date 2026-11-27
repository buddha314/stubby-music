## ADDED Requirements

### Requirement: The game starts at a main menu
A plain launch SHALL show a main menu offering Play, Host, Join (an address), Host on Steam, Settings and Quit. Host on Steam SHALL be disabled, with the reason shown, when Steam is not running. Accepting a Steam invite from the menu SHALL join that game.

#### Scenario: Launch
- **WHEN** the player starts the game with no arguments
- **THEN** the main menu is shown over the desert, with the mouse free

#### Scenario: Steam not running
- **WHEN** Steam is not running
- **THEN** Host on Steam is disabled and the menu says why

### Requirement: A loading screen covers world generation
While the world generates and the scene spawns, the game SHALL show a loading screen instead of a partial world, and SHALL go on after a timeout if the scene never arrives.

#### Scenario: Starting to play
- **WHEN** the player chooses Play
- **THEN** a loading screen shows until the world is ready, then play begins with the mouse captured

### Requirement: Settings apply live and persist
A Settings screen, reachable from the main menu and the pause menu, SHALL offer window mode, graphics and volume settings that take effect at once and are restored on the next launch.

#### Scenario: Volume
- **WHEN** the player lowers a volume and restarts the game
- **THEN** the lower volume is heard at once and after the restart

### Requirement: The pause menu is the lobby in a session
In a multiplayer session, the pause menu SHALL list the players present, offer Invite friends in a Steam session, Settings and Leave, and SHALL keep the world running behind it. When a session ends or cannot start, a screen SHALL say why and offer a way back to the main menu.

#### Scenario: Who is here
- **WHEN** a player opens the pause menu during a session with two others
- **THEN** it lists all three players, marking the host and the player themself

#### Scenario: The host leaves
- **WHEN** the host quits while a client plays
- **THEN** the client is told the session is over and can return to the main menu
