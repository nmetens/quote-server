#!/bin/bash

set -a
source .env
set +a

curl -X POST http://localhost:8000/api/v1/register \
  -H "Content-Type: application/json" \
  -d "{
    \"full_name\": \"Nathan Metens\",
    \"email\": \"nathan@example.com\",
    \"password\": \"$REG_PASSWORD\"
  }"
