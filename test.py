from websockets.sync.client import connect
import requests


TOKEN = "rs.vHGzpkVPfVFao7LG9EiSV86k2"
MASTER_TOKEN = "1234"
DOMEN = "http://localhost:5050"
API_URL = f"{DOMEN}/api/v1"

def create_token():
    print("create_token")
    headers = {
        "Authorization": f"Bearer {MASTER_TOKEN}"
    }
    params = {
        "ttl": 1000000,
        "op_limit": 20000,
        "tc_limit": 20
    }
    res = requests.post(f"{API_URL}/create_token/", headers=headers, params=params)
    print(res.json())

def update_token():
    print("update_token")
    headers = {
        "Authorization": f"Bearer {MASTER_TOKEN}"
    }
    params = {
        "id": "rs.SUyISUT06JWFY8AWEDLNVnvZHnLJU65K4",
        "ttl": 1090009,
        "op_limit": 21000,
        "tc_limit": 19
    }
    res = requests.post(f"{API_URL}/update_token/", headers=headers, params=params)
    print(res.json())

def cutout_token():
    print("cutout_token")
    headers = {
        "Authorization": f"Bearer {MASTER_TOKEN}"
    }
    res = requests.delete(f"{API_URL}/cutout_token/rs.c8zY0IJpwoMQPw6neiJntdbJwRUOufl3h", headers=headers)
    print(res.json())

def token_info():
    print("token_info")
    headers = {
        "Authorization": f"Bearer {TOKEN}"
    }
    res = requests.get(f"{API_URL}/token_info", headers=headers)
    print(res.json())

def token_info_():
    print("token_info_")
    res = requests.get(f"{API_URL}/token_info/rs.SUyISUT06JWFY8AWEDLNVnvZHnLJU65K4")
    print(res.json())

def test_token():
    print("test_token")
    res = requests.get(f"{API_URL}/test-token")
    print(res.text)

def state():
    print("state")
    res = requests.get(f"{API_URL}/state")
    print(res.json())

def markets():
    print("markets")
    res = requests.get(f"{API_URL}/markets")
    print(res.text)

def ping():
    print("ping")
    res = requests.get(f"{API_URL}/ping")
    print(res.json())

def myip():
    print("myip")
    res = requests.get(f"{API_URL}/myip")
    print(res.json())

def task_ws(order_hash):
	headers = {
		"Authorization": f"Bearer {TOKEN}"
	}
	with connect(f"ws://localhost:5050/api/v1/task_ws/{order_hash}", additional_headers=headers) as task_ws:
		try:
			while (task := task_ws.recv()):
				print(task)
		except Exception as _:
			print("Connection closed...")

def task_(order_hash):
	print("task")
	headers = {
		"Authorization": f"Bearer {TOKEN}"
	}

	res = requests.post(f"{API_URL}/task/{order_hash}", headers=headers)
	print(res.text)
	return res.text

def order():
	print("order")
	headers = {
		"Authorization": f"Bearer {TOKEN}"
	}
	order = {
		"products": [
			"ym/1732949807-100352880819-5997015",
			"oz/1596079870",
			"oz/1793879666",
			"wb/145700662",
		],
		"proxyPool": [
			"1kpF8S:GPnFUb@147.45.62.117:8000"
		]
	}

	res = requests.post(f"{API_URL}/order", headers=headers, json=order)
	print(res.text)

	return res.text

if __name__ == "__main__":
	import time

	order()
	#task_ws(order_hash)
