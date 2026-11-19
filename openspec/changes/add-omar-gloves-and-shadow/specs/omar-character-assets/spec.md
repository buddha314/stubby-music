## ADDED Requirements

### Requirement: Omar's gloves hold the hand item
The first-person view SHALL show Omar's gloved hands holding the item in hand, following `docs/art/omar.md`: long midnight-navy gloves with the coat's flared mint sleeve at the wrist. With nothing in hand, no gloves SHALL show.

#### Scenario: The recorder is held in gloves
- **WHEN** the field recorder is in hand
- **THEN** a midnight-navy glove grips it at its existing pose, with a band of flared mint sleeve at the wrist

#### Scenario: An empty hand shows no gloves
- **WHEN** the player puts the hand item away
- **THEN** neither the item nor the gloves are drawn

### Requirement: Omar casts a shadow the camera never sees
The player SHALL cast a shadow in Omar's silhouette (tall head wrap with hanging fringed end, knee-length coat, short flared sleeves, puffed trousers gathered into boots), and the body casting it SHALL never be drawn to the player's camera.

#### Scenario: The shadow reads as Omar
- **WHEN** the low sun is behind the player standing on open ground
- **THEN** the shadow ahead of the player shows the tall wrap, the knee coat and the boots

#### Scenario: The body stays invisible
- **WHEN** the player looks down or turns in any direction
- **THEN** no part of the shadow body is drawn in the view

### Requirement: The shadow follows the player's state
The shadow SHALL change with crouching and with an item in hand, so that it agrees with what the player does.

#### Scenario: Crouching lowers the shadow
- **WHEN** the player holds C to crouch
- **THEN** the shadow's silhouette crouches

#### Scenario: Holding an item raises the arm
- **WHEN** the field recorder is in hand
- **THEN** the shadow's right arm is raised as if holding it
