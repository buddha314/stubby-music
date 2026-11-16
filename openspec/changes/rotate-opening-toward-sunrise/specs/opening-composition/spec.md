## ADDED Requirements

### Requirement: Sunrise sits slightly right of the opening view
The calendar SHALL open at 06:30 so the low rising sun clears the authored mountain skyline. The opening player view SHALL have a bearing of 75 degrees, with north at -Z and east at +X. At sunrise the sun SHALL remain at its physical eastward bearing, 15 degrees to the player's right at astronomical sunrise and approximately 20 degrees right at the opening time.

#### Scenario: A new game opens at sunrise
- **WHEN** the player enters the production opening without moving the camera
- **THEN** the view points east-northeast and the rising sun lies slightly right of centre

### Requirement: Rotate the composition as a whole
Authored opening placements and their orientations, and the decorative mountain rings, SHALL rotate by the same yaw about the existing spawn. The placed dormant oak, relative placement distances, nested local transforms, scene identities and prefab references SHALL be preserved. Celestial directions SHALL remain fixed.

#### Scenario: The opening is reframed
- **WHEN** the player sees the rotated opening
- **THEN** the existing oak and burbler retain their previous positions relative to the view and the bugs retain their original distance from spawn
