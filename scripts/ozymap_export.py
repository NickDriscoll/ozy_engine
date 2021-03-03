bl_info = {
    "name" : "OzyMap exporter",
    "blender" : (2, 80 ,0),
    "category" : "Export"
}

import bpy
import bmesh
import struct
from bpy_extras.io_utils import ImportHelper
from bpy.props import StringProperty
from mathutils import Matrix, Vector

from ozy_common import *

class MapExporter(bpy.types.Operator, ImportHelper):
    """Export selection to OzyMesh file (.ozy)"""      # Use this as a tooltip for menu items and buttons.
    bl_idname = "ozymap.exporter"        # Unique identifier for buttons and menu items to reference.
    bl_label = "OzyMap (.ozm)"         # Display name in the interface.
    bl_options = {'REGISTER'}
    
    filename_ext = ".ozm"
    filter_glob: StringProperty(
        default='*.ozm',
        options={'HIDDEN'}
    )
    
    def execute(self, context):            
        map_collection = bpy.context.scene.collection.children[0]

        return {'FINISHED'}

def menu_func(self, context):
    self.layout.operator(MapExporter.bl_idname)

def register():    
    bpy.utils.register_class(MapExporter)
    bpy.types.TOPBAR_MT_file_export.append(menu_func)
    
def unregister():
    bpy.utils.unregister_class(MapExporter)
    bpy.types.TOPBAR_MT_file_export.remove(menu_func)
    
if __name__ == '__main__':
    register()