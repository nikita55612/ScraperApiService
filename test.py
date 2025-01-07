import asyncio
from websockets.asyncio.client import connect
import requests

TOKEN_ID = "rs.SUyISUT06JWFY8AWEDLNVnvZHnLJU65K4"

async def ws():
    async with connect("ws://localhost:5050/test_ws") as websocket:
        try:
            while (msg := await websocket.recv()):
                print(msg)
        except Exception as _:
            print("Connection closed...")

def create_token():
    print("create_token")
    headers = {
        "Authorization": "Bearer 123"
    }
    params = {
        "ttl": 1000000,
        "op_limit": 20000,
        "tc_limit": 20
    }
    res = requests.post("http://localhost:5050/create_token/", headers=headers, params=params)
    print(res.text)

def cutout_token():
    print("cutout_token")
    headers = {
        "Authorization": "Bearer 123"
    }
    res = requests.delete("http://localhost:5050/cutout_token/rs.c8zY0IJpwoMQPw6neiJntdbJwRUOufl3h", headers=headers)
    print(res.text)

def token_info():
    print("token_info")
    headers = {
        "Authorization": "Bearer rs.c8zY0IJpwoMQPw6neiJntdbJwRUOufl3h"
    }
    res = requests.get("http://localhost:5050/token_info", headers=headers)
    print(res.text)

def token_info_():
    print("token_info_")
    res = requests.get("http://localhost:5050/token_info/rs.SUyISUT06JWFY8AWEDLNVnvZHnLJU65K4")
    print(res.text)

def update_token():
    print("update_token")
    headers = {
        "Authorization": "Bearer 123"
    }
    params = {
        "id": "rs.SUyISUT06JWFY8AWEDLNVnvZHnLJU65K4",
        "ttl": 1090009,
        "op_limit": 21000,
        "tc_limit": 19
    }
    res = requests.post("http://localhost:5050/update_token/", headers=headers, params=params)
    print(res.text)

def state():
    print("state")
    res = requests.get("http://localhost:5050/state")
    print(res.text)

def markets():
    print("markets")
    res = requests.get("http://localhost:5050/markets")
    print(res.text)

def ping():
    print("ping")
    res = requests.get("http://localhost:5050/ping")
    print(res.text)

def myip():
    print("myip")
    res = requests.get("http://localhost:5050/myip")
    print(res.text)

if __name__ == "__main__":
    create_token()
