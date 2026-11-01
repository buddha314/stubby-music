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
        if window and previous_scene:
            window.scene = previous_scene



if __name__ == "__main__":
    project = Path(__file__).resolve().parents[2]
    export_scene(bpy.context.scene, project / "assets/models" / (Path(bpy.data.filepath).stem + ".glb"))
