# Back to Super Increasing Era Challenge source files

<p align="justify">In this repo are attached official source files of the challenge Back to Super Increasing Era I made for Hackday2026 CTF challenge.</p>

### Run the oracle on your host
<p alige="justify">Run the orcale using pyhton: </p>

````bash
python3 custom.py
````

### Curl oracle to encrypt payload

```bash
curl -X POST http://127.0.0.1:5000/custom_encryption/encrypt/ \
     -H "Content-Type: application/json" \
     -d '{"M": "Hello Hackday"}'
````

### Curl oracle to decrypt cipher

````bash
curl -X POST http://127.0.0.1:5000/custom_encryption/decrypt/ \
     -H "Content-Type: application/json" \
     -d '{"Cipher": 123456789, "MessageSizeBits": 112}'
````

> <p alige="justify">This server is a single thread server.</p> 
