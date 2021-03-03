
#Returns bytearray that is a u32 representing the size of inp in bytes
def size_as_u32(inp, type_size):
    return bytearray((len(inp) * type_size).to_bytes(4, "little"))

def write_float_3d(out_file, vertex3):
    for number in vertex3:
        out_file.write(bytearray(struct.pack('f', number)))

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