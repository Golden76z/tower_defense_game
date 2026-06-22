import os

obj_path = "assets/models/sniper_tower.obj"
mtl_path = "assets/models/sniper_tower.mtl"

print(f"Checking if paths exist: OBJ: {os.path.exists(obj_path)}, MTL: {os.path.exists(mtl_path)}")

with open(obj_path, "r") as f:
    lines = f.readlines()

print(f"Total lines in OBJ: {len(lines)}")
usemtl_lines = [l.strip() for l in lines if l.startswith("usemtl")]
v_lines = [l.strip() for l in lines if l.startswith("v ")]
vt_lines = [l.strip() for l in lines if l.startswith("vt ")]
vn_lines = [l.strip() for l in lines if l.startswith("vn ")]
f_lines = [l.strip() for l in lines if l.startswith("f ")]

print(f"v lines: {len(v_lines)}")
print(f"vt lines: {len(vt_lines)}")
print(f"vn lines: {len(vn_lines)}")
print(f"f lines: {len(f_lines)}")
print(f"usemtl lines: {len(usemtl_lines)}")
print("First 10 usemtl occurrences:")
for x in usemtl_lines[:10]:
    print("  ", x)
print("First 5 f lines:")
for x in f_lines[:5]:
    print("  ", x)
