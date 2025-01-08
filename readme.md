# To run
Run with `./run.sh`.
Right now your shell has to be located in the same folder as the assets folder or else assets will not load.

The game runs on Rust nightly (1.85 to be exact), so you need to use this version of the compiler
along with installing the compilation target for `wasm32-unknown-unknown`. 
If you are using [Nix](https://nixos.org/) this is automatically set up
when you use `nix develop`.

# To create maps:
Maps are developed in the [Map](https://quakewiki.org/wiki/Quake_Map_Format) format first 
developed for Quake. Any editor supporting this format should be usable but
[TrenchBroom](https://trenchbroom.github.io/) has been used so far. 
The following chapter shows how to set up TrenchBroom for Ondth development.

Maps should be saved in `assets/maps` folder, or a subfolder there.

I can recommend the TrenchBroom tutorial by 
[dumptruck_ds](https://www.youtube.com/@dumptruckds)
to get started:
[youtube.com/playlist?list=PLgDKRPte5Y0AZ_K_PZbWbgBAEt5xf74aE&si=Gvt-tgHmPAKhJjGQ](https://www.youtube.com/playlist?list=PLgDKRPte5Y0AZ_K_PZbWbgBAEt5xf74aE).
Keep in mind that everything in this tutorial might not be applicable,
as Ondth does not do scripting etc the same way Quake does, but it should get you 
started with brush editing.

If you are using [Nix](https://nixos.org/) you can simply run `nix develop` to have TrenchBroom
(`trenchbroom`) added to your path. If you are not using Nix package manager
or NixOS you are on your own when it comes to installation, but checking the official
website should suffice.

## Step 1.
To create a new map press the `New map...` button.
![alt](./readme/step%201.png)
## Step 2. 
If this is your first time setting up TrenchBroom for Ondth follow these steps otherwise 
you can skip to step 4.

Press the `Open preferences...` button.
![alt](./readme/step%202.png)
## Step 3
Go to the `Generic` game and set the game path. This should be set to the `assets` folder 
found at the project root. After that press the `Ok` button.
![alt](./readme/step%203.png)
## Step 4
Select the game, set the format to `Standard` and then press the `Ok` button.
![alt](./readme/step%204.png). Only limited support for 
[Valve's format](https://developer.valvesoftware.com/wiki/MAP_(file_format)#Valve220)
exists at the moment, but this might change in the future. 
Support for the 
[Quake 2 format](https://developer.valvesoftware.com/wiki/MAP_(file_format)#Quake_II) is
current non-existent.
## Step 5
After entering the editor go to the `Entity` tab.
![alt](./readme/step%205.png)
## Step 6
Press the `Settings` button.
![alt](./readme/step%206.png)
## Step 7
Press the `Browse` button.
![alt](./readme/step%207.png)
## Step 8
Select the file `Base.fgd` located in the `assets` folder.
![alt](./readme/step%208.png)
## Step 9
Select the option `Relative to game directory`. Press the `Ok` button.
![alt](./readme/step%209.png)
## Step 10
Press the `Browser` button.
![alt](./readme/step%2010.png)
## Step 11
Now the games entities should be visible like this.
![alt](./readme/step%2011.png)
## Step 12
Press the `Face` button.
![alt](./readme/step%2012.png)
## Step 13
Press the `Settings` button.
![alt](./readme/step%2013.png)
## Step 14
Select all the texture folders on the left side, and then press the 
`+` button. If you add any more texture folders in the future, you will
need to enable them here.
![alt](./readme/step%2014.png)
## Step 15
Now press `Browser` button to go back.
![alt](./readme/step%2015.png)
## Step 16
Now you can select the games textures in the editor!
![alt](./readme/step%2016.png)