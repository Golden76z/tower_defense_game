import os

obj_path = "assets/models/sniper_tower.obj"

with open(obj_path, "r") as f:
    lines = f.readlines()

xs, ys, zs = [], [], []
for line in lines:
    if line.startswith("v "):
        parts = line.split()
        xs.append(float(parts[1]))
        ys.append(float(parts[2]))
        zs.append(float(parts[3]))

print(f"Bounding box:")
print(f"  X: min={min(xs):.4f}, max={max(xs):.4f}, range={max(xs)-min(xs):.4f}")
print(f"  Y: min={min(ys):.4f}, max={max(ys):.4f}, range={max(ys)-min(ys):.4f}")
print(f"  Z: min={min(zs):.4f}, max={max(zs):.4f}, range={max(zs)-min(zs):.4f}")
