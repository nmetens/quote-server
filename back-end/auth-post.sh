#!/bin/bash

access_token="eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzUxMiJ9.eyJpc3MiOiJxdW90ZS1zZXJ2ZXIucG84Lm9yZyIsInN1YiI6Ik5hdGhhbiBNZXRlbnMgPG5hdGhhbkBleGFtcGxlLmNvbT4iLCJleHAiOjE3NDk3NjU3NTV9.FWOedZkEmWOnE-0JS0928OzhYkKZsDSf--QAFh07qD2dp3h4MG_jiFkbghzuCRd7tUQm4rlAgP8cRSnRfDZ41A"

curl -X POST http://localhost:8000/api/v1/add-quote \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $access_token" \
  -d '{
    "id": "700",
    "quote": "Your quote here",
    "author": "Nathan",
    "tags": ["example", "auth"]
}'

curl -X DELETE http://localhost:8000/api/v1/delete-quote/700 \
  -H "Authorization: Bearer $access_token"

curl -X GET http://localhost:8000/api/v1/all-quotes \
  -H "Authorization: Bearer $access_token"
