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

COLLISION_COLLECTION_NAME = "collision"
GRAPHICS_COLLECTION_NAME = "visible"
BOTH_COLLECTION_NAME = "both"

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

        main_collection = bpy.context.scene.collection.children[0]
        visible_collections = [
            main_collection.children.get(BOTH_COLLECTION_NAME),
            main_collection.children.get(GRAPHICS_COLLECTION_NAME)
        ]
        collision_collections = [            
            main_collection.children.get(BOTH_COLLECTION_NAME),
            main_collection.children.get(COLLISION_COLLECTION_NAME)
        ]

        level_name = main_collection.name
            
        #Compute the game's base directory
        directory = ""
        splits = self.filepath.split('\\')
        for i in range(0, len(splits) - 2):
            directory += "%s/" % splits[i]

        #Build the map between material names and objects that use that material
        material_object_maps = []
        for collection in visible_collections:
            mat_ob_map = {}
            for i in range(0, len(collection.objects)):
                print("%s" % collection.objects[i].name)
                mat_name = collection.objects[i].active_material.name
                if mat_name in mat_ob_map:
                    mat_ob_map[mat_name].append(i)
                else:
                    mat_ob_map[mat_name] = [i]
            material_object_maps.append(mat_ob_map)
            
        #open the map file
        map_file = open(self.filepath, "wb")

        #For each material present, save an ozymesh containing all geometry using that material
        for col_idx in range(0, len(material_object_maps)):
            for mat, ob_indices in material_object_maps[col_idx].items():
                #Ozymesh data for this material collection
                mesh_data = MeshData()

                #We're just making all of these objects into one big VAO
                #Get the uv velocity
                uv_velocity = Vector((0.0, 0.0))
                    
                for j in ob_indices:
                    ob = visible_collections[col_idx].objects[j]
                    write_object_to_mesh_data(ob, ob.matrix_world.copy(), mesh_data)
                    if "u velocity" in ob:
                        uv_velocity.x = ob["u velocity"]
                    if "v velocity" in ob:
                        uv_velocity.y = ob["v velocity"]

                #record this ozymesh in the lvl file
                ozyname = "%s_%s_terrain.ozy" % (level_name, mat)
                filename = "%s/models/%s" % (directory, ozyname)
                write_pascal_strings(map_file, [ozyname])
                map_file.write(bytearray((1).to_bytes(4, "little")))
                for v in IDENTITY_MATRIX:
                    write_vector(map_file, v)

                #Now to save the data to a file
                output = open(filename, "wb")

                #Write the material name as a pascal string
                #Write zero byte
                output.write((0).to_bytes(1, "little"))
                write_pascal_strings(output, [mat])
                
                #Write the uv velocity
                for i in range(0, 2):
                    output.write(bytearray(struct.pack('f', uv_velocity[i])))
                    
                #Write the vertex data
                vertex_elements = 14
                output.write(size_as_u32(mesh_data.vertex_index_map, vertex_elements * 4))
                for vertex in list(mesh_data.vertex_index_map):
                    for i in range(0, vertex_elements):
                        output.write(bytearray(struct.pack('f', vertex[i])))
                            
                #Write the index data
                output.write(size_as_u32(mesh_data.index_buffer, 2))    
                for index in mesh_data.index_buffer:
                    output.write(index.to_bytes(2, "little"))        
                        
                output.close()
            
        #instanced meshes
        for v_collection in visible_collections:
            for collection in v_collection.children:
                filename = "%s.ozy" % collection.name
                filepath = "%s/models/%s" % (directory, filename)
                ob = collection.objects[0]
                save_ozymesh(ob, IDENTITY_MATRIX, filepath) #Save an ozymesh for this model
                    
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
        terrain_data = TerrainData()
        for collection in collision_collections:
            collection_to_terrain_data(collection, terrain_data)
            
        filepath = "%s/models/%s.ozt" % (directory, main_collection.name)
        write_ozyterrain_file(filepath, terrain_data)
        print("Finished exporting %s.ozt" % main_collection.name)

        print("Finished exporting \"%s\" to \"%s\"" % (main_collection.name, self.filepath))
        map_file.close()
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