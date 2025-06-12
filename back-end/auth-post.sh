#!/bin/bash

TOKEN="eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzUxMiJ9.eyJpc3MiOiJxdW90ZS1zZXJ2ZXIucG84Lm9yZyIsInN1YiI6Ik5hdGhhbiBNZXRlbnMgPG5hdGhhbkBleGFtcGxlLmNvbT4iLCJleHAiOjE3NDk3OTU5ODl9.EEhvD2jxcjGLLPJ_DRv1eW3a26aXD99dX0XYPgSdhCe_SOuz7JV56ifGnRDJjegqnG2ZJONiyMa334wHRF9zPQ"

curl -X POST http://localhost:8000/api/v1/add-quote \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -d '{
    "id": "700",
    "quote": "Your quote here",
    "author": "Nathan",
    "tags": ["example", "auth"]
}'

curl -X DELETE http://localhost:8000/api/v1/delete-quote/700 \
  -H "Authorization: Bearer $TOKEN"

curl -X GET http://localhost:8000/api/v1/all-quotes \
  -H "Authorization: Bearer $TOKEN"
