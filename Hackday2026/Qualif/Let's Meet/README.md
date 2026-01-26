# Let's Meet Challenge source files
<p align="justify">In this repo are attached official source files of the challenge Let's Meet I made for Hackday2026 CTF challenge.</p>

### Run the docker on your host

<p align="justify">The docker setup assumes that the cargo application has already been built and copies binary from target/ folder. 
Hence be fore running the application through docker container, you must compile binaries at least one : </p>

````bash
cargo build

## or cargo run and shutdown after compilation ends(no otpion --release /!\)
````

<p align="justify">Once the compiled binaries are generated under target/, you can build and run the container using:</p>

````bash
docker compose up
````

> <p align="justify">You must install cargo rust utilities to be able to compile server binaries before running the server through docker</p>
> <p align="justify">Docker container exposes server on port 32778</p>
> <p align="justify">Once Image is built you can delete cargo cache and target folder using 'cargo clean'</p>

### Run the application on your host

<p align="justify">To run the application on your host you must compile server binaries first, as with docker version:</p>

````bash
cargo run
````
<p align="justify">The server will expose 2 ports :</p>

- 0.0.0.0:8000 (frontend)
- 127.0.0.1:5000 (API)
  
> You need a mongodb instance running on port 27017 (authentication-less)

<p align="justify">To write Admin profile in db, you must run init script attached to this repo:</p>

````bash
chmod +x init_db.sh
./init_db.sh
````
