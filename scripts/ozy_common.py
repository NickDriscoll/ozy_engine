import bpy
import bmesh
import struct
from mathutils import Matrix, Vector


#Returns bytearray that is a u32 representing the size of inp in bytes
def size_as_u32(inp, type_size):
    return bytearray((len(inp) * type_size).to_bytes(4, "little"))

def write_vector(out_file, vector):
    for number in vector:
        out_file.write(bytearray(struct.pack('f', number)))

def write_pascal_strings(file, strs):
    for s in strs:
        file.write(size_as_u32(s, 1))
        file.write(bytearray(s, 'utf-8'))
        
def show_message_box(message = "", title = "Message Box", icon = 'INFO'):
    def draw(self, context):
        self.layout.label(text=message)
    bpy.context.window_manager.popup_menu(draw, title = title, icon = icon)

def get_base_color(ob, index):
    mat = ob.material_slots[index].material
    principled = next(n for n in mat.node_tree.nodes if n.type == 'BSDF_PRINCIPLED')
    return principled.inputs["Base Color"]

#Recursive helper for get_color_buffer()
def color_map_rec(ob, map):
    for i in range(0, len(ob.material_slots)):
        base_color = get_base_color(ob, i)
        color = (base_color.default_value[0], base_color.default_value[1], base_color.default_value[2], base_color.default_value[3])
        val = "%s%i" % (ob.name, i)
        if color in map:
            map[color].append(val)
        else:
            map[color] = [val]

    #Recursively call this on all children
    for child in ob.children:
        color_map_rec(child, map)

#Returns a map of colors belonging to this object and its children
def get_color_map(ob):
    map = {}
    color_map_rec(ob, map)
    return map

def write_vertex_array_rec(ob, model_transform, color_map, vertex_index_map, index_buffer, current_index):
    #Get a copy of the object with all modifiers applied
    depsgraph = bpy.context.evaluated_depsgraph_get()
    ob_copy = ob.evaluated_get(depsgraph)
    mesh = ob_copy.data
            
    #Figure out the normal matrix
    normal_matrix = model_transform.to_3x3()
    normal_matrix.invert()
    normal_matrix = normal_matrix.to_4x4()
    normal_matrix.transpose()

    if mesh.uv_layers.active:
        mesh.calc_tangents()
        
    print("\"%s\" has %i faces" % (ob.name, len(mesh.polygons)))
    for face in mesh.polygons:
        for i in face.loop_indices:
            loop = mesh.loops[i]
            pos = model_transform @ mesh.vertices[loop.vertex_index].co
            
            if len(color_map) > 0:
                color_count = len(color_map)
                
                color_index = -1
                val = "%s%i" % (ob.name, face.material_index)
                for i, (key, value) in enumerate(color_map.items()):
                    if val in value:
                        color_index = i
                        break
                
                u = 1.0 / (2.0 * color_count) + color_index / color_count
                uvs = Vector((u, -0.5))
            else:
                uv_data = mesh.uv_layers.active.data
                uvs = uv_data[i].uv
            
            tangent = normal_matrix @ loop.tangent
            normal = normal_matrix @ loop.normal
            bitangent = normal_matrix @ loop.bitangent
            
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
        
    #Base case is when len(ob.children) == 0        
    for child in ob.children:
        current_index = write_vertex_array_rec(child, model_transform, color_map, vertex_index_map, index_buffer, current_index)
        
    return current_index

def save_ozymesh(ob, model_transform, filepath):
    vertex_index_map = {} #Dict elements are (vertex, u16)
    index_buffer = []
    color_map = {}
    texture_name = ""
    current_index = 0
    vertex_elements = 14    #The number of floats in a single
    
    mesh = ob.data
    if not ob.active_material:
        show_message_box("\"%s\" needs to have an active material." % mesh.name, "Unable to export OzyMesh", 'ERROR')
        return False
    
    #We only set one of the color_map/texture_name based on what kind of materials this model uses
    base_color = get_base_color(ob, 0)
    if len(base_color.links) == 0:
        print("Color is solid")
        color_map = get_color_map(ob) #Use original object here bc copy doesn't keep children        
    else:
        print("Color is from texture")
        texture_name = ob.active_material.name
        #Assuming there's only one UV map
        uv_data = mesh.uv_layers.active.data
    
    write_vertex_array_rec(ob, model_transform, color_map, vertex_index_map, index_buffer, current_index)

    #Write the data to a file
    output = open(filepath, "wb")

    if len(color_map) == 0:
        #Write zero byte
        output.write((0).to_bytes(1, "little"))
        
        #Write the material name as a pascal string
        write_pascal_strings(output, [texture_name])
    else:
        #Write number of colors
        output.write((len(color_map)).to_bytes(1, "little"))
        
        #Write the colors one after the other as normalized RGBA f32 values
        for color in color_map:
            for i in range(0, len(color)):
                output.write(bytearray(struct.pack('f', color[i])))
        
        
    #Write the vertex data
    output.write(size_as_u32(vertex_index_map, vertex_elements * 4))
    for vertex in list(vertex_index_map):
        for i in range(0, vertex_elements):
            output.write(bytearray(struct.pack('f', vertex[i])))
                
    #Write the index data
    output.write(size_as_u32(index_buffer, 2))
    for index in index_buffer:
        #print(index)
        output.write(index.to_bytes(2, "little"))        
            
    output.close()
    return True

class TerrainData:
    def __init__(self):
        self.vertex_index_map = {}
        self.index_buffer = []
        self.face_normals = []
        self.current_index = 0

def append_collision_to_buffers(col, terrain_data):
    for ob in col.objects:
        if ob.type != "MESH":
            continue

        if "non-collidable" in ob:
            continue
                
        #Create triangulated mesh
        me = bmesh.new()
        me.from_mesh(ob.data)
        triangles = me.calc_loop_triangles()
        num_tris = len(triangles)
        if num_tris > 100:
            print("%s is kind of large to be a collision mesh at %i tris." % (ob.name, num_tris))

        for face in triangles:
            face_verts = []
            for loop in face:
                vertex_vector = ob.matrix_world @ Vector((loop.vert.co.x, loop.vert.co.y, loop.vert.co.z, 1.0))
                face_verts.append(Vector((vertex_vector.x, vertex_vector.y, vertex_vector.z)))
                potential_vertex = (vertex_vector.x, vertex_vector.y, vertex_vector.z)
                if potential_vertex in terrain_data.vertex_index_map:
                    terrain_data.index_buffer.append(terrain_data.vertex_index_map[potential_vertex])
                else:
                    terrain_data.vertex_index_map[potential_vertex] = terrain_data.current_index
                    terrain_data.index_buffer.append(terrain_data.current_index)
                    terrain_data.current_index += 1
                            
            edge0 = face_verts[1] - face_verts[0]
            edge1 = face_verts[2] - face_verts[0]
            face_normal = edge0.cross(edge1)
            face_normal.normalize()
            terrain_data.face_normals.append(face_normal)

def save_ozyterrain(filepath, collection):
    terrain_data = TerrainData()

    for col in collection.children:
        append_collision_to_buffers(col, terrain_data)

    append_collision_to_buffers(collection, terrain_data)
        
    #Write the data to a file
    output = open(filepath, "wb")
        
    #Write the size of the vertices in the vertex block
    output.write(size_as_u32(terrain_data.vertex_index_map, 12))
        
    #Write the vertex block
    for vertex in list(terrain_data.vertex_index_map):
        write_vector(output, vertex)
        
    #Write the size of the indices in the index block
    output.write(size_as_u32(terrain_data.index_buffer, 2))
        
    #Write the index block
    for index in terrain_data.index_buffer:
        output.write(index.to_bytes(2, "little"))
                
    #Write the size of the face normals
    output.write(size_as_u32(terrain_data.face_normals, 12))
        
    #Write the face normals
    for normal in terrain_data.face_normals:
        write_vector(output, normal)
        
    output.close()