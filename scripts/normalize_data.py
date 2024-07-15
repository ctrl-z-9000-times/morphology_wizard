
with open("quaddiameter_P.txt", 'rt') as f:
    P = f.read()

with open("quaddiameter_ldend.txt", 'rt') as f:
    ldend = f.read()

ldend = [float(x) for x in ldend.split()]
P = [[float(x) for x in line.split()] for line in P.split('\n')]

for i in range(len(ldend)):
    P[i][0] *= ldend[i] ** 2
    P[i][1] *= ldend[i]

P = '\n'.join(' '.join(str(x) for x in row) for row in P)

with open("quaddiameter_P_normalized.txt", 'wt') as f:
    f.write(P)
