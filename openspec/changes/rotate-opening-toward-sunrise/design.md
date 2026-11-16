## Decisions

The existing Bevy spawn is x=0, z=6. Rotate the authored scene by -75 degrees around Y about that point, and apply the same yaw to the opening camera and decorative ridge rings. A camera bearing of 75 degrees keeps the world-space eastward sunrise (+X) 15 degrees to its right. Do not rotate the celestial frame or alter the physical sunrise. Start the calendar at 06:30, about 112.5 real seconds after astronomical sunrise at normal speed, so the sun clears the 4.5-degree maximum authored mountain skyline. At that time the sun is about 20 degrees right of the 75-degree view. The first exact-06:00 screenshot verified orientation but hid the sun behind the ridge; the later start follows AGENTS.md guidance.

Store the heading and spawn XZ as data constants in stubby_core, shared by the player and world systems. The saved scene holds authored numeric transforms; loading adds no second rotation. Preserve every SceneNodeId, prefab reference, nested local transform and gameplay instance ID. Rotate parent placements, not their model geometry, and retain the existing oak and burbler arrangement. This is an edit to the production opening, not a replacement with the sibling's different distances or a rotation of diagnostic fixtures.

The Godot scene has yaw -90 degrees on content groups and the player, preserving composition. Here the user's request for a sun slightly to the right takes precedence over the earlier exact-east wording. The dormant oak already placed about 97 m from spawn remains in the same part of the view.

## Validation

Check the opening bearing/sun offset and preserve grounded movement/look tests. Resolve the opening scene through Jackdaw's existing parser; verify the oak stays ahead/left and the burbler ahead, and that pickups remain beyond hearing range as before. Compare transformed positions/orientations against the previous scene to verify a rigid rotation, preserving all distances. Build stubby_jackdaw before a standalone opening screenshot using only tools/jackdaw_timed_test.py and its independent 20-second watchdog.
