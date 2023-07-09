# Notice
The goal of this project has changed signifigantly, and this project now represents a modernization of the beta client rather than a POC for minecraft algorithms, so the following sections are partially wrong. There is now heavy focus on resourcepacks, blockstates, data-driven design, and maybe modularity.

# Orange-rs

A minecraft beta 1.7.3 clone written in rust

This project might lead me into making tools for loading data such as resourcepacks, world types, and maybe even datapacks

I hope this project can serve not only as a learning tool between me, rust, and asynchonous code; 
but also as a basis to represent the algorithms needed to implement minecraft on any language, or even as a library to be expanded upon by others


# Current Progress
Very primitive chunk generation and rendering, much bad code design

# Todo
* Generate chunks and manage server operations on a separate thread
* Introduce transparency, chunk dirtying, and remeshing
* Add lighting
* Fix queue issue in which the queue is always full (and not clearing a continuously building queue)
* Finish the list of blocks
* Add Items
* Add Entities
* Add Tile Entities
* Add collision
* Add "Screens"
* Add gui's and containers
* Add world saving and loading (Anvil Format)
* Add networking capabilities

# Possible features
Things I might think about adding after reaching the goal of vanilla b1.7.3 with anvil format, list from most likely at the top, to least likely at the bottom
* Authentication
* Resourcepacks
* Fully functional web client
* Scripting API for moddable content
* Alternate Game Versions
* Minecraft Java server compatiblity
* Modded Minecraft Java server compatibility
