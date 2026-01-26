####HACKDAY{M3rkl3_H3llm4n_crypt0syst3m_l4tt1c3_r3duct10n_4tt4ck_!!}
import requests

flag = "HACKDAY{M3rkl3_H3llm4n_crypt0syst3m_l4tt1c3_r3duct10n_4tt4ck_!!}"
chunks = [flag[i:i+8] for i in range(0, len(flag), 8)]
url = "http://127.0.0.1:5000/custom_encryption/encrypt/"


for idx, chunk in enumerate(chunks):
    #print(f"Chunk {idx} : {chunk} (length: {len(chunk)})")
    response = requests.post(url, json={"M": chunk})
    print(response.json())

