## ADDED Requirements
### Requirement: Hard graphical test limit
Each diagnostic SHALL stop its own editor and game processes within 20 seconds using a watchdog independent of the renderer.
#### Scenario: Hung render loop
- **WHEN** the editor stops responding during a diagnostic
- **THEN** the external watchdog kills the test at its configured limit of no more than 20 seconds
#### Scenario: Excessive duration
- **WHEN** a test requests more than 20 seconds
- **THEN** the diagnostic refuses to launch
### Requirement: Automatic recovery
After each graphical test the diagnostic SHALL restore fixed 60Hz, reset the display output after higher-refresh tests, and restore its temporary run-configuration edits without restarting the desktop session.
#### Scenario: Test completion
- **WHEN** the timed editor exits or is killed
- **THEN** fixed 60Hz is restored and the display is powered back on
### Requirement: Isolated diagnostics
The diagnostic SHALL refuse to attach to an existing editor on its control port.
#### Scenario: Busy port
- **WHEN** the control port already belongs to an editor
- **THEN** no test editor is launched and that session is left intact
