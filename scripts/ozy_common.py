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

def save_ozymesh(ob, model_transform, filepath):
    vertex_index_map = {} #Dict elements are (vertex, u16)
    index_buffer = []
    texture_name = ""
    current_index = 0
    
    mesh = ob.data
            
    #Figure out the normal matrix
    normal_matrix = model_transform.to_3x3()
    normal_matrix.invert()
    normal_matrix = normal_matrix.to_4x4()
    normal_matrix.transpose()

    blender_to_game_world = model_transform
    normal_to_game_world = normal_matrix
    
    #Assuming there's only one UV map
    uv_data = mesh.uv_layers.active.data
    
    if not ob.active_material:
        show_message_box("\"%s\" needs to have an active material." % mesh.name, "Unable to export OzyMesh", 'ERROR')
        return False
    texture_name = ob.active_material.name
    
    ob.data.calc_tangents() #Have blender calculate the tangent and normal vectors
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

    #Write the data to a file
    output = open(filepath, "wb")
            
    #Write the texture name as a pascal string
    write_pascal_strings(output, [texture_name])
        
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
    return True

def save_ozyterrain(filepath, collection):
    #We just want to export all of the triangles
    vertex_index_map = {} #Elements are ((f32, f32, f32), u16)
    face_normals = []
    index_buffer = []
        
    current_index = 0

    for col in collection.children:
        for ob in col.objects:
            if ob.type != "MESH":
                continue
                
            #Create triangulated mesh
            me = bmesh.new()
            me.from_mesh(ob.data)
            for face in me.calc_loop_triangles():
                face_verts = []
                for loop in face:
                    vertex_vector = ob.matrix_world @ Vector((loop.vert.co.x, loop.vert.co.y, loop.vert.co.z, 1.0))
                    face_verts.append(Vector((vertex_vector.x, vertex_vector.y, vertex_vector.z)))
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

    for ob in collection.objects:
        if ob.type != "MESH":
            continue
            
        #Create triangulated mesh
        me = bmesh.new()
        me.from_mesh(ob.data)
        for face in me.calc_loop_triangles():
            face_verts = []
            for loop in face:
                vertex_vector = ob.matrix_world @ Vector((loop.vert.co.x, loop.vert.co.y, loop.vert.co.z, 1.0))
                face_verts.append(Vector((vertex_vector.x, vertex_vector.y, vertex_vector.z)))
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
    output = open(filepath, "wb")
        
    #Write the size of the vertices in the vertex block
    output.write(size_as_u32(vertex_index_map, 12))
        
    #Write the vertex block
    for vertex in list(vertex_index_map):
        write_vector(output, vertex)
        
    #Write the size of the indices in the index block
    output.write(size_as_u32(index_buffer, 2))
        
    #Write the index block
    for index in index_buffer:
        output.write(index.to_bytes(2, "little"))
                
    #Write the size of the face normals
    output.write(size_as_u32(face_normals, 12))
        
    #Write the face normals
    for normal in face_normals:
        write_vector(output, normal)
        
    output.close()