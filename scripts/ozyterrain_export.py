bl_info = {
    "name" : "OzyTerrain exporter",
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

'''
Vertex:
    ------
['__class__',
'__delattr__',
'__delitem__',
'__dir__', '__doc__', '__eq__', '__format__', '__ge__', '__getattribute__', '__getitem__', '__gt__',
'__hash__', '__init__', '__init_subclass__', '__le__', '__lt__', '__ne__', '__new__', '__reduce__', '__reduce_ex__',
'__repr__', '__setattr__', '__setitem__', '__sizeof__', '__str__', '__subclasshook__', 'calc_edge_angle', 'calc_shell_factor',
'co', 'copy_from', 'copy_from_face_interp', 'copy_from_vert_interp', 'hide', 'hide_set', 'index', 'is_boundary', 'is_manifold',
'is_valid', 'is_wire', 'link_edges', 'link_faces', 'link_loops', 'normal', 'normal_update', 'select', 'select_set', 'tag']
'''

'''
Loop:
    ------
['__class__', '__delattr__', '__delitem__', '__dir__', '__doc__', '__eq__', '__format__',
'__ge__', '__getattribute__', '__getitem__', '__gt__', '__hash__', '__init__', '__init_subclass__',
'__le__', '__lt__', '__ne__', '__new__', '__reduce__', '__reduce_ex__', '__repr__', '__setattr__',
'__setitem__', '__sizeof__', '__str__', '__subclasshook__', 'calc_angle', 'calc_normal', 'calc_tangent',
'copy_from', 'copy_from_face_interp', 'edge', 'face', 'index', 'is_convex', 'is_valid', 'link_loop_next',
'link_loop_prev', 'link_loop_radial_next', 'link_loop_radial_prev', 'link_loops', 'tag', 'vert']
'''

'''
Face:
    ------
['__add__', '__class__', '__contains__', '__delattr__', '__dir__', '__doc__', '__eq__',
'__format__', '__ge__', '__getattribute__', '__getitem__', '__getnewargs__', '__gt__',
'__hash__', '__init__', '__init_subclass__', '__iter__', '__le__', '__len__', '__lt__',
'__mul__', '__ne__', '__new__', '__reduce__', '__reduce_ex__', '__repr__', '__rmul__',
'__setattr__', '__sizeof__', '__str__', '__subclasshook__', 'count', 'index']
'''


class TerrainExporter(bpy.types.Operator, ImportHelper):
    """Export selection to OzyMesh file (.ozy)"""      # Use this as a tooltip for menu items and buttons.
    bl_idname = "ozyterrain.exporter"        # Unique identifier for buttons and menu items to reference.
    bl_label = "OzyTerrain (.ozt)"         # Display name in the interface.
    bl_options = {'REGISTER'}
    
    filename_ext = ".ozt"
    filter_glob: StringProperty(
        default='*.ozt',
        options={'HIDDEN'}
    )
    
    def execute(self, context):            
        #We just want to export all of the triangles
        save_ozyterrain(self.filepath)
        print("Finished exporting collision mesh.")
        return {'FINISHED'}

def menu_func(self, context):
    self.layout.operator(TerrainExporter.bl_idname)

def register():    
    bpy.utils.register_class(TerrainExporter)
    bpy.types.TOPBAR_MT_file_export.append(menu_func)
    
def unregister():
    bpy.utils.unregister_class(TerrainExporter)
    bpy.types.TOPBAR_MT_file_export.remove(menu_func)
    
if __name__ == '__main__':
    register()