import requests

url = "https://wcode.net/api/account/billing/grants"

payload = {}
headers = {
  'Authorization': 'sk-528.kT3wdhoKY531DD59egtWtRZKT8deOwLVo0i0IxorxyQVePoY'  # <-------- TODO: 替换这里的 API_KEY
}

response = requests.request("GET", url, headers=headers, data=payload)

print(response.text)

# title