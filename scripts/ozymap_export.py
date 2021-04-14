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
    """Export selection to OzyMap file (.lvl)"""      # Use this as a tooltip for menu items and buttons.
    bl_idname = "ozymap.exporter"        # Unique identifier for buttons and menu items to reference.
    bl_label = "OzyMap (.lvl)"         # Display name in the interface.
    bl_options = {'REGISTER'}
    
    filename_ext = ".lvl"
    filter_glob: StringProperty(
        default='*.lvl',
        options={'HIDDEN'}
    )
    
    def execute(self, context):
        IDENTITY_MATRIX = Matrix.Identity(4)        #Standard 4x4 identity

        #Exporting everything in the first collection of the blend file
        map_collection = bpy.context.scene.collection.children[0]
        level_name = map_collection.name
            
        #Compute the game's base directory
        directory = ""
        splits = self.filepath.split('\\')
        for i in range(0, len(splits) - 2):
            directory += "%s/" % splits[i]

        #Build the map between material names and objects that use that material
        material_object_map = {}
        for i in range(0, len(map_collection.objects)):
            mat_name = map_collection.objects[i].active_material.name
            if mat_name in material_object_map:
                material_object_map[mat_name].append(i)
            else:
                material_object_map[mat_name] = [i]
            
        #open the map file
        map_file = open(self.filepath, "wb")

        #For each material present, save an ozymesh containing all geometry using that material
        for mat, ob_indices in material_object_map.items():
            #Ozymesh data for this material collection            
            vertex_index_map = {} #Dict elements are (vertex, u16)
            index_buffer = []
            current_index = 0

            #We're just making all of these objects into one big VAO
            for i in ob_indices:
                ob = map_collection.objects[i]
                mesh = ob.data
                model_transform = ob.matrix_world.copy()
                
                #Figure out the normal matrix
                normal_matrix = model_transform.to_3x3()
                normal_matrix.invert()
                normal_matrix = normal_matrix.to_4x4()
                normal_matrix.transpose()

                blender_to_game_world = model_transform
                normal_to_game_world = normal_matrix
    
                #Assuming there's only one UV map
                uv_data = mesh.uv_layers.active.data
                
                mesh.calc_tangents() #Have Blender calculate the TBN vectors
                for face in mesh.polygons:
                    for i in face.loop_indices:
                        loop = mesh.loops[i]
                        pos = blender_to_game_world @ mesh.vertices[loop.vertex_index].co
                        uvs = uv_data[i].uv
                        
                        tangent = normal_to_game_world @ loop.tangent
                        normal = normal_to_game_world @ loop.normal
                        bitangent = normal_to_game_world @ loop.bitangent
                        
                        #Just making sure they're normalized
                        tangent.normalize()
                        bitangent.normalize()
                        normal.normalize()
                        
                        #Construct the potential vertex
                        potential_vertex = (pos.x, pos.y, pos.z,
                                            tangent.x, tangent.y, tangent.z,
                                            bitangent.x, bitangent.y, bitangent.z,
                                            normal.x, normal.y, normal.z,
                                            uvs.x, -uvs.y)                                        
                        
                        #Compute size of a single vertex
                        vertex_elements = len(potential_vertex)
                                
                        #Check if we've already seen this vertex
                        if potential_vertex in vertex_index_map:
                            index_buffer.append(vertex_index_map[potential_vertex])
                        else:
                            vertex_index_map[potential_vertex] = current_index
                            index_buffer.append(current_index)
                            current_index += 1

            #Write record this ozymesh in the lvl file
            ozyname = "%s_%s_terrain.ozy" % (level_name, mat)
            filename = "%s/models/%s" % (directory, ozyname)
            write_pascal_strings(map_file, [ozyname])
            map_file.write(bytearray((1).to_bytes(4, "little")))
            for v in IDENTITY_MATRIX:
                write_vector(map_file, v)

            #Now to save the data to a file
            output = open(filename, "wb")

            #Write the material name as a pascal string
            write_pascal_strings(output, [mat])
                
            #Write the vertex data
            output.write(size_as_u32(vertex_index_map, vertex_elements * 4))
            for vertex in list(vertex_index_map):
                for i in range(0, vertex_elements):
                    output.write(bytearray(struct.pack('f', vertex[i])))
                        
            #Write the index data
            output.write(size_as_u32(index_buffer, 2))    
            for index in index_buffer:
                output.write(index.to_bytes(2, "little"))        
                    
            output.close()
            
        #instanced meshes
        for collection in map_collection.children:
            filename = "%s.ozy" % collection.name
            filepath = "%s/models/%s" % (directory, filename)
            save_ozymesh(collection.objects[0], IDENTITY_MATRIX, filepath) #Save an ozymesh for this model
                
            #Save the transforms for each instance
            write_pascal_strings(map_file, [filename])
            map_file.write(bytearray((len(collection.objects)).to_bytes(4, "little")))
            for ob in collection.objects:
                #Make matrix column-major
                world_mat = ob.matrix_world.copy()
                world_mat.transpose()
                for v in world_mat:
                    write_vector(map_file, v)
                
        #Now for the collision data
        filepath = "%s/models/%s.ozt" % (directory, map_collection.name)
        save_ozyterrain(filepath, map_collection)
        print("Finished exporting %s.ozt" % map_collection.name)

        print("Finished exporting \"%s\" to \"%s\"" % (map_collection.name, self.filepath))
        map_file.close()
        return {'FINISHED'}

def menu_func(self, context):
    self.layout.operator(MapExporter.bl_idname)

def register():    
    bpy.utils.register_class(MapExporter)
    #bpy.types.TOPBAR_MT_file_export.append(menu_func)
    
def unregister():
    bpy.utils.unregister_class(MapExporter)
    #bpy.types.TOPBAR_MT_file_export.remove(menu_func)
    
if __name__ == '__main__':
    register()