# Runar Map File Format
8 Bytes Magic Number
4 Bytes Major Version
4 Bytes Minor Version
4 Bytes Patch
8 Bytes Number of containers
[
    16 Bytes Container UUID
    8 Bytes Container Offset
] x Number of Containers

MapContainerIndex
8 Bytes Number of Items
[
    8 Bytes Item UUID
    8 Bytes Container Index
]