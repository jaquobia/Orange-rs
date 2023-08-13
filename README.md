# Orange-rs

A modernization of the b1.7.3 client with focus on server play and authentication.
Modernization includes concepts like blockstates, resourcepacks, data-driven content, and potential protocol expansion

Orange will also serve as my testing grounds for new libraries, such as Rine for applications, and orange-networking for network communications.

# Running
The game runs with downloaded assets through  
```cargo run --release -p orange_rs_client```  
And an assets path can be specified with  
```cargo run --release -p orange_rs_client -- -a path/to/assets```  

# Configuring
The config and assets directory are by default stored in %APPDATA%/.orange for windows,
and /home/USERNAME/.local/share/.orange

These options can be used to specify the offline username and the stored server to load into the ip bar every time orange is started

# Goals
* Resourcepacks (In Progress)
* Minecraft Java server compatiblity (In Progress)
* Authentication

# Current Progress
Ability to load a 10 chunk radius render distance around the players position and display most blocks
Transparency issues, occlusion culling issues, missing models, incomplete resourcepack system, buggy lighting

# Todo
* Add Entities
* Add Tile Entities? (Necessity uncertain.)
* Add Items
* Add collision
* Add "Screens"
* Add gui's and containers  
* ~~Generate chunks and manage server operations on a separate thread~~
* ~~Add world saving and loading (Anvil Format)~~

# Possible features
* Fully functional web client (requires being able to load assets and use multiple threads through a wasm runtime)
* Scripting API for moddable content
* Alternate Game Versions
* Modded Minecraft Java server compatibility
