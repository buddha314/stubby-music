## Existing practice
Unity GlobalObjectId combines prefab-local object and placement identity: https://docs.unity.com/en-us/engine/6000.7/script-reference/unityeditor/globalobjectid . Godot resource_local_to_scene duplicates per-instance state: https://docs.godotengine.org/en/stable/classes/class_resource.html#class-resource-property-resource-local-to-scene .

Jackdaw already provides SceneNodeId for persisted placements, IsA for the prefab root, and PrefabEntityId for its local objects. Use these to scope the legacy lightning_bug-placed-0 template identity. This is a compatibility migration of template data; explicit non-template ResourceNode IDs and explicit PlacedAsset IDs stay authoritative. Default inherited clock 1 is unbound before runtime binding, which uses the existing ModuleIds allocator. Explicit other clocks remain authored. Missing native placement IDs retain the existing hierarchy identity fallback. Moving a persisted placement does not change its native identity.

No editor changes, random IDs, transform-derived IDs for persisted prefab placements, custom prefab loader or DSP.
