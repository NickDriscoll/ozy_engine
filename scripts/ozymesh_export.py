bl_info = {
    "name" : "OzyMesh exporter",
    "blender" : (2, 80 ,0),
    "category" : "Export"
}

import bpy
import bmesh
import struct
from bpy_extras.io_utils import ImportHelper
from bpy.props import StringProperty
from mathutils import Matrix, Vector

def len_as_u32(inp, type_size):
    return bytearray((len(inp) * type_size).to_bytes(4, "little"))

def write_pascal_strings(file, strs):
    for s in strs:
        file.write(len_as_u32(s, 1))
        file.write(bytearray(s, 'utf-8'))
        
def show_message_box(message = "", title = "Message Box", icon = 'INFO'):
    def draw(self, context):
        self.layout.label(text=message)
    bpy.context.window_manager.popup_menu(draw, title = title, icon = icon)

class Exporter(bpy.types.Operator, ImportHelper):
    """Export selection to OzyMesh file (.ozy)"""      # Use this as a tooltip for menu items and buttons.
    bl_idname = "ozymesh.exporter"        # Unique identifier for buttons and menu items to reference.
    bl_label = "OzyMesh (.ozy)"         # Display name in the interface.
    bl_options = {'REGISTER'}
    
    filename_ext = ".ozy"
    filter_glob: StringProperty(
        default='*.ozy',
        options={'HIDDEN'}
    )
    
    def execute(self, context):
        vertex_index_map = {} #Dict elements are (vertex, u16)
        index_buffer = []
        names = []
        texture_names = []
        origins = []
        geo_boundaries = [0]
        current_index = 0
        
        #Collect relevant data about meshes
        for ob in bpy.context.selected_objects:
            print("Serializing %s" % ob.name) 
            
            normal_matrix = ob.matrix_world.to_3x3()
            normal_matrix.invert()
            normal_matrix = normal_matrix.to_4x4()
            normal_matrix.transpose()
            
            names.append(ob.data.name)
            
            if not ob.active_material:
                show_message_box("\"%s\" needs to have an active material." % ob.data.name, "Unable to export OzyMesh", 'ERROR')
                return { "CANCELLED" }
            texture_names.append(ob.active_material.name)
            
            origin = ob.matrix_world @ Vector((0.0, 0.0, 0.0, 1.0))
            origins.append((origin.x, origin.y, origin.z))            

            ob.data.calc_tangents() #Have blender calculate the tangent and normal vectors

            #Create local triangulated mesh
            me = bmesh.new()
            me.from_mesh(ob.data)
            uv_lay = me.loops.layers.uv.active
            for face in me.calc_loop_triangles():
                for loop in face:
                    pos = ob.matrix_world @ loop.vert.co #Transform into world space
                    uv = loop[uv_lay].uv
                    
                    tangent = normal_matrix @ loop.calc_tangent()
                    normal = normal_matrix @ -loop.calc_normal() #This has to be negated for some reason
                    bitangent = tangent.cross(normal)
                    
                    tangent.normalize()
                    bitangent.normalize()
                    normal.normalize()
                    
                    #Construct the potential vertex
                    potential_vertex = (pos.x, pos.y, pos.z,
                                        tangent.x, tangent.y, tangent.z,
                                        bitangent.x, bitangent.y, bitangent.z,
                                        normal.x, normal.y, normal.z,
                                        uv.x, uv.y)
            
                    #Compute size of a single vertex
                    vertex_elements = len(potential_vertex)
                    
                    if potential_vertex in vertex_index_map:
                        index_buffer.append(vertex_index_map[potential_vertex])
                    else:
                        vertex_index_map[potential_vertex] = current_index
                        index_buffer.append(current_index)
                        current_index += 1
            geo_boundaries.append(geo_boundaries[len(geo_boundaries) - 1] + len(me.faces) * 3)

        #Write the data to a file
        filepath = self.filepath
        output = open(filepath, "wb")
        
        #Write the number of meshes
        output.write(len_as_u32(names, 1))
        
        #Write the geo_boundaries array
        u16_buffers = [geo_boundaries]
        for buf in u16_buffers:
            for element in buf:
                output.write(bytearray(element.to_bytes(2, "little")))
            
        #Write the names as pascal-strings
        write_pascal_strings(output, names)
        write_pascal_strings(output, texture_names)
        
        #Write the origins of the meshes
        for origin in origins:
            for i in range(0, 3):
                output.write(bytearray(struct.pack('f', origin[i])))
        
        #Write the vertex data
        output.write(len_as_u32(vertex_index_map, vertex_elements * 4))
        for vertex in list(vertex_index_map):
            for i in range(0, vertex_elements):
                output.write(bytearray(struct.pack('f', vertex[i])))
                
        #Write the index data
        output.write(len_as_u32(index_buffer, 2))    
        for index in index_buffer:
            output.write(index.to_bytes(2, "little"))        
            
        output.close()

        print("Successfully saved mesh to %s" % filepath)
        return {'FINISHED'}

def menu_func(self, context):
    self.layout.operator(Exporter.bl_idname)

def register():    
    bpy.utils.register_class(Exporter)
    bpy.types.TOPBAR_MT_file_export.append(menu_func)

def unregister():
    bpy.utils.unregister_class(Exporter)
    bpy.types.TOPBAR_MT_file_export.remove(menu_func)
    
if __name__ == '__main__':
    register()