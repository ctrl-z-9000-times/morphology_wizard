import random

with open("points_with_header.csv", "wt") as f:
    f.write("x,y,z\n")
    for n in range(100):
        x = random.uniform(-50, 50)
        y = random.uniform(-50, 50)
        z = random.uniform(-50, 50)
        f.write(f"{x},{y},{z}\n")

with open("points_no_header.csv", "wt") as f:
    f.write("x,y,z\n")
    for n in range(100):
        x = random.uniform(-50, 50)
        y = random.uniform(150, 200)
        z = random.uniform(-50, 50)
        f.write(f"{x},{y},{z}\n")
