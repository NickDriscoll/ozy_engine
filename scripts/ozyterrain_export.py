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

#Returns bytearray that is a u32 representing the size of inp in bytes
def size_as_u32(inp, type_size):
    return bytearray((len(inp) * type_size).to_bytes(4, "little"))

def write_float_3d(out_file, vertex3):
    for number in vertex3:
        out_file.write(bytearray(struct.pack('f', number)))

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
        ob = bpy.context.selected_objects[0]
        
        #Create triangulated mesh
        me = bmesh.new()
        me.from_mesh(ob.data)
            
        #We just want to export all of the triangles
        vertex_index_map = {} #Elements are ((f32, f32, f32), u16)
        face_normals = []
        index_buffer = []
            
        current_index = 0
        for face in me.calc_loop_triangles():
            face_verts = []
            for loop in face:
                vertex_vector = ob.matrix_world @ Vector((loop.vert.co.x, loop.vert.co.y, loop.vert.co.z))
                face_verts.append(vertex_vector)
                potential_vertex = (vertex_vector.x, vertex_vector.y, vertex_vector.z)
                if potential_vertex in vertex_index_map:
                    index_buffer.append(vertex_index_map[potential_vertex])
                else:
                    vertex_index_map[potential_vertex] = current_index
                    index_buffer.append(current_index)
                    current_index += 1
                    
            edge0 = face_verts[1] - face_verts[0]
            edge1 = face_verts[2] - face_verts[0]
            face_normal = edge0.cross(edge1)
            face_normal.normalize()
            face_normals.append(face_normal)
        
        #Write the data to a file
        filepath = self.filepath
        output = open(filepath, "wb")
        
        #Write the size of the vertices in the vertex block
        output.write(size_as_u32(vertex_index_map, 12))
        
        #Write the vertex block
        for vertex in list(vertex_index_map):
            write_float_3d(output, vertex)
        
        #Write the size of the indices in the index block
        output.write(size_as_u32(index_buffer, 2))
        
        #Write the index block
        for index in index_buffer:
            output.write(index.to_bytes(2, "little"))
                
        #Write the size of the face normals
        output.write(size_as_u32(face_normals, 12))
        
        #Write the face normals
        for normal in face_normals:
            write_float_3d(output, normal)
        
        output.close()
        print("Finished exporting collision mesh.")
        return {'FINISHED'}


def register():    
    bpy.utils.register_class(TerrainExporter)
    
def unregister():
    bpy.utils.unregister_class(TerrainExporter)
    
if __name__ == '__main__':
    register()