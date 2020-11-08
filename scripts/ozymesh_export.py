bl_info = {
    "name" : "OzyMesh exporter",
    "blender" : (2, 80 ,0),
    "category" : "Export"
}

import bpy
import struct
from bpy_extras.io_utils import ImportHelper
from mathutils import Matrix, Vector

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
        vertex_index_map = {} #Dict elements are (vertex, u16)
        index_buffer = []
        names = []
        texture_names = []
        node_ids = []
        parent_ids = []
        origins = []
        geo_boundaries = [0]
        current_index = 0
        
        #Collect relevant data about meshes
        for ob in bpy.context.selected_objects:
            print("Serializing %s" % ob.name)
            mesh = ob.data
            
            normal_matrix = ob.matrix_world.to_3x3()
            normal_matrix.invert()
            normal_matrix = normal_matrix.to_4x4()
            normal_matrix.transpose()
            z_to_y = Matrix(((1.0, 0.0, 0.0, 0.0),
                             (0.0, 0.0, 1.0, 0.0),
                             (0.0, 1.0, 0.0, 0.0),
                             (0.0, 0.0, 0.0, 1.0)))

            blender_to_game_world = z_to_y @ ob.matrix_world
            normal_to_game_world = z_to_y @ normal_matrix
            
            #Assuming there's only one UV map
            uv_data = mesh.uv_layers[0].data
            
            names.append(mesh.name)
            
            if not ob.active_material:
                print("%s does not have an active material." % mesh.name)
                return { "CANCELLED" }                
            texture_names.append(ob.active_material.name)
            
            origin = blender_to_game_world @ Vector((0.0, 0.0, 0.0, 1.0))
            origins.append((origin.x, origin.y, origin.z))
            
            if 'node_id' not in ob:
                print("%s does not have the custom property \"%s\" defined." % (mesh.name, 'node_id'))
                return { "CANCELLED" }
            node_ids.append(int(ob['node_id']))
            
            if 'parent_id' not in ob:
                parent_ids.append(0)
            else:
                parent_ids.append(int(ob['parent_id']))
            
            ob.data.calc_tangents() #Have blender calculate the tangent and normal vectors
            for face in mesh.polygons:
                for i in face.loop_indices:
                    loop = mesh.loops[i]
                    pos = blender_to_game_world @ mesh.vertices[loop.vertex_index].co #Transform into world space and switch y and z axes
                    uvs = uv_data[i].uv
                    
                    tangent = normal_to_game_world @ loop.tangent
                    normal = normal_to_game_world @ loop.normal
                    bitangent = normal_to_game_world @ loop.bitangent
                    
                    tangent.normalize()
                    bitangent.normalize()
                    normal.normalize()
                    
                    #Construct the potential vertex
                    potential_vertex = (pos.x, pos.y, pos.z,
                                        tangent.x, tangent.y, tangent.z,
                                        bitangent.x, bitangent.y, bitangent.z,
                                        normal.x, normal.y, normal.z,
                                        uvs.x, uvs.y)
            
                    #Compute size of a single vertex
                    vertex_elements = len(potential_vertex)
                    
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
        
        #Write the geo_boundaries array, node_ids, and parent_ids
        u16_buffers = [geo_boundaries, node_ids, parent_ids]
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
        return {'FINISHED'}

def register():
    bpy.utils.register_class(Exporter)

def unregister():
    bpy.utils.unregister_class(Exporter)
    
if __name__ == '__main__':
    register()