import socket
import random
import hashlib
import os
import json
from Crypto.Cipher import AES
import threading

FLAG = "Hackday{D1ff13_H3llm4n_Sm4ll_Subgr0ups_4tt4ck_$$$!!}"
Alice_message = "Welcome on As Small As Possible Challenge - Hackday 2026 - Chall by 0x4X3L"

g = 2
p = 20321128995987616756483325054626935149463293864989548829208395564815070181789010804680829746553442903924545722921456474841378536063088732360944556082163198120055756500279467749665181248071556864415766255786098359143918642269171273177381721465327312073852498155166663504135007996617441861328149318088038651749417829311255949105106948012256557083842826432026681950620384455629467158365453401666276578179214568421220053490633027847533558869435684387881964388466221688904782873277108420442957105393653559422515949053825483740343805065278939830360501675305754797054665278794274455140895594221088103413859016533691895587603 


ALICE_PRIVKEY = random.getrandbits(1024)
BOB_PRIVKEY = random.getrandbits(1024)
ALICE_PUBKEY = pow(g, ALICE_PRIVKEY, p)
BOB_PUBKEY = pow(g, BOB_PRIVKEY, p)

def encrypt_payload(message, secret):
    key = hashlib.sha256(str(secret).encode()).digest()
    iv = os.urandom(12)
    cipher = AES.new(key, AES.MODE_GCM, nonce=iv)
    ciphertext, tag = cipher.encrypt_and_digest(message.encode())
    return {"iv": iv.hex(), "ciphertext": ciphertext.hex(), "tag": tag.hex()}

SHARED_SECRET_AB = pow(ALICE_PUBKEY, BOB_PRIVKEY, p)
FLAG_INTERCEPTED = encrypt_payload(f"SECRET_FLAG: {FLAG}", SHARED_SECRET_AB)

def handler(conn):
    try:
        conn.settimeout(15.0)
        conn.send(b"=== TRAFFIC INTERCEPTED ===\n")
        conn.send(f"Payload: {json.dumps(FLAG_INTERCEPTED)}\n".encode())
        conn.send(b"===========================\n\n")
        
        conn.send(b"1. Alice (receive an encrypted message)\n2. Bob (send encrypted message)\n> ")
        choice = conn.recv(1024).decode().strip()
        
        if "1" in choice:
            conn.send(f"Alice PubKey: {ALICE_PUBKEY}\np: {p}\n".encode())
            conn.send(b"Your Key: ")
            data = conn.recv(2048).decode().strip()
            
            if data.isdigit():
                user_pub = int(data)
                res = encrypt_payload(Alice_message, pow(user_pub, ALICE_PRIVKEY, p))
                conn.send(f"Alice: {json.dumps(res)}\n".encode())
            else:
                conn.send(b"Error: Public key must be an integer.\n")

        elif "2" in choice:
            conn.send(f"Bob PubKey: {BOB_PUBKEY}\np: {p}\n".encode())
            conn.send(b"Your Key: ")
            data = conn.recv(2048).decode().strip()
            if data.isdigit():
                conn.send(b"Bob: Now send me your encrypted message: ")
                _ = conn.recv(4096)
                conn.send(b"Bob: Message received. Connection closed.\n")

    except Exception:
        pass
    finally:
        conn.close()

def main():
    server = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    server.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
    try:
        server.bind(('0.0.0.0', 4444))
    except Exception as e:
        print(f"Bind failed: {e}")
        return

    server.listen(200)
    print(f"Server listening on port 4444...")
    print(f"Alice PubKey: {ALICE_PUBKEY}")

    while True:
        client_conn, addr = server.accept()
        client_thread = threading.Thread(target=handler, args=(client_conn,))
        client_thread.daemon = True 
        client_thread.start()

if __name__ == "__main__":
    main()
