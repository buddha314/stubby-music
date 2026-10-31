"""Create only our asset scene; preserve the user's open scene when used through MCP.

Reproduce: blender -b --factory-startup -P tools/blender/create_lightning_bug.py
"""
from pathlib import Path
import math
import bmesh
import sys
import bpy
from mathutils import Vector

PROJECT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(PROJECT / "tools/blender"))
from export_gltf import export_scene


def build():
    # Keep previous assets on rerun rather than touching unrelated user data.
    scene = bpy.data.scenes.new("LightningBugAsset")
    scene.unit_settings.system = "METRIC"
    scene.unit_settings.scale_length = 1.0
    scene.render.fps = 60
    scene.frame_start, scene.frame_end = 1, 19
    collection = bpy.data.collections.new("STUBBY_LightningBug")
    scene.collection.children.link(collection)
    material = bpy.data.materials.new("LightningBug_WhiteEmission")
    material.use_nodes = True
    bsdf = material.node_tree.nodes.get("Principled BSDF")
    bsdf.inputs["Base Color"].default_value = (1, 1, 1, 1)
    bsdf.inputs["Roughness"].default_value = 0.4
    bsdf.inputs["Emission Color"].default_value = (1, 1, 1, 1)
    bsdf.inputs["Emission Strength"].default_value = 80000
    material.diffuse_color = (1, 1, 1, 1)
    root = bpy.data.objects.new("LightningBug_FlashRoot", None)
    collection.objects.link(root)
    root["asset_id"] = "lightning_bug"
    root["rest_visibility"] = "hidden"
    root["pulse_driver"] = "audio_sample_timestamp"

    def own(obj):
        for owner in list(obj.users_collection):
            owner.objects.unlink(obj)
        collection.objects.link(obj)
        obj.parent = root
        obj.data.materials.append(material)
        return obj

    for name, location, scale in [
        ("Abdomen", (0, -0.022, 0), (0.018, 0.037, 0.014)),
        ("Thorax", (0, 0.022, 0.003), (0.012, 0.016, 0.012)),
        ("Head", (0, 0.044, 0.004), (0.010, 0.010, 0.009)),
    ]:
        mesh = bpy.data.meshes.new("LightningBug_" + name)
        bm = bmesh.new()
        bmesh.ops.create_icosphere(bm, subdivisions=2, radius=1)
        for vertex in bm.verts:
            vertex.co = Vector(tuple(vertex.co[i] * scale[i] for i in range(3)))
        bm.to_mesh(mesh)
        bm.free()
        obj = own(bpy.data.objects.new("LightningBug_" + name, mesh))
        obj.location = location
        for face in mesh.polygons:
            face.use_smooth = True
    for side in [-1, 1]:
        vertices = [(0.004 * side, 0.02, 0.009), (0.045 * side, 0.035, 0.018),
                    (0.060 * side, 0.005, 0.01), (0.018 * side, -0.022, 0.005)]
        mesh = bpy.data.meshes.new("LightningBug_WingMesh")
        mesh.from_pydata(vertices, [], [(0, 1, 2), (0, 2, 3)])
        own(bpy.data.objects.new("LightningBug_Wing", mesh))
        for i in range(3):
            start = Vector((0.008 * side, 0.01 - i * 0.011, -0.002))
            end = Vector((0.027 * side, 0.022 - i * 0.018, -0.014))
            delta = end - start
            rotation = delta.to_track_quat("Z", "Y")
            vertices = [rotation @ Vector((0.0012 * math.cos(j * math.tau / 6),
                        0.0012 * math.sin(j * math.tau / 6), z)) + start
                        for z in (0, delta.length) for j in range(6)]
            faces = [(j, (j+1)%6, (j+1)%6+6, j+6) for j in range(6)]
            faces.extend([tuple(range(5, -1, -1)), tuple(range(6, 12))])
            mesh = bpy.data.meshes.new("LightningBug_LegMesh")
            mesh.from_pydata(vertices, [], faces)
            own(bpy.data.objects.new("LightningBug_Leg", mesh))
    for frame, value in [(1, 1.0), (2, 0.98), (7, 0.64), (14, 0.15), (19, 0.0)]:
        root.scale = (value,) * 3
        root.keyframe_insert(data_path="scale", frame=frame, group="White flash")
    root.animation_data.action.name = "white_flash"
    scene.frame_set(1)

    world = bpy.data.worlds.new("LightningBug_PreviewWorld")
    world.use_nodes = True
    world.node_tree.nodes["Background"].inputs["Color"].default_value = (0.003, 0.004, 0.006, 1)
    world.node_tree.nodes["Background"].inputs["Strength"].default_value = 0.1
    scene.world = world
    camera_data = bpy.data.cameras.new("LightningBug_PreviewCamera")
    camera = bpy.data.objects.new("LightningBug_PreviewCamera", camera_data)
    scene.collection.objects.link(camera)
    camera.location = (0.18, -0.23, 0.23)
    camera.rotation_euler = (-camera.location).to_track_quat("-Z", "Y").to_euler()
    camera.data.lens = 50
    scene.camera = camera
    scene.render.resolution_x = scene.render.resolution_y = 768
    scene.render.resolution_percentage = 100
    scene.render.engine = "CYCLES"
    scene.cycles.samples = 24
    scene.view_settings.view_transform = "AgX"
    scene.view_settings.look = "AgX - Medium High Contrast"
    scene.view_settings.exposure = -11
    source = PROJECT / "assets-src/blender/lightning_bug.blend"
    source.parent.mkdir(parents=True, exist_ok=True)
    if bpy.app.background:
        # In a dedicated headless process, save a normal, directly openable project.
        if bpy.context.window:
            bpy.context.window.scene = scene
        for other in list(bpy.data.scenes):
            if other != scene:
                bpy.data.scenes.remove(other)
        bpy.ops.wm.save_as_mainfile(filepath=str(source), check_existing=False)
        export_scene(scene, PROJECT / "assets/models/lightning_bug.glb")
    else:
        # Never save the user's whole session or switch their viewport scene.
        # Export in a dedicated background process after this MCP build finishes.
        bpy.data.libraries.write(str(source), {scene}, fake_user=True)
    return {"scene": scene.name, "source": str(source), "model": str(PROJECT / "assets/models/lightning_bug.glb"),
            "animation": "white_flash", "objects": len(collection.objects)}


if __name__ == "__main__":
    report = build()
    if "result" in globals():
        result.update(report)
    else:
        print(report)
