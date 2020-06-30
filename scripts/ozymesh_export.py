bl_info = {
    "name" : "OzyMesh exporter",
    "blender" : (2, 80 ,0),
    "category" : "Export"
}

import bpy
import struct
from bpy_extras.io_utils import ImportHelper
from mathutils import Matrix

def len_as_u32(inp, type_size):
    return bytearray((len(inp) * type_size).to_bytes(4, "little"))

def write_pascal_strings(file, strs):
    for s in strs:
        file.write(len_as_u32(s, 1))
        file.write(bytearray(s, 'utf-8'))

class Exporter(bpy.types.Operator, ImportHelper):
    """OzyMesh exporter"""      # Use this as a tooltip for menu items and buttons.
    bl_idname = "ozymesh.exporter"        # Unique identifier for buttons and menu items to reference.
    bl_label = "Export as OzyMesh"         # Display name in the interface.
    bl_options = {'REGISTER'}
    
    def execute(self, context):
        vertex_index_map = {} #Dict elements are ((f32, f32, f32, f32, f32), u16)
        index_buffer = []
        names = []
        texture_names = []
        geo_boundaries = [0]
        current_index = 0
        
        #Collect relevant data about meshes
        #for mesh in [bpy.data.meshes['Turret'], bpy.data.meshes['Hull'], bpy.data.meshes['Tread0'], bpy.data.meshes['Tread1'], bpy.data.meshes['Barrel']]:
        for mesh in bpy.data.meshes:
            world_matrix = bpy.data.objects[mesh.name].matrix_world
            z_to_y = Matrix(((1.0, 0.0, 0.0, 0.0),
                             (0.0, 0.0, 1.0, 0.0),
                             (0.0, -1.0, 0.0, 0.0),
                             (0.0, 0.0, 0.0, 1.0)))
            
            #Assuming there's only one UV map
            uv_data = mesh.uv_layers[0].data
            
            names.append(mesh.name)
            texture_names.append(bpy.data.objects[mesh.name].active_material.name)
            
            for face in mesh.polygons:
                for i in face.loop_indices:
                    vert_index = mesh.loops[i].vertex_index
                    pos = z_to_y @ world_matrix @ mesh.vertices[vert_index].co #Transform into world space and switch y and z axes
                    uvs = uv_data[i].uv
                    potential_vertex = (pos.x, pos.y, pos.z, uvs.x, uvs.y)
                    
                    if potential_vertex in vertex_index_map:
                        index_buffer.append(vertex_index_map[potential_vertex])
                    else:
                        vertex_index_map[potential_vertex] = current_index
                        index_buffer.append(current_index)
                        current_index += 1
            geo_boundaries.append(geo_boundaries[len(geo_boundaries) - 1] + len(mesh.polygons) * 3)

        #Write the data to a file
        output = open(self.filepath, "wb")
        
        #Write the number of meshes
        output.write(len_as_u32(names, 1))
        
        #Write the geo_boundaries array
        for geo in geo_boundaries:
            output.write(bytearray(geo.to_bytes(2, "little")))
            
        #Write the names and texture_names as pascal-strings
        write_pascal_strings(output, names)
        write_pascal_strings(output, texture_names)
            
        #Write the vertex data
        output.write(len_as_u32(vertex_index_map, 20))		#Length of vertices block in bytes
        for vertex in list(vertex_index_map):
            for i in range(0, 5):
                output.write(bytearray(struct.pack('f', vertex[i])))
                
        #Write the index data
        output.write(len_as_u32(index_buffer, 2))    
        for index in index_buffer:
            output.write(index.to_bytes(2, "little"))
        
            
        output.close()
        return {'FINISHED'}

def register():
    bpy.utils.register_class(Exporter)


def unregister():
    bpy.utils.unregister_class(Exporter)
    
if __name__ == '__main__':
    register()