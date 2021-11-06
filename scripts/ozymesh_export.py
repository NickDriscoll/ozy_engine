bl_info = {
    "name" : "OzyMesh MeshExporter",
    "blender" : (2, 80 ,0),
    "category" : "Export"
}

import bpy
import struct
from bpy_extras.io_utils import ImportHelper
from bpy.props import StringProperty
from mathutils import Matrix, Vector

from ozy_common import *

class MeshExporter(bpy.types.Operator, ImportHelper):
    """Export selection to OzyMesh file (.ozy)"""      # Use this as a tooltip for menu items and buttons.
    bl_idname = "ozymesh.meshexporter"        # Unique identifier for buttons and menu items to reference.
    bl_label = "OzyMesh (.ozy)"         # Display name in the interface.
    bl_options = {'REGISTER'}
    
    filename_ext = ".ozy"
    filter_glob: StringProperty(
        default='*.ozy',
        options={'HIDDEN'}
    )
    
    def execute(self, context):
        for ob in bpy.context.selected_objects:
            save_ozymesh(ob, ob.matrix_world.copy(), self.filepath)

        return {'FINISHED'}

def menu_func(self, context):
    self.layout.operator(MeshExporter.bl_idname)

def register():    
    bpy.utils.register_class(MeshExporter)
    #bpy.types.TOPBAR_MT_file_export.append(menu_func)

def unregister():
    bpy.utils.unregister_class(MeshExporter)
    #bpy.types.TOPBAR_MT_file_export.remove(menu_func)
    
if __name__ == '__main__':
    register()