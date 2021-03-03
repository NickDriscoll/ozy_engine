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
    """Export selection to OzyMap file (.ozm)"""      # Use this as a tooltip for menu items and buttons.
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
            
        #Compute the game's base directory
        directory = ""
        splits = self.filepath.split('\\')
        for i in range(0, len(splits) - 1):
            directory += "%s/" % splits[i]
            
        #open the map file itself
        map_file = open(self.filepath,"wb")
        
        #non-instanced objects
        for ob in map_collection.objects:
            print("Serializing %s" % ob.name)
            filename = "%s.ozy" % ob.name
            filepath = "%s/%s" % (directory, filename)            
            save_ozymesh(ob, filepath)
            
            #Write to the map file
            write_pascal_strings(map_file, filename)
            map_file.write(bytearray((1).to_bytes(4, "little")))
            
        #instanced meshes
        ignored_collections = ["Lights"]
        for collection in map_collection.children:
            if collection.name not in ignored_collections:
                filepath = "%s/%s.ozy" % (directory, collection.objects[0].name)
                save_ozymesh(collection.objects[0], filepath) #Save an ozymesh for this model
                
                #Save the transforms for each instance
                #for ob in collection.objects:
                
        #Now for the collision data
        filepath = "%s/%s.ozt" % (directory, map_collection.name)
        save_ozyterrain(filepath)
        
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