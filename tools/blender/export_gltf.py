"""Reproducible glTF export: blender -b source.blend -P tools/blender/export_gltf.py."""
from pathlib import Path
import bpy


def export_scene(scene, destination, only=None):
    """Export `scene` (or just the objects in `only`) as a .glb. Blender 5.3 alpha
    segfaults exporting a scene that is not the active one headless, so export a
    subset from the active scene with `only` rather than from a scene of its own."""
    destination = Path(destination)
    destination.parent.mkdir(parents=True, exist_ok=True)
    window = bpy.context.window
    previous_scene = window.scene if window else None
    # Parts posed hidden at scale 0 (dormant leaves, closed blooms, stubble, halos)
    # would lose their children's transforms: Blender's exporter can't invert a
    # zero-scale parent. Export them at scale 1, tagged `start_scale: 0` (a glTF extra);
    # the game puts them back to 0 on load (stubby_world's presenter), so the model still
    # ships posed as before. Same contract as wait-for-stubs (sync-blender-asset-contracts).
    zeroed = [obj for obj in scene.objects
              if (only is None or obj in only) and obj.type not in {"CAMERA", "LIGHT"}
              and max(abs(c) for c in obj.scale) < 1e-6]
    for obj in zeroed:
        obj["start_scale"] = 0.0
        obj.scale = (1.0, 1.0, 1.0)
    try:
        if window:
            window.scene = scene
        with bpy.context.temp_override(scene=scene, view_layer=scene.view_layers[0]):
            for obj in scene.objects:
                obj.select_set(obj.type not in {"CAMERA", "LIGHT"} and (only is None or obj in only),
                               view_layer=scene.view_layers[0])
            bpy.ops.export_scene.gltf(
                filepath=str(destination), export_format="GLB", use_selection=True,
                export_yup=True, export_apply=True, export_animations=True,
                export_animation_mode="ACTIONS", export_force_sampling=True,
                export_materials="EXPORT", export_extras=True,
            )
    finally:
        for obj in zeroed:
            obj.scale = (0.0, 0.0, 0.0)
            del obj["start_scale"]
        if window and previous_scene:
            window.scene = previous_scene



if __name__ == "__main__":
    project = Path(__file__).resolve().parents[2]
    # assets-src/blender/<folder>/<id>.blend exports to assets/models/<folder>/<id>.glb.
    source = Path(bpy.data.filepath).resolve()
    folder = source.parent.relative_to(project / "assets-src/blender")
    export_scene(bpy.context.scene, project / "assets/models" / folder / (source.stem + ".glb"))
