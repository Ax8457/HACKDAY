# The Lotery Challenge source files
<p align="justify">In this repo are attached official source files of the challenge The Lotery I made for Hackday2026 CTF challenge.</p>

### Run the docker on your host

<p align="justify">WASM binary behing the application is already compiled using emscripten and attached to this repo. To build and run the application: </p>

````bash
cd docker
docker compose up
````

<p align="justify">The server exposes port 33779.</p>

### Run the application on your host

<p align="justify">To run the application on your host you can simply deploy a python server in the folder of the index.html file: </p>

````bash
python3 -m http.server
````

### Recompile binary

<p align="justify">If you want to recompile wasm you do it using emscripten: </p>

````bash
emcc chall_hackday_wipin.c chall.c   -s USE_SDL=2   -s USE_SDL_TTF=2   -O3   -s INITIAL_MEMORY=64MB   -s ALLOW_MEMORY_GROWTH=1   --preload-file rules.txt   --preload-file Ithaca.ttf   -o index.html -s MINIFY_HTML=0 -s ASSERTIONS=1 -s FORCE_FILESYSTEM=1 -s FULL_ES2=1
````

> You must back up index.html, otherwise emscripten will override current one and custom integrations and styles will be lost
> You're very likely to face issues with frozen cach, to solve  it your must edit /usr/share/emscripten/tools/config_template.py or /usr/share/emscripten/tools/config_template.py
