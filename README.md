# Douconel
Douconel is a Rust library designed to facilitate the creation, manipulation, and analysis of complex 3D mesh data structures through a Doubly Connected Edge List (DCEL). This data structure is particularly suited for geometric and topological operations, making it an essential tool for computational geometry, computer graphics, and mesh processing tasks.

## About DCEL
The DCEL data structure, also known as a half-edge data structure, is designed to efficiently store a mesh's vertices, edges, and faces while maintaining easy access to their relationships. Each entity (vertex, edge, face) in a DCEL is connected, allowing for efficient traversal and manipulation of the mesh. This structure is particularly useful for algorithms that require frequent access to the entities adjacent to any given entity, such as polygon triangulation, mesh refinement, and geometric queries.

## Features

including but not limited to

1. Storing DCEL
2. importing a mesh from stl and storing it as DCEL
3. custom data types for vertices, edges, and faces
4. great api for basic DCEL operations
5. great api for basic geometry operations, if defined positions and normals on the vertices
6. compatibility with petgraph and bevy

## TODO
While Douconel is already capable of handling a variety of tasks related to 3D mesh processing, the roadmap includes several enhancements to broaden its applicability:

1. Support for Additional File Formats: Importing from and exporting to popular mesh formats such as .obj and .stl.
2. Mesh Refinement: Implementing different types of mesh refinement techniques for various applications.
3. Export to Petgraph: Enabling the export of mesh data to Petgraph structures for advanced graph analysis.
4. export to bevy: enabling importing the dcel as a mesh into bevy..
5. Export to OBJ/STL: Facilitating the export of meshes to .obj and .stl formats for compatibility with other tools and libraries in the 3D processing ecosystem.

## Contributions
Contributions to Douconel are welcome! Whether it's adding new features, improving existing ones, or fixing bugs, your help is invaluable. Please refer to the repository's contribution guidelines for more information on how to contribute.

## License
Douconel is distributed under the MIT license. See the LICENSE file in the repository for more details.
