# As Small As Possible Challenge source files
<p align="justify">In this repo are attached official source files of the challenge As Small As Possible I made for Hackday2026 CTF challenge.</p>

### Run the docker on your host

<p align="justify">You can run the oracle through a docker container using:</p>

````bash
docker compose up
````

<p align="justify">The server exposes port 4444.</p>


### Run the application on your host

<p align="justify">To run the application on your host you, use python: </p>

````bash
python3 DH_oracle.py
````

<p align="justify">The server exposes port 4444.</p>

### Bind socket
<p align="justify">The oracle is exposed on a TCP socket you can bind using nectact: </p>

````bash
nc localhost 4444
````

> The server exposes a multi threaded socket.
